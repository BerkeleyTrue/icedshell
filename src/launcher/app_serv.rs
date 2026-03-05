use std::{
    collections::{BTreeMap, BTreeSet},
    env::VarError,
    ffi::OsStr,
    path::PathBuf,
};

use derive_more::{Constructor, Deref, DerefMut, From};
use freedesktop_entry_parser::{Entry, parse_entry};
use iced::{Subscription, Task, advanced::graphics::futures::MaybeSend};
use itertools::Itertools;
use tokio::fs;
use tracing::info;

use crate::{
    fdo_icons::{self, FdIcon},
    feature::Service,
};

#[derive(Debug, Clone, Constructor)]
pub struct AppDesc {
    pub name: String,
    pub exec: String,
    pub comment: Option<String>,
    pub try_exec: Option<String>,
    pub icon: Option<FdIcon>,
}

#[derive(Debug, Deref, DerefMut, From, Clone, Default)]
pub struct AppNameToAppMap(BTreeMap<String, AppDesc>);

#[derive(Debug, Clone)]
pub enum Message {
    LoadApps(AppNameToAppMap),
}

pub struct AppServ {
    apps: AppNameToAppMap,
}

impl Service for AppServ {
    type Message = Message;
    type Init = ();

    fn new<O: MaybeSend + 'static>(
        _input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let init = Task::future(async {
            get_apps()
                .await
                .map(Message::LoadApps)
                .inspect_err(|err| {
                    info!("Error loading apps: {err:?}");
                })
                .unwrap_or_else(|_| Message::LoadApps(AppNameToAppMap::default()))
        });
        (
            Self {
                apps: AppNameToAppMap::from(BTreeMap::new()),
            },
            init.map(f),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::LoadApps(apps) => {
                info!("apps: {apps:?}");
                self.apps = apps;
                Task::none()
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ListArgs {
    pub skip: usize,
    pub limit: usize,
}

impl AppServ {
    pub fn list(&self, ListArgs { skip, limit }: ListArgs) -> impl Iterator<Item = &AppDesc> {
        self.apps.values().skip(skip).take(limit)
    }
}

fn get_bin_dirs() -> anyhow::Result<BTreeSet<PathBuf>> {
    let paths: BTreeSet<_> = std::env::var("PATH")?
        .split(":")
        .filter_map(|path_str| PathBuf::from(path_str).canonicalize().ok())
        .collect();

    Ok(paths)
}

fn get_data_dirs() -> anyhow::Result<Vec<PathBuf>> {
    let mut data_dir: Vec<_> = std::env::var("XDG_DATA_DIRS")
        .or_else(|err| match err {
            VarError::NotPresent => Ok("/usr/local/share:/usr/share".to_owned()),
            _ => Err(err),
        })?
        .split(":")
        .filter_map(|path_str| {
            PathBuf::from(format!("{path_str}/applications"))
                .canonicalize()
                .ok()
        })
        .filter(|data_dir| data_dir.exists())
        .dedup()
        .collect();

    // initial data dirs have higher priority
    data_dir.reverse();

    Ok(data_dir)
}

async fn get_apps() -> anyhow::Result<AppNameToAppMap> {
    let paths = get_bin_dirs()?;
    let data_dirs = get_data_dirs()?;

    let mut apps = AppNameToAppMap::from(BTreeMap::new());

    for data_dir in data_dirs.iter() {
        let mut entries = fs::read_dir(data_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path
                .file_stem()
                .and_then(|filename| filename.to_str())
                .map(|s| s.to_owned())
                .unwrap_or("na".to_owned());

            if path.extension() == Some(OsStr::new("desktop")) {
                let desktop = tokio::task::spawn_blocking(|| parse_entry(path)).await??;
                let is_visible_app = desktop
                    .get_desk_entry("Type")
                    .is_some_and(|typo| typo == "Application")
                    && desktop.get_desk_entry("Hidden").is_none_or(|s| s != "True")
                    && desktop
                        .get_desk_entry("NoDisplay")
                        .is_none_or(|s| s != "True");

                let name = desktop.get_desk_entry("Name");
                let exec = desktop.get_desk_entry("Exec");

                let try_exec = desktop.get_desk_entry("TryExec");
                let comment = desktop.get_desk_entry("Comment");
                let icon = desktop.get_desk_entry("Icon").cloned();
                let icon = tokio::task::spawn_blocking(move || {
                    icon.and_then(|name| fdo_icons::find(&name))
                })
                .await?;

                if is_visible_app
                    && let Some(name) = name
                    && let Some(exec) = exec
                    && verify_exec(exec, try_exec, &paths).await
                {
                    let exec = exec
                        .split_whitespace()
                        .filter(|token| !token.starts_with("%"))
                        .collect::<Vec<_>>()
                        .join(" ");

                    apps.insert(
                        file_name,
                        AppDesc::new(
                            name.to_owned(),
                            exec.to_owned(),
                            comment.cloned(),
                            try_exec.cloned(),
                            icon,
                        ),
                    );
                }
            }
        }
    }

    Ok(apps)
}

async fn verify_exec(exec: &String, try_exec: Option<&String>, paths: &BTreeSet<PathBuf>) -> bool {
    let maybe_exec = try_exec
        .unwrap_or(exec)
        .split_whitespace()
        // grab first part, should be exec name or full path
        .next()
        .map(PathBuf::from);

    if let Some(exec_path) = maybe_exec {
        if exec_path.is_absolute() {
            exec_path.exists()
        } else {
            paths.iter().any(move |path| path.join(&exec_path).exists())
        }
    } else {
        false
    }
}

trait EntryExt {
    fn get_desk_entry(&self, attr: impl AsRef<str>) -> Option<&String>;
}

impl EntryExt for Entry {
    fn get_desk_entry(&self, attr: impl AsRef<str>) -> Option<&String> {
        self.get("Desktop Entry", attr)
            .and_then(|entries| entries.first())
    }
}

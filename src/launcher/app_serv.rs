use std::{
    cmp,
    collections::{BTreeMap, BTreeSet, HashMap},
    env::VarError,
    ffi::OsStr,
    ops::Mul,
    path::{Path, PathBuf},
};

use derive_more::{Constructor, Deref, DerefMut, From};
use freedesktop_entry_parser::{Entry, parse_entry};
use iced::{Subscription, Task, advanced::graphics::futures::MaybeSend};
use itertools::Itertools;
use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::info;

use crate::{
    fdo_icons::{self, FdIcon},
    feature::Service,
};

#[derive(Debug, Clone, Constructor)]
pub struct AppDesc {
    pub name: String,
    pub count: usize,
    pub app_id: String,
    pub exec: String,
    pub gen_name: Option<String>,
    pub comment: Option<String>,
    pub try_exec: Option<String>,
    pub icon: Option<FdIcon>,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Deref, DerefMut, From, Clone, Default)]
pub struct AppNameToAppMap(BTreeMap<String, AppDesc>);

#[derive(Debug, Clone, Constructor, PartialEq, Eq, Default)]
pub struct Query {
    query: Option<String>,
    page: usize,
    page_size: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadApps(AppNameToAppMap),
    LoadCache(CountCache),
    Query(Query),
}

pub struct AppServ {
    count_cache: CountCache,
    apps: AppNameToAppMap,
    last_query: Query,
    pub res: Vec<AppDesc>,
}

impl Service for AppServ {
    type Message = Message;
    type Init = ();

    fn new<O: MaybeSend + 'static>(
        _input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let init_apps = Task::future(async {
            get_apps()
                .await
                .map(Message::LoadApps)
                .inspect_err(|err| {
                    info!("Error loading apps: {err:?}");
                })
                .unwrap_or(Message::LoadApps(AppNameToAppMap::default()))
        });
        let init_cache = Task::future(async {
            CountCache::load()
                .await
                .inspect_err(|err| {
                    info!("Error loading cache: {err:?}");
                })
                .map(Message::LoadCache)
                .unwrap_or(Message::LoadCache(CountCache::default()))
        });
        (
            Self {
                count_cache: CountCache::default(),
                apps: AppNameToAppMap::default(),
                res: Vec::new(),
                last_query: Query::default(),
            },
            Task::batch([init_apps, init_cache]).map(f),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::LoadApps(apps) => {
                self.apps = apps;
                for (app_id, count) in self.count_cache.iter() {
                    self.apps.entry(app_id.clone()).and_modify(move |app| {
                        app.count = *count;
                    });
                }

                Task::done(Message::Query(Query::new(None, 0, 10)))
            }
            Message::LoadCache(cache) => {
                self.count_cache = cache;

                for (app_id, count) in self.count_cache.iter() {
                    self.apps.entry(app_id.clone()).and_modify(move |app| {
                        app.count = *count;
                    });
                }

                Task::done(Message::Query(Query::new(None, 0, 10)))
            }
            Message::Query(query) => {
                self.last_query = query.clone();
                self.res = self.list(query.into());
                Task::none()
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ListArgs {
    pub query: Option<String>,
    pub skip: usize,
    pub limit: usize,
}

impl From<Query> for ListArgs {
    fn from(value: Query) -> Self {
        Self {
            query: value.query,
            skip: value.page,
            limit: value.page_size,
        }
    }
}

impl AppServ {
    pub fn list(&self, ListArgs { query, skip, limit }: ListArgs) -> Vec<AppDesc> {
        let mut apps: Vec<_> = self.apps.values().collect();

        if let Some(query) = query {
            apps = Self::match_list(query, apps);
        } else {
            apps.sort_by_key(|app| cmp::Reverse(app.count));
        }

        apps.into_iter().skip(skip).take(limit).cloned().collect()
    }

    fn match_list(query: String, items: Vec<&AppDesc>) -> Vec<&AppDesc> {
        let pattern = Pattern::parse(&query, CaseMatching::Ignore, Normalization::Smart);

        if pattern.atoms.is_empty() {
            return items;
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut buff = Vec::new();

        items
            .into_iter()
            .filter_map(|app| {
                let name_score = {
                    let haystack = Utf32Str::new(&app.name, &mut buff);
                    pattern
                        .score(haystack, &mut matcher)
                        .map(|score| score.mul(120))
                };
                let gen_name_score = {
                    app.gen_name.as_ref().and_then(|name| {
                        let haystack = Utf32Str::new(name, &mut buff);
                        pattern
                            .score(haystack, &mut matcher)
                            .map(|score| score.mul(100))
                    })
                };
                let cat_score = {
                    app.categories.as_ref().and_then(|cats| {
                        pattern
                            .match_list(cats, &mut matcher)
                            .iter()
                            .max_by_key(|(_, score)| cmp::Reverse(*score))
                            .map(|(_, score)| score.mul(100))
                    })
                };
                name_score.or(gen_name_score).or(cat_score).map(|_| {
                    let score = name_score
                        .unwrap_or_default()
                        .max(gen_name_score.unwrap_or_default())
                        .max(cat_score.unwrap_or_default());

                    (app, score)
                })
            })
            .sorted_by_key(|(_, score)| cmp::Reverse(*score))
            .map(|(app, _)| app)
            .collect()
    }

    pub fn exec(&self, app: &AppDesc) -> anyhow::Result<()> {
        let exec = app.exec.to_owned();
        tokio::process::Command::new(exec)
            .process_group(0)
            .spawn()?;

        Ok(())
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
            let app_id = path
                .file_stem()
                .and_then(|filename| filename.to_str())
                .map(|s| s.to_owned())
                .unwrap_or("na".to_owned());

            if path.extension() == Some(OsStr::new("desktop")) {
                let desktop = tokio::task::spawn_blocking(|| parse_entry(path)).await??;
                let is_visible_app = desktop
                    .get_desk_entry("Type")
                    .is_some_and(|typo| typo == "Application")
                    && desktop.get_desk_entry("Hidden").is_none_or(|s| s != "true")
                    && desktop
                        .get_desk_entry("NoDisplay")
                        .is_none_or(|s| s != "true");

                let name = desktop.get_desk_entry("Name");
                let exec = desktop.get_desk_entry("Exec");

                let try_exec = desktop.get_desk_entry("TryExec");
                let comment = desktop.get_desk_entry("Comment");
                let icon = desktop.get_desk_entry("Icon").cloned();
                let icon = tokio::task::spawn_blocking(move || {
                    icon.and_then(|name| fdo_icons::find(&name))
                })
                .await?;
                let gen_name = desktop.get_desk_entry("GenericName");
                let categores = desktop.get_desk_entry("Categories").map(|cats| {
                    cats.split(";")
                        .map(|str| str.to_owned())
                        .collect::<Vec<String>>()
                });

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
                        app_id.clone(),
                        AppDesc::new(
                            name.to_owned(),
                            0,
                            app_id,
                            exec.to_owned(),
                            gen_name.cloned(),
                            comment.cloned(),
                            try_exec.cloned(),
                            icon,
                            categores,
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

#[derive(Debug, Constructor, Deref, DerefMut, Default, Deserialize, Serialize, Clone)]
pub struct CountCache(pub HashMap<String, usize>);

impl CountCache {
    fn get_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or(PathBuf::from("."))
            .join("icedshell/launcher_counts.json")
    }

    async fn ensure_dir(path: &Path) -> anyhow::Result<()> {
        if let Some(basename) = path.parent() {
            info!("basename: {basename:?}");
            fs::create_dir_all(basename).await?;
        }

        Ok(())
    }

    async fn load() -> anyhow::Result<Self> {
        let path = Self::get_path();

        Self::ensure_dir(&path).await?;

        if !fs::try_exists(&path).await? {
            return Ok(Self::default());
        };

        fs::read_to_string(&path)
            .await
            .map_err(anyhow::Error::from)
            .and_then(|file_str| serde_json::from_str(&file_str).map_err(anyhow::Error::from))
    }

    async fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_path();
        Self::ensure_dir(&path).await?;

        let to_save = serde_json::to_string(self)?;

        fs::write(&path, &to_save).await?;
        Ok(())
    }

    fn inc_count(&mut self, app_id: String) {
        *self.entry(app_id).or_default() += 1;
    }
}

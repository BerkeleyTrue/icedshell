use std::{
    collections::{BTreeMap, BTreeSet},
    env::VarError,
    ffi::OsStr,
    path::PathBuf,
};

use derive_more::{Constructor, Deref, DerefMut, Display, From};
use freedesktop_entry_parser::{Entry, parse_entry};
use iced::{
    Border, Event, Length, Task,
    alignment::Vertical,
    border, event,
    keyboard::{self, Key, key::Named},
    padding,
    widget::{column, container, operation::focus, row, text, text_input},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use itertools::Itertools;
use tokio::fs;
use tracing::info;

use crate::{
    feature::{Comp, Feature, align_center},
    theme::CAT_THEME,
};

#[derive(Debug, Clone, Constructor)]
struct Application {
    name: String,
    exec: String,
    comment: Option<String>,
    try_exec: Option<String>,
    icon: Option<String>,
}

#[derive(Debug, Deref, DerefMut, From)]
struct AppNameToAppMap(BTreeMap<String, Application>);

#[derive(Clone, Debug, Display)]
enum PromptType {
    Run,
}

#[derive(Clone, Debug)]
pub enum Message {
    Close,
    SearchUpdated(String),
}

pub struct Launcher {
    search: String,
    prompt_type: PromptType,
}

impl Comp for Launcher {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> (Self, Task<Self::Message>) {
        (
            Self {
                prompt_type: PromptType::Run,
                search: "".to_string(),
            },
            Task::future(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            })
            .discard()
            .chain(focus::<Message>("search-input")),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen_with(|event, _, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                ..
            }) => Some(Message::Close),
            _ => None,
        })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Close => {
                info!("close window");
                Task::none()
            }
            Message::SearchUpdated(search) => {
                self.search = search;
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let prompt = {
            let size = theme.spacing().lg();
            let input = text_input("", &self.search)
                .id("search-input")
                .width(Length::Fill)
                .padding(padding::horizontal(theme.spacing().xs()))
                .style(|_, _| text_input::Style {
                    background: theme.background().into(),
                    border: Border::default(),
                    icon: theme.text_color(),
                    placeholder: theme.subtext0(),
                    value: theme.text_color(),
                    selection: theme.subtext1(),
                })
                .size(size)
                .on_input(Message::SearchUpdated);

            let prompt = self.prompt_type.to_string();
            let prompt = text!("{prompt} >").size(size);

            align_center!(row![prompt, input])
                .padding(padding::right(theme.spacing().md()))
                .height(theme.spacing().xl2())
        };

        let results = { row![text!("food")].height(Length::Fill) };

        let content = column![prompt, results].height(Length::Fill);

        container(content)
            .align_y(Vertical::Top)
            .style(|_| container::Style {
                background: Some(theme.background().into()),
                text_color: Some(theme.text_color()),
                border: border::color(theme.pink())
                    .width(theme.spacing().xs())
                    .rounded(theme.radius().sm()),
                ..Default::default()
            })
            .padding(theme.spacing().lg())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Feature for Launcher {
    type Settings = NewLayerShellSettings;

    fn layer(&self) -> Self::Settings {
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((800, 600)),
            anchor: Anchor::empty(),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            output_option: OutputOption::LastOutput,
            namespace: Some("AppLauncher".into()),
            events_transparent: false,
            exclusive_zone: None,
            margin: None,
        }
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
                let icon = desktop.get_desk_entry("Icon");

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
                        Application::new(
                            name.to_owned(),
                            exec.to_owned(),
                            comment.cloned(),
                            try_exec.cloned(),
                            icon.cloned(),
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

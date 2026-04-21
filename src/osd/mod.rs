use clap::{Args, Subcommand};
use derive_more::Display;
use iced::{
    Length, Task, border,
    widget::{container, row, text},
};
use iced_font_awesome::{fa_icon, fa_icon_solid};
use iced_layershell::reexport::{self as layer, OutputOption};
use serde::{Deserialize, Serialize};

use crate::{
    feature::{Comp, Feature},
    theme::CAT_THEME,
    types::MonitorId,
};

#[derive(Debug, Clone)]
pub struct Init {
    pub monitor: Option<MonitorId>,
    pub command: OsdCommand,
}

#[derive(Debug, Clone, Display, Subcommand, Serialize, Deserialize)]
pub enum VolumeLevel {
    Inc,
    Dec,
    Mut,
}

#[derive(Debug, Clone, Display, Subcommand, Serialize, Deserialize)]
pub enum BrightLevel {
    Inc,
    Dec,
}

#[derive(Debug, Clone)]
pub enum Modi {
    Volume(VolumeLevel, Option<usize>),
    Brightness(BrightLevel, Option<usize>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Timeout,
}

pub struct Osd {
    monitor: Option<MonitorId>,
    modi: Modi,
}

impl Comp for Osd {
    type Message = Message;
    type Init = Init;

    fn new<O: iced::advanced::graphics::futures::MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + iced::advanced::graphics::futures::MaybeSend + 'static,
    ) -> (Self, iced::Task<O>) {
        let modi = match input.command {
            OsdCommand::Volume(VolArgs { command, val }) => Modi::Volume(command, val),
            OsdCommand::Bright(BrightArgs { command, val }) => Modi::Brightness(command, val),
        };
        let timeout = Task::perform(
            tokio::time::sleep(tokio::time::Duration::from_millis(650)),
            |_| Message::Timeout,
        )
        .map(f);
        (
            Self {
                monitor: input.monitor,
                modi,
            },
            timeout,
        )
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Timeout => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();
        let icon = match self.modi {
            Modi::Volume(VolumeLevel::Inc, _) => fa_icon_solid("volume-high"),
            Modi::Volume(VolumeLevel::Dec, _) => fa_icon_solid("volume-low"),
            Modi::Volume(VolumeLevel::Mut, _) => fa_icon_solid("volume-xmark"),

            Modi::Brightness(BrightLevel::Inc, _) => fa_icon_solid("lightbulb"),
            Modi::Brightness(BrightLevel::Dec, _) => fa_icon("lightbulb"),
        };
        let val = match self.modi {
            Modi::Volume(_, val) => val,
            Modi::Brightness(_, val) => val,
        }
        .map(|val| val.clamp(0, 100));

        let val = val.map(|val| text!("{}", val).size(spacing.xl3()).color(theme.subtext0()));

        let icon = icon.size(spacing.xl3()).color(theme.subtext0());

        let txt = match (icon, val) {
            (icon, Some(val)) => row!(icon, val).spacing(spacing.md()),
            (icon, None) => row!(icon),
        };

        container(txt)
            .style(move |_| container::Style {
                background: Some(theme.crust().into()),
                border: border::rounded(theme.radius().lg()),
                ..Default::default()
            })
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

impl Feature for Osd {
    type Settings = layer::NewLayerShellSettings;
    fn layer(&self) -> Self::Settings {
        Self::Settings {
            size: Some((200, 100)),
            layer: layer::Layer::Overlay,
            anchor: layer::Anchor::empty(),
            margin: None,
            keyboard_interactivity: layer::KeyboardInteractivity::None,
            output_option: self
                .monitor
                .as_ref()
                .map(|monitor| OutputOption::OutputName(monitor.inner().to_owned()))
                .unwrap_or(OutputOption::None),
            exclusive_zone: None,
            events_transparent: false,
            namespace: Some("IcedOsd".to_owned()),
        }
    }
}

#[derive(Debug, Args, Clone, Display)]
pub struct OsdArgs {
    #[command(subcommand)]
    pub command: OsdCommand,
}

#[derive(Debug, Subcommand, Clone, Display, Serialize, Deserialize)]
pub enum OsdCommand {
    Volume(VolArgs),
    Bright(BrightArgs),
}

#[derive(Debug, Args, Clone, Display, Serialize, Deserialize)]
#[display("{command}({val:?})")]
pub struct VolArgs {
    #[command(subcommand)]
    pub command: VolumeLevel,
    pub val: Option<usize>,
}

#[derive(Debug, Args, Clone, Display, Serialize, Deserialize)]
#[display("{command}({val:?})")]
pub struct BrightArgs {
    #[command(subcommand)]
    pub command: BrightLevel,
    pub val: Option<usize>,
}

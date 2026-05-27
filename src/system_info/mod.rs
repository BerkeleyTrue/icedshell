mod battery;
mod cpu_temp;
use iced::{
    Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    alignment::Vertical,
    padding, time,
    widget::{container, row, text},
};
use lucide_icons::Icon;
use sysinfo::{CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use tracing::info;

pub use battery::BatteryState;

use crate::{
    feature::Comp,
    system_info::battery::{self as bat},
    theme::CAT_THEME,
    widget::{
        align_center,
        container_ext::ContainExt,
        divider::{self, Angled, Semi},
        text_ext::TextExt,
    },
};

const BYTES_IN_GIG: u64 = 1_073_741_824;

#[derive(Debug, Clone)]
pub enum Message {
    OnCpuTempTick(f32),
    OnBatTick(bat::BatteryState),
    OnTick,
}

pub struct Init {
    pub bat_name: Option<String>,
}

pub struct SysInfoComp {
    disks: Disks,
    system: System,
    load: f64,
    cpu_temp: f32,
    bat_name: String,
    bat_stat: BatteryState,
}

impl SysInfoComp {
    pub fn disk_usage(&self) -> String {
        let disk = self
            .disks
            .iter()
            .find(|disk| disk.mount_point() == "/")
            .or(self.disks.first())
            .map(|disk| disk.available_space() / BYTES_IN_GIG)
            .unwrap_or_default();

        format!("{disk}G")
    }

    pub fn bat_stat(&self) -> &BatteryState {
        &self.bat_stat
    }
}

impl Comp for SysInfoComp {
    type Message = Message;
    type Init = Init;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        _f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        let disks =
            Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage());
        Self {
            disks,
            system,
            load: 0.,
            cpu_temp: 0.,
            bat_stat: BatteryState::None,
            bat_name: input.bat_name.unwrap_or_default(),
        }
        .to_tuple()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let avg_temp_sub = Subscription::run_with(cpu_temp::ListenData(1000), cpu_temp::listen)
            .filter_map(|res| match res {
                Ok(temp) => Some(Message::OnCpuTempTick(temp)),
                Err(err) => {
                    info!("Error getting temp {err:?}");
                    None
                }
            });

        let bat_stat = Subscription::run_with(
            bat::ListenData {
                delay: 1000,
                bat: self.bat_name.clone(),
            },
            bat::listen,
        )
        .filter_map(|res| match res {
            Ok(bat) => Some(Message::OnBatTick(bat)),
            Err(err) => {
                info!("Error getting bat state {err:?}");
                None
            }
        });

        let refresh_sub = time::every(time::Duration::from_millis(750)).map(|_| Message::OnTick);

        Subscription::batch([avg_temp_sub, refresh_sub, bat_stat])
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::OnCpuTempTick(cpu_temp) => {
                self.cpu_temp = cpu_temp;
                Task::none()
            }
            Message::OnBatTick(bat_state) => {
                self.bat_stat = bat_state;
                Task::none()
            }
            Message::OnTick => {
                self.system.refresh_memory();
                self.system.refresh_cpu_usage();
                self.load = System::load_average().one;
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;

        let cpu = {
            let temp_icon = Icon::Thermometer
                .widget()
                .center()
                .size(theme.spacing().md())
                .color(theme.base());

            let cpu_icon = Icon::Cpu
                .widget()
                .center()
                .size(theme.spacing().md())
                .color(theme.base());

            let temp = self.cpu_temp;
            let usage = self.system.global_cpu_usage();

            let temp_text = text!("{temp:.0} C").center().color(theme.base()).bold();
            let usage_text = text!("{usage:.0}%").center().color(theme.base()).bold();

            align_center!(
                row![cpu_icon, usage_text, temp_icon, temp_text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xs()),
            )
            .background(theme.mauve())
            .padding(padding::horizontal(theme.spacing().md()))
        };

        let avg_load = {
            let div = Angled::new(
                theme.peach(),
                theme.mauve(),
                divider::Direction::Left,
                divider::Heading::North,
                theme.spacing().xl(),
            );

            let icon = Icon::BicepsFlexed
                .widget()
                .center()
                .color(theme.base())
                .size(theme.spacing().md());

            let load = self.load;

            let text = text!("{load:.0}%").color(theme.base()).bold();

            let content = align_center!(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xs()),
            )
            .background(theme.peach())
            .padding(padding::horizontal(theme.spacing().md()));

            row![div, content].align_y(Vertical::Center)
        };

        let mem = {
            let div = Angled::new(
                theme.blue(),
                theme.peach(),
                divider::Direction::Left,
                divider::Heading::North,
                theme.spacing().xl(),
            );

            let icon = Icon::MemoryStick
                .widget()
                .size(theme.spacing().lg())
                .color(theme.base())
                .center();

            let tot_mem = self.system.total_memory() as f32;
            let avail_mem = self.system.available_memory() as f32;
            let mem = ((tot_mem - avail_mem) / tot_mem) * 100.0;

            let text = text!("{mem:.0}%").color(theme.base()).bold();

            let content = align_center!(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xxs()),
            )
            .background(theme.blue())
            .padding(padding::horizontal(theme.spacing().md()));

            row![div, content].align_y(Vertical::Center)
        };

        let disk_usage = {
            let div = Semi::new(
                theme.blue(),
                theme.trans(),
                divider::Direction::Right,
                theme.spacing().xl(),
            );

            let icon = Icon::HardDrive
                .widget()
                .size(theme.spacing().lg())
                .center()
                .color(theme.base());

            let text = self.disk_usage();
            let text = text!("{text}").color(theme.base()).bold();

            let main = align_center!(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xxs()),
            )
            .background(theme.trans())
            .padding(padding::horizontal(theme.spacing().lg()));

            align_center!(row![div, main])
        };

        container(row![cpu, avg_load, mem, disk_usage])
            .align_y(Vertical::Center)
            .into()
    }
}

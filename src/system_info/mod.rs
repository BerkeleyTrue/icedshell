mod cpu_temp;
use iced::{
    Subscription, Task,
    alignment::Vertical,
    padding, time,
    widget::{container, row, text},
};
use lucide_icons::Icon;
use sysinfo::{CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use tracing::{debug, info};

use crate::{
    divider::{self, Angled},
    feature::{Comp, align_center},
    fira_fonts::TextExt,
    theme::CAT_THEME,
    widget_ext::ContainExt,
};

const BYTES_IN_GIG: u64 = 1_073_741_824;

#[derive(Debug, Clone)]
pub enum Message {
    SystemLoad(f64),
    CpuTemp(f32),
}

pub struct SysInfoComp {
    disks: Disks,
    system: System,
    load: f64,
    cpu_temp: f32,
}

impl Comp for SysInfoComp {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
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
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let load_sub = time::every(time::Duration::from_millis(250))
            .map(|_| Message::SystemLoad(System::load_average().five));

        let avg_temp_sub = Subscription::run_with(cpu_temp::ListenData(1000), cpu_temp::listen)
            .filter_map(|res| match res {
                Ok(temp) => Some(Message::CpuTemp(temp)),
                Err(err) => {
                    info!("Error getting temp {err:?}");
                    None
                }
            });

        Subscription::batch([load_sub, avg_temp_sub])
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::SystemLoad(load) => {
                self.load = load;
                Task::none()
            }
            Message::CpuTemp(cpu_temp) => {
                self.cpu_temp = cpu_temp;
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let cpu = {
            let icon = Icon::BicepsFlexed.widget().center().color(theme.base());
            let load = self.load;

            let div = align_center!(Angled::new(
                theme.peach(),
                divider::Direction::Left,
                divider::Heading::North,
                theme.spacing().xl(),
            ))
            .background(theme.trans());

            let text = text!("{load:.0}%").color(theme.base()).bold();

            let content = container(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xxs()),
            )
            .background(theme.peach())
            .padding(padding::horizontal(theme.spacing().md()));

            row![div, content].align_y(Vertical::Center)
        };

        let temp = {
            let icon = Icon::Thermometer
                .widget()
                .center()
                .size(theme.spacing().md())
                .color(theme.base());
            let temp = self.cpu_temp;

            let div = align_center!(Angled::new(
                theme.mauve(),
                divider::Direction::Left,
                divider::Heading::North,
                theme.spacing().xl(),
            ))
            .background(theme.peach());

            let text = text!("{temp:.0}%").center().color(theme.base()).bold();

            let content = container(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xxs()),
            )
            .background(theme.mauve())
            .padding(padding::horizontal(theme.spacing().md()));

            row![div, content].align_y(Vertical::Center)
        };

        let mem = {
            let icon = Icon::MemoryStick
                .widget()
                .size(theme.spacing().lg())
                .color(theme.base())
                .center();
            let tot_mem = self.system.total_memory() as f32;
            let avail_mem = self.system.available_memory() as f32;
            let mem = ((tot_mem - avail_mem) / tot_mem) * 100.0;

            let div = align_center!(Angled::new(
                theme.blue(),
                divider::Direction::Left,
                divider::Heading::North,
                theme.spacing().xl(),
            ))
            .background(theme.mauve());

            let text = text!("{mem:.0}%").color(theme.base()).bold();

            let content = container(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xxs()),
            )
            .background(theme.blue())
            .padding(padding::horizontal(theme.spacing().md()));

            row![div, content].align_y(Vertical::Center)
        };

        let disk_usage = {
            let icon = Icon::HardDrive
                .widget()
                .size(theme.spacing().lg())
                .center()
                .color(theme.base());
            let disk = self
                .disks
                .iter()
                .find(|disk| disk.mount_point() == "/")
                .or(self.disks.first())
                .map(|disk| disk.available_space() / BYTES_IN_GIG)
                .unwrap_or_default();

            let text = text!("{disk}G").color(theme.base()).bold();

            let div = align_center!(Angled::new(
                theme.green(),
                divider::Direction::Left,
                divider::Heading::North,
                theme.spacing().xl(),
            ))
            .background(theme.blue());

            let main = container(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(theme.spacing().xxs()),
            )
            .background(theme.green())
            .padding(padding::horizontal(theme.spacing().xs()));

            align_center!(row![div, main])
        };

        container(row![cpu, temp, mem, disk_usage])
            .align_y(Vertical::Center)
            .into()
    }
}

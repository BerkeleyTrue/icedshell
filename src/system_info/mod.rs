use iced::{
    alignment::Vertical,
    padding,
    widget::{container, row, text},
};
use lucide_icons::Icon;
use sysinfo::{CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use tracing::info;

use crate::{
    divider::{self, Angled},
    feature::{Comp, align_center},
    fira_fonts::TextExt,
    theme::CAT_THEME,
    widget_ext::ContainExt,
};

const BYTES_IN_GIG: u64 = 1_073_741_824;

#[derive(Debug, Clone)]
pub enum Message {}

pub struct SysInfoComp {
    disks: Disks,
    system: System,
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
        Self { disks, system }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
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
            .background(theme.trans());

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

        container(row![mem, disk_usage])
            .align_y(Vertical::Center)
            .into()
    }
}

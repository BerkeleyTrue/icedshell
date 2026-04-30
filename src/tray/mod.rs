mod dbus;
mod eventstream;
pub mod menu_comp;
pub mod service;
pub mod tray_comp;
pub use dbus::{TrayLayout, TrayLayoutProps, TrayMenuItemId};

use iced::{Task, advanced::graphics::futures::MaybeSend};

pub trait TrayBar
where
    Self: 'static,
{
    type Message: MaybeSend + 'static;

    fn tray_serv_mut(&mut self) -> &mut service::TrayService;
    fn wrap_tray_msg(msg: service::Message) -> Self::Message;

    fn tray_menu_item_clicked(&mut self, name: String, id: TrayMenuItemId) -> Task<Self::Message> {
        self.tray_serv_mut()
            .menu_item_clicked(name, id)
            .map(Self::wrap_tray_msg)
    }
}

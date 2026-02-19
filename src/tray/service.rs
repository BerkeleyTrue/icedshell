use crate::{
    fdo_icons::FdIcon,
    feature::Service,
    tray::{
        dbus::Layout,
        eventstream::{SNItem, SNItemEvent, TrayEvent, listen},
    },
};
use iced::{Subscription, Task};
use std::ops::Deref;
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub enum Message {
    Registered(Box<SNItem>),
    IconChanged(String, FdIcon),
    MenuLayoutChanged(String, Layout),
    Unregistered(String),
    UpdateItems(TrayItems),
}

#[derive(Debug, Default, Clone)]
pub struct TrayItems(Vec<SNItem>);

#[derive(Debug, Clone)]
pub struct TrayService {
    pub items: TrayItems,
}

impl TrayService {
    pub fn menu_item_clicked(&mut self, id: i32, name: String) -> Task<Message> {
        debug!("Click on {name:} menu: {id}");
        self.items
            .iter()
            .find(|item| item.name == name)
            .map(|item| {
                let name_cb = name.clone();
                let item = item.clone();
                Task::future(async move { item.menu_item_clicked(id).await })
                    .map(|res| res.ok())
                    .and_then(move |layout| {
                        Task::done(Message::MenuLayoutChanged(name_cb.clone(), layout))
                    })
            })
            .unwrap_or_default()
    }
}

impl Service for TrayService {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
        Self {
            items: TrayItems(Vec::new()),
        }
    }

    fn update(&mut self, message: Self::Message) -> Task<Message> {
        match message {
            Message::Registered(new_item) => {
                match self
                    .items
                    .0
                    .iter_mut()
                    .find(|item| item.name == new_item.name)
                {
                    Some(existing_item) => {
                        *existing_item = *new_item;
                    }
                    None => {
                        self.items.0.push(*new_item);
                    }
                }
                Task::none()
            }
            Message::Unregistered(name) => {
                self.items.0.retain(|item| item.name != name);
                Task::none()
            }
            Message::IconChanged(name, handle) => {
                if let Some(item) = self.items.0.iter_mut().find(|item| item.name == name) {
                    item.icon = Some(handle);
                }
                Task::none()
            }
            Message::MenuLayoutChanged(name, layout) => {
                if let Some(item) = self.items.0.iter_mut().find(|item| item.name == name) {
                    debug!("menu layout updated, {layout:?}");
                    item.menu = layout;
                }
                Task::none()
            }
            Message::UpdateItems(items) => {
                self.items = items;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(listen).filter_map(|res| match res {
            Ok(tray_event) => Some(Message::from(tray_event)),
            Err(err) => {
                error!("Error from tray stream: {err:}");
                None
            }
        })
    }
}

impl Deref for TrayItems {
    type Target = Vec<SNItem>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for TrayService {
    type Target = TrayItems;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl From<SNItemEvent> for Message {
    fn from(sni_event: SNItemEvent) -> Self {
        match sni_event {
            SNItemEvent::IconChanged(id, handle) => Message::IconChanged(id, handle),
            SNItemEvent::MenuLayoutChanged(id, layout) => Message::MenuLayoutChanged(id, layout),
        }
    }
}

impl From<TrayEvent> for Message {
    fn from(tray_event: TrayEvent) -> Self {
        match tray_event {
            TrayEvent::ItemRegistered(item) => Message::Registered(item),
            TrayEvent::ItemUnregistered(name) => Message::Unregistered(name),
            TrayEvent::SNItem(sni_event) => (*sni_event).into(),
            TrayEvent::RegisteredItems(items) => Message::UpdateItems(TrayItems(items)),
        }
    }
}

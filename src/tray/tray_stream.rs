use iced::{
    futures::{
        Stream, StreamExt,
        stream::{self, BoxStream, select, select_all},
        stream_select,
    },
    widget::image,
};
use tracing::{debug, info, trace};

use crate::{
    fdo_icons::{self, FdIcon},
    tray::{
        TrayItems,
        dbus::{
            DBusMenuProxy, Layout, StatusNotifierItemProxy, StatusNotifierWatcher,
            StatusNotifierWatcherProxy, icons_to_fd_icon,
        },
    },
};

#[derive(Debug, Clone)]
pub struct SNItem {
    pub name: String,
    pub icon: Option<FdIcon>,
    pub menu: Layout,
    item_proxy: StatusNotifierItemProxy<'static>,
    menu_proxy: DBusMenuProxy<'static>,
}

impl SNItem {
    pub async fn new(conn: &zbus::Connection, name: String) -> anyhow::Result<Self> {
        let (dest, path) = if let Some(idx) = name.find('/') {
            (&name[..idx], &name[idx..])
        } else {
            (name.as_ref(), "/StatusNotifierItem")
        };

        let item_proxy = StatusNotifierItemProxy::builder(conn)
            .destination(dest.to_owned())?
            .path(path.to_owned())?
            .build()
            .await?;

        debug!("item_proxy {item_proxy:?}");

        let icon_pixmap = item_proxy.icon_pixmap().await;

        let icon = match icon_pixmap {
            Ok(icons) => icons_to_fd_icon(icons),
            Err(_) => item_proxy
                .icon_name()
                .await
                .ok()
                .as_deref()
                .and_then(fdo_icons::find),
        };

        let menu_path = item_proxy.menu().await?;
        let menu_proxy = DBusMenuProxy::builder(conn)
            .destination(dest.to_owned())?
            .path(menu_path.to_owned())?
            .build()
            .await?;

        let (_, menu) = menu_proxy.get_layout(0, -1, &[]).await?;

        Ok(Self {
            name,
            icon,
            menu,
            item_proxy,
            menu_proxy,
        })
    }
    pub async fn menu_item_clicked(&self, id: i32) -> anyhow::Result<Layout> {
        let value = zbus::zvariant::Value::I32(32).try_to_owned()?;

        self.menu_proxy
            .event(
                id,
                "clicked",
                &value,
                time::OffsetDateTime::now_utc().microsecond(),
            )
            .await?;

        let (_, layout) = self.menu_proxy.get_layout(0, -1, &[]).await?;

        Ok(layout)
    }
}

#[derive(Debug, Clone)]
pub enum SNItemEvent {
    Registered(SNItem),
    Unregistered(String),
    IconChanged(String, FdIcon),
    MenuLayoutChanged(String, Layout),
}

impl SNItemEvent {
    pub async fn eventstream(
        conn: &zbus::Connection,
    ) -> anyhow::Result<BoxStream<'static, SNItemEvent>> {
        let watcher_proxy = StatusNotifierWatcherProxy::new(conn).await?;

        let item_registered = watcher_proxy
            .receive_status_notifier_item_registered()
            .await?
            .filter_map({
                let conn = conn.clone();
                move |e| {
                    let conn = conn.clone();
                    async move {
                        debug!("registered {e:?}");
                        match e.args() {
                            Ok(args) => {
                                let item = SNItem::new(&conn, args.service.to_string()).await;

                                item.map(Self::Registered).ok()
                            }
                            _ => None,
                        }
                    }
                }
            })
            .boxed();

        let item_unregistered = watcher_proxy
            .receive_status_notifier_item_unregistered()
            .await?
            .filter_map(|e| async move {
                debug!("unregistered {e:?}");

                match e.args() {
                    Ok(args) => Some(Self::Unregistered(args.service.to_string())),
                    _ => None,
                }
            })
            .boxed();

        let curr_items = watcher_proxy.registered_status_notifier_items().await?;
        let mut icon_pixel_change = Vec::with_capacity(curr_items.len());
        let mut icon_name_change = Vec::with_capacity(curr_items.len());
        let mut menu_layout_change = Vec::with_capacity(curr_items.len());

        for name in curr_items {
            let item = SNItem::new(conn, name.to_string()).await?;

            icon_pixel_change.push(
                item.item_proxy
                    .receive_icon_pixmap_changed()
                    .await
                    .filter_map({
                        let name = name.clone();
                        move |icon_changed| {
                            let name = name.clone();
                            async move {
                                icon_changed
                                    .get()
                                    .await
                                    .ok()
                                    .and_then(icons_to_fd_icon)
                                    .map(|icon| SNItemEvent::IconChanged(name, icon))
                            }
                        }
                    })
                    .boxed(),
            );

            icon_name_change.push(
                item.item_proxy
                    .receive_icon_name_changed()
                    .await
                    .filter_map({
                        let name = name.clone();
                        move |icon_name| {
                            let name = name.clone();
                            async move {
                                icon_name
                                    .get()
                                    .await
                                    .ok()
                                    .as_deref()
                                    .and_then(fdo_icons::find)
                                    .map(|icon| SNItemEvent::IconChanged(name.to_owned(), icon))
                            }
                        }
                    })
                    .boxed(),
            );

            let layout_updated = item.menu_proxy.receive_layout_updated().await;
            if let Ok(layout_updated) = layout_updated {
                menu_layout_change.push(
                    layout_updated
                        .filter_map({
                            let name = name.clone();
                            let menu_proxy = item.menu_proxy.clone();
                            move |_| {
                                debug!("layout update event name {}", &name);

                                let name = name.clone();
                                let menu_proxy = menu_proxy.clone();
                                async move {
                                    menu_proxy.get_layout(0, -1, &[]).await.ok().map(
                                        |(_, layout)| {
                                            Self::MenuLayoutChanged(name.to_owned(), layout)
                                        },
                                    )
                                }
                            }
                        })
                        .boxed(),
                );
            }
        }

        Ok(stream_select!(
            item_registered,
            item_unregistered,
            select_all(icon_pixel_change),
            select_all(icon_name_change),
            select_all(menu_layout_change)
        )
        .boxed())
    }
}

pub async fn get_registered_items(conn: &zbus::Connection) -> anyhow::Result<Vec<SNItem>> {
    let watcher_proxy = StatusNotifierWatcherProxy::new(conn).await?;

    let cur_items = watcher_proxy.registered_status_notifier_items().await?;

    let mut status_items = Vec::with_capacity(cur_items.len());

    for item in cur_items {
        let item = SNItem::new(conn, item).await?;
        status_items.push(item);
    }

    Ok(status_items)
}

#[derive(Debug, Clone)]
pub enum TrayEvent {
    SNItem(Box<SNItemEvent>),
    RegisteredItems(Vec<SNItem>),
}

pub enum TrayStream {
    Init,
    Active(zbus::Connection, BoxStream<'static, SNItemEvent>),
    Error,
}

impl TrayStream {
    pub fn listen() -> impl Stream<Item = anyhow::Result<TrayEvent>> {
        let es = TrayStream::Init;
        stream::unfold(es, |es| async move {
            match es {
                TrayStream::Init => {
                    let conn = match StatusNotifierWatcher::start_server().await {
                        Ok(conn) => conn,
                        Err(err) => return Some((Err(err), TrayStream::Error)),
                    };

                    let items = match get_registered_items(&conn).await {
                        Ok(items) => TrayEvent::RegisteredItems(items),
                        Err(err) => return Some((Err(err), TrayStream::Error)),
                    };

                    let item_es = match SNItemEvent::eventstream(&conn).await {
                        Ok(item_es) => item_es,
                        Err(err) => return Some((Err(err), TrayStream::Error)),
                    };

                    Some((Ok(items), TrayStream::Active(conn, item_es)))
                }
                TrayStream::Active(conn, mut item_es) => {
                    match item_es.next().await {
                        Some(event) => {
                            let state = if matches!(event, SNItemEvent::Registered(_)) {
                                TrayStream::Init // drop stream and reconnect
                            } else {
                                TrayStream::Active(conn, item_es)
                            };
                            let tray_event = TrayEvent::SNItem(Box::new(event));
                            Some((Ok(tray_event), state))
                        }
                        None => None,
                    }
                }
                TrayStream::Error => None,
            }
        })
    }
}

enum RegistrationEvent {
    Registered(SNItem),
    Unregistered(String),
}

async fn get_registration_stream(
    conn: &zbus::Connection,
) -> anyhow::Result<BoxStream<'static, RegistrationEvent>> {
    let watcher_proxy = StatusNotifierWatcherProxy::new(conn).await?;

    let registered_stream = watcher_proxy
        .receive_status_notifier_item_registered()
        .await?
        .filter_map({
            let conn = conn.clone();
            move |e| {
                let conn = conn.clone();
                async move {
                    debug!("registered {e:?}");
                    match e.args() {
                        Ok(args) => {
                            let item = SNItem::new(&conn, args.service.to_string()).await;

                            item.map(RegistrationEvent::Registered).ok()
                        }
                        _ => None,
                    }
                }
            }
        })
        .boxed();

    let unregistered_stream = watcher_proxy
        .receive_status_notifier_item_unregistered()
        .await?
        .filter_map(|e| async move {
            debug!("unregistered {e:?}");

            match e.args() {
                Ok(args) => Some(RegistrationEvent::Unregistered(args.service.to_string())),
                _ => None,
            }
        })
        .boxed();

    Ok(select(registered_stream, unregistered_stream).boxed())
}

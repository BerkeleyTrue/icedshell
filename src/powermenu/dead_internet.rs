use iced::{
    Element, Subscription, Task,
    advanced::image,
    time,
    widget::{Image, stack},
};
use tokio::task;
use tracing::error;

use crate::widget::AtlasFrame;

const GIF_BYTES: &[u8] = include_bytes!("./resources/redlotoo_dead-internet-atlas.png");
const POSTER_BYTES: &[u8] = include_bytes!("./resources/redlotoo_dead-internet.png");
const FRAMES: u32 = 43;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    VideoLoaded(Option<image::Handle>),
}

pub struct DeadInternet {
    poster: image::Handle,
    handle: Option<image::Handle>,
    index: u32,
}

impl DeadInternet {
    async fn load_video() -> Message {
        let handle = task::spawn_blocking(|| image::Handle::from_bytes(GIF_BYTES))
            .await
            .inspect_err(|err| error!("load image err: {err:?}"))
            .ok();
        Message::VideoLoaded(handle)
    }

    pub fn new() -> (Self, Task<Message>) {
        let dead_internet = Self {
            poster: image::Handle::from_bytes(POSTER_BYTES),
            handle: None,
            index: 0,
        };
        (dead_internet, Task::future(DeadInternet::load_video()))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.index = (self.index + 1) % FRAMES;
            }
            Message::VideoLoaded(handle) => self.handle = handle,
        }
        Task::none()
    }

    pub fn subscriptions(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(67)).map(|_| Message::Tick)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut children = vec![Image::new(&self.poster).into()];

        if let Some(handle) = self.handle.as_ref() {
            children.push(AtlasFrame::new(handle, 4, 910.0, 512.0, self.index).into())
        }

        stack(children).into()
    }
}

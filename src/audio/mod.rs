mod stream;

use iced::{Subscription, Task};
use tracing::info;

use crate::{
    audio::stream::{AudioEvents, listen},
    feature::Service,
};

#[derive(Debug, Clone)]
pub enum Message {
    Audio(AudioEvents),
}

pub struct PulseAudio {
    vol: u32,
}

impl PulseAudio {
    pub fn get_vol(&self) -> u32 {
        self.vol
    }
}

impl Service for PulseAudio {
    type Message = Message;
    type Init = ();
    fn new<O: iced::advanced::graphics::futures::MaybeSend + 'static>(
        _input: Self::Init,
        _f: impl Fn(Self::Message) -> O + iced::advanced::graphics::futures::MaybeSend + 'static,
    ) -> (Self, iced::Task<O>) {
        (Self { vol: 0 }, Task::none())
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::run(listen)
            .map(|res| match res {
                Ok(audio) => Some(Message::Audio(audio)),
                Err(err) => {
                    info!("Error audio: {:?}", err);
                    None
                }
            })
            .filter_map(|x| x)
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Audio(audio) => match audio {
                AudioEvents::Vol(vol) => {
                    self.vol = vol;
                    Task::none()
                }
                AudioEvents::Connected | AudioEvents::None => Task::none(),
            },
        }
    }
}

use iced::{
    Color, Element, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    time::{every, seconds},
    widget::text,
};

use crate::feature::CompWithProps;

async fn run_cmd(cmd: String, args: Vec<String>) -> String {
    tokio::process::Command::new(&cmd)
        .args(&args)
        .output()
        .await
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub struct Init {
    pub cmd: String,
    pub args: Vec<String>,
    pub interval: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Output(String),
}

pub struct CmdComp {
    cmd: String,
    args: Vec<String>,
    interval: u64,
    output: String,
}

impl CompWithProps for CmdComp {
    type Message = Message;
    type Init = Init;
    type Props<'a> = Color;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let Init {
            cmd,
            args,
            interval,
        } = input;
        let comp = Self {
            cmd: cmd.clone(),
            args: args.clone(),
            interval,
            output: String::new(),
        };

        let task = Task::perform(run_cmd(cmd, args), move |out| f(Message::Output(out)));

        (comp, task)
    }

    fn subscription(&self) -> Subscription<Message> {
        every(seconds(self.interval)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let cmd = self.cmd.clone();
                let args = self.args.clone();
                Task::perform(run_cmd(cmd, args), Message::Output)
            }
            Message::Output(out) => {
                self.output = out;
                Task::none()
            }
        }
    }

    fn view(&self, color: Color) -> Element<'_, Message> {
        text(self.output.clone()).color(color).into()
    }
}

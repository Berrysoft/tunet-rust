#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use compio::runtime::spawn;
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::SettingsReader;
use winio::{
    App, Child, Color, Component, ComponentSender, Layoutable, Size, Visible, Window, WindowEvent,
};

fn main() {
    let cred = read_cred().unwrap_or_default();
    App::new().run::<MainModel>(cred)
}

fn accent_color() -> Color {
    let c = color_theme::Color::accent().unwrap_or(color_theme::Color {
        r: 0,
        g: 120,
        b: 212,
    });
    Color::new(c.r, c.g, c.b, 255)
}

fn read_cred() -> Option<(String, String)> {
    let reader = SettingsReader::new().ok()?;
    reader.read_full().ok()
}

enum MainMessage {
    Noop,
    Refresh,
    Close,
    Action(Action),
    Update(UpdateMsg),
}

struct MainModel {
    model: Model,
    window: Child<Window>,
}

impl Component for MainModel {
    type Init<'a> = (String, String);
    type Message = MainMessage;
    type Event = ();

    fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        let (action_sender, action_receiver) = flume::unbounded();
        let (update_sender, update_receiver) = flume::unbounded();
        let model = Model::new(action_sender, update_sender).unwrap();
        {
            let sender = sender.clone();
            spawn(async move {
                while let Ok(a) = action_receiver.recv_async().await {
                    sender.post(MainMessage::Action(a));
                }
            })
            .detach();
        }
        {
            let sender = sender.clone();
            spawn(async move {
                while let Ok(m) = update_receiver.recv_async().await {
                    sender.post(MainMessage::Update(m));
                }
            })
            .detach();
        }

        let (username, password) = init;
        if !username.is_empty() {
            model.queue(Action::Credential(username, password));
        }

        let mut window = Child::<Window>::init(());
        window.set_size(Size::new(600.0, 800.0));
        window.set_text("清华校园网");
        #[cfg(windows)]
        window.set_icon_by_id(1);

        window.show();

        Self { model, window }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(
            sender,
            |e| match e {
                WindowEvent::Close => Some(MainMessage::Close),
                WindowEvent::Resize | WindowEvent::Move => Some(MainMessage::Refresh),
                _ => None,
            },
            || MainMessage::Noop,
        );

        futures_util::join!(fut_window);
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MainMessage::Noop => false,
            MainMessage::Refresh => true,
            MainMessage::Close => {
                sender.output(());
                false
            }
            _ => true,
        }
    }

    fn render(&mut self, sender: &ComponentSender<Self>) {}
}

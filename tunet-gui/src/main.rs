#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use compio::runtime::spawn;
use tunet_helper::NetState;
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::SettingsReader;
use winio::{
    App, Button, ButtonEvent, Child, Color, ComboBox, ComboBoxEvent, Component, ComponentSender,
    Enable, Grid, HAlign, Label, Layoutable, Margin, Size, VAlign, Visible, Window, WindowEvent,
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
    ComboSelect,
    Action(Action),
    Update(UpdateMsg),
}

struct MainModel {
    model: Model,
    window: Child<Window>,
    state_combo: Child<ComboBox>,
    username: Child<Label>,
    flux: Child<Label>,
    online_time: Child<Label>,
    balance: Child<Label>,
    status: Child<Label>,
    log: Child<Label>,
    login_button: Child<Button>,
    logout_button: Child<Button>,
    refresh_button: Child<Button>,
    info1: Child<Label>,
    info2: Child<Label>,
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
        model.queue(Action::Status(None));
        model.queue(Action::Timer);

        let mut window = Child::<Window>::init(());
        window.set_size(Size::new(300.0, 500.0));
        window.set_text("清华校园网");
        #[cfg(windows)]
        window.set_icon_by_id(1);

        let mut state_combo = Child::<ComboBox>::init(&window);
        state_combo.insert(0, "Auth4");
        state_combo.insert(1, "Auth6");

        let mut username = Child::<Label>::init(&window);
        username.set_text("用户：");
        let mut flux = Child::<Label>::init(&window);
        flux.set_text("流量：");
        let mut online_time = Child::<Label>::init(&window);
        online_time.set_text("时长：");
        let mut balance = Child::<Label>::init(&window);
        balance.set_text("余额：");
        let mut status = Child::<Label>::init(&window);
        status.set_text("网络：");

        let mut log = Child::<Label>::init(&window);
        log.set_halign(HAlign::Center);

        let mut login_button = Child::<Button>::init(&window);
        login_button.set_text("登录");

        let mut logout_button = Child::<Button>::init(&window);
        logout_button.set_text("注销");

        let mut refresh_button = Child::<Button>::init(&window);
        refresh_button.set_text("刷新");

        let mut info1 = Child::<Label>::init(&window);
        info1.set_text("服务热线（8:00~20:00）010-62784859");
        info1.set_halign(HAlign::Center);
        let mut info2 = Child::<Label>::init(&window);
        info2.set_text(format!(
            "版本 {} 版权所有 © 2021-2025 Berrysoft",
            env!("CARGO_PKG_VERSION")
        ));
        info2.set_halign(HAlign::Center);

        window.show();

        Self {
            model,
            window,
            state_combo,
            username,
            flux,
            online_time,
            balance,
            status,
            log,
            login_button,
            logout_button,
            refresh_button,
            info1,
            info2,
        }
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
        let fut_combo = self.state_combo.start(
            sender,
            |e| match e {
                ComboBoxEvent::Select => Some(MainMessage::ComboSelect),
                _ => None,
            },
            || MainMessage::Noop,
        );
        let fut_login = self.login_button.start(
            sender,
            |e| match e {
                ButtonEvent::Click => Some(MainMessage::Action(Action::Login)),
                _ => None,
            },
            || MainMessage::Noop,
        );
        let fut_logout = self.logout_button.start(
            sender,
            |e| match e {
                ButtonEvent::Click => Some(MainMessage::Action(Action::Logout)),
                _ => None,
            },
            || MainMessage::Noop,
        );
        let fut_refresh = self.refresh_button.start(
            sender,
            |e| match e {
                ButtonEvent::Click => Some(MainMessage::Action(Action::Flux)),
                _ => None,
            },
            || MainMessage::Noop,
        );

        futures_util::join!(fut_window, fut_combo, fut_login, fut_logout, fut_refresh);
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MainMessage::Noop => false,
            MainMessage::Refresh => true,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::ComboSelect => {
                self.model
                    .queue(Action::State(self.state_combo.selection().and_then(
                        |i| match i {
                            0 => Some(NetState::Auth4),
                            1 => Some(NetState::Auth6),
                            _ => None,
                        },
                    )));
                false
            }
            MainMessage::Action(a) => {
                self.model.handle(a);
                false
            }
            MainMessage::Update(m) => {
                match m {
                    UpdateMsg::Credential => {
                        self.model.queue(Action::State(None));
                        //self.update_username(&model.username);
                    }
                    UpdateMsg::State => {
                        self.model.queue(Action::Flux);
                        let index = match self.model.state {
                            NetState::Unknown => None,
                            NetState::Auth4 => Some(0),
                            NetState::Auth6 => Some(1),
                        };
                        self.state_combo.set_selection(index);
                    }
                    UpdateMsg::Status => {
                        self.model.queue(Action::State(None));
                        self.status.set_text(format!("网络：{}", self.model.status));
                    }
                    UpdateMsg::Log => {
                        self.log.set_text(&self.model.log);
                    }
                    UpdateMsg::Flux => {
                        self.username
                            .set_text(format!("用户：{}", self.model.flux.username));
                        self.flux
                            .set_text(format!("流量：{}", self.model.flux.flux));
                        self.online_time
                            .set_text(format!("时长：{}", self.model.flux.online_time));
                        self.balance
                            .set_text(format!("余额：{}", self.model.flux.balance));
                    }
                    UpdateMsg::LogBusy => {
                        let busy = self.model.log_busy();
                        self.login_button.set_enabled(!busy);
                        self.logout_button.set_enabled(!busy);
                        self.refresh_button.set_enabled(!busy);
                    }
                }
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let margin = Margin::new_all_same(4.0);

        let csize = self.window.client_size();
        {
            let mut grid = Grid::from_str("1*,1*,1*", "auto,1*,auto,auto,auto,auto,auto").unwrap();
            grid.push(&mut self.state_combo)
                .column(0)
                .column_span(3)
                .row(0)
                .margin(margin)
                .finish();

            let mut flux_grid = Grid::from_str("1*", "1*,1*,1*,1*,1*").unwrap();
            flux_grid
                .push(&mut self.username)
                .column(0)
                .row(0)
                .margin(margin)
                .finish();
            flux_grid
                .push(&mut self.flux)
                .column(0)
                .row(1)
                .margin(margin)
                .finish();
            flux_grid
                .push(&mut self.online_time)
                .column(0)
                .row(2)
                .margin(margin)
                .finish();
            flux_grid
                .push(&mut self.balance)
                .column(0)
                .row(3)
                .margin(margin)
                .finish();
            flux_grid
                .push(&mut self.status)
                .column(0)
                .row(4)
                .margin(margin)
                .finish();

            grid.push(&mut flux_grid)
                .column(0)
                .column_span(3)
                .row(1)
                .halign(HAlign::Center)
                .valign(VAlign::Center)
                .finish();

            grid.push(&mut self.log)
                .column(0)
                .column_span(3)
                .row(2)
                .margin(margin)
                .finish();

            grid.push(&mut self.login_button)
                .column(0)
                .row(3)
                .margin(margin)
                .finish();
            grid.push(&mut self.logout_button)
                .column(1)
                .row(3)
                .margin(margin)
                .finish();
            grid.push(&mut self.refresh_button)
                .column(2)
                .row(3)
                .margin(margin)
                .finish();

            grid.push(&mut self.info1)
                .column(0)
                .column_span(3)
                .row(5)
                .margin(margin)
                .finish();
            grid.push(&mut self.info2)
                .column(0)
                .column_span(3)
                .row(6)
                .margin(margin)
                .finish();

            grid.set_size(csize);
        }
    }
}

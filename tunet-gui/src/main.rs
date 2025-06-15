#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use compio::runtime::spawn;
use tunet_helper::NetState;
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::SettingsReader;
use winio::{
    App, BrushPen, Button, ButtonEvent, Canvas, Child, Color, ComboBox, ComboBoxEvent, Component,
    ComponentSender, Edit, Enable, Grid, HAlign, Label, Layoutable, Margin, Monitor, Point, Rect,
    Size, SolidColorBrush, VAlign, Visible, Window, WindowEvent,
};

fn main() {
    let mut reader = SettingsReader::new().unwrap();
    let cred = reader.read_full().unwrap_or_default();
    let save = App::new_with_name("io.github.berrysoft.tunet").run::<MainModel>(cred);
    if let MainEvent::Delete(u) = save {
        reader.delete(&u).unwrap();
    }
}

fn accent_color() -> Color {
    let c = color_theme::Color::accent().unwrap_or(color_theme::Color {
        r: 0,
        g: 120,
        b: 212,
    });
    Color::new(c.r, c.g, c.b, 255)
}

enum MainMessage {
    Noop,
    Refresh,
    Close(bool),
    ComboSelect,
    Cred,
    Action(Action),
    Update(UpdateMsg),
}

enum MainEvent {
    Save,
    Delete(String),
}

struct MainModel {
    model: Model,
    window: Child<Window>,
    state_combo: Child<ComboBox>,
    canvas: Child<Canvas>,
    username: Child<Label>,
    flux: Child<Label>,
    online_time: Child<Label>,
    balance: Child<Label>,
    status: Child<Label>,
    log: Child<Label>,
    login_button: Child<Button>,
    logout_button: Child<Button>,
    refresh_button: Child<Button>,
    username_label: Child<Label>,
    password_label: Child<Label>,
    username_input: Child<Edit>,
    password_input: Child<Edit>,
    cred_button: Child<Button>,
    del_button: Child<Button>,
    info1: Child<Label>,
    info2: Child<Label>,
}

impl Component for MainModel {
    type Init<'a> = (String, String);
    type Message = MainMessage;
    type Event = MainEvent;

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
        {
            let monitors = Monitor::all();
            let region = monitors[0].client_scaled();
            window.set_loc(region.origin + region.size / 2.0 - window.size() / 2.0);
        }

        let mut state_combo = Child::<ComboBox>::init(&window);
        state_combo.insert(0, "Auth4");
        state_combo.insert(1, "Auth6");

        let canvas = Child::<Canvas>::init(&window);

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

        let mut username_label = Child::<Label>::init(&window);
        username_label.set_text("用户：");
        let mut password_label = Child::<Label>::init(&window);
        password_label.set_text("密码：");

        let username_input = Child::<Edit>::init(&window);
        let mut password_input = Child::<Edit>::init(&window);
        password_input.set_password(true);

        let mut cred_button = Child::<Button>::init(&window);
        cred_button.set_text("更新凭据");

        let mut del_button = Child::<Button>::init(&window);
        del_button.set_text("删除并退出");

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
            canvas,
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
            username_label,
            password_label,
            username_input,
            password_input,
            cred_button,
            del_button,
            info1,
            info2,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(
            sender,
            |e| match e {
                WindowEvent::Close => Some(MainMessage::Close(true)),
                WindowEvent::Resize => Some(MainMessage::Refresh),
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
        let fut_cred = self.cred_button.start(
            sender,
            |e| match e {
                ButtonEvent::Click => Some(MainMessage::Cred),
                _ => None,
            },
            || MainMessage::Noop,
        );
        let fut_del = self.del_button.start(
            sender,
            |e| match e {
                ButtonEvent::Click => Some(MainMessage::Close(false)),
                _ => None,
            },
            || MainMessage::Noop,
        );

        futures_util::join!(
            fut_window,
            fut_combo,
            fut_login,
            fut_logout,
            fut_refresh,
            fut_cred,
            fut_del
        );
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MainMessage::Noop => false,
            MainMessage::Refresh => true,
            MainMessage::Close(save) => {
                sender.output(if save {
                    MainEvent::Save
                } else {
                    MainEvent::Delete(self.username_input.text())
                });
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
            MainMessage::Cred => {
                self.model.queue(Action::Credential(
                    self.username_input.text(),
                    self.password_input.text(),
                ));
                false
            }
            MainMessage::Action(a) => {
                self.model.handle(a);
                false
            }
            MainMessage::Update(m) => match m {
                UpdateMsg::Credential => {
                    self.model.queue(Action::State(None));
                    self.username_input.set_text(&self.model.username);
                    false
                }
                UpdateMsg::State => {
                    self.model.queue(Action::Flux);
                    let index = match self.model.state {
                        NetState::Unknown => None,
                        NetState::Auth4 => Some(0),
                        NetState::Auth6 => Some(1),
                    };
                    self.state_combo.set_selection(index);
                    false
                }
                UpdateMsg::Status => {
                    self.model.queue(Action::State(None));
                    self.status.set_text(format!("网络：{}", self.model.status));
                    false
                }
                UpdateMsg::Log => {
                    self.log.set_text(&self.model.log);
                    false
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
                    true
                }
                UpdateMsg::LogBusy => {
                    let busy = self.model.log_busy();
                    self.login_button.set_enabled(!busy);
                    self.logout_button.set_enabled(!busy);
                    self.refresh_button.set_enabled(!busy);
                    false
                }
            },
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

            grid.push(&mut self.canvas)
                .column(0)
                .column_span(3)
                .row(1)
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

            let mut cred_grid = Grid::from_str("auto,1*,auto", "1*,1*").unwrap();
            let mut label_margin = margin;
            label_margin.right = 0.0;
            let mut input_margin = margin;
            input_margin.left = 0.0;
            cred_grid
                .push(&mut self.username_label)
                .column(0)
                .row(0)
                .valign(VAlign::Center)
                .margin(label_margin)
                .finish();
            cred_grid
                .push(&mut self.password_label)
                .column(0)
                .row(1)
                .valign(VAlign::Center)
                .margin(label_margin)
                .finish();
            cred_grid
                .push(&mut self.username_input)
                .column(1)
                .row(0)
                .margin(input_margin)
                .finish();
            cred_grid
                .push(&mut self.password_input)
                .column(1)
                .row(1)
                .margin(input_margin)
                .finish();
            cred_grid
                .push(&mut self.cred_button)
                .column(2)
                .row(0)
                .margin(margin)
                .finish();
            cred_grid
                .push(&mut self.del_button)
                .column(2)
                .row(1)
                .margin(margin)
                .finish();

            grid.push(&mut cred_grid)
                .column(0)
                .column_span(3)
                .row(4)
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

        const ARC_WIDTH: f64 = 30.0;
        use std::f64::consts::*;

        let size = self.canvas.size();
        let (width, height) = (size.width - ARC_WIDTH, size.height - ARC_WIDTH);
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let color = accent_color();
        let pen = BrushPen::new(SolidColorBrush::new(color), ARC_WIDTH);
        let color_t1 = color.with_alpha(168);
        let pen_t1 = BrushPen::new(SolidColorBrush::new(color_t1), ARC_WIDTH);
        let color_t2 = color.with_alpha(84);
        let pen_t2 = BrushPen::new(SolidColorBrush::new(color_t2), ARC_WIDTH);

        let arc_rect = if width > height {
            Rect::new(
                Point::new((width - height + ARC_WIDTH) / 2.0, ARC_WIDTH / 2.0),
                Size::new(height, height),
            )
        } else {
            Rect::new(
                Point::new(ARC_WIDTH / 2.0, (height - width + ARC_WIDTH) / 2.0),
                Size::new(width, width),
            )
        };
        let mut ctx = self.canvas.context();
        let flux = &self.model.flux;
        let flux_gb = flux.flux.to_gb();
        ctx.draw_arc(pen_t2, arc_rect, FRAC_PI_2, PI * 2.0 + FRAC_PI_2 - 0.001);
        let free_angle =
            0.0f64.max(50.0 / (flux.balance.0 + 50.0f64.max(flux_gb)) * 2.0 * PI - 0.001);
        ctx.draw_arc(pen_t1, arc_rect, FRAC_PI_2, FRAC_PI_2 + free_angle);
        let flux_angle =
            0.0f64.max(flux_gb / (flux.balance.0 + 50.0f64.max(flux_gb)) * 2.0 * PI - 0.001);
        ctx.draw_arc(pen, arc_rect, FRAC_PI_2, FRAC_PI_2 + flux_angle);
    }
}

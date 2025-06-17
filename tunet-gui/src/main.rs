#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use compio::runtime::spawn;
use tunet_helper::NetState;
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::SettingsReader;
use winio::{
    init, start, App, BrushPen, Button, ButtonEvent, Canvas, Child, Color, ComboBox, ComboBoxEvent,
    Component, ComponentSender, Edit, Enable, Grid, HAlign, Label, Layoutable, Margin, MessageBox,
    MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Monitor, Point, Rect, Size,
    SolidColorBrush, VAlign, Visible, Window, WindowEvent,
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
    Close,
    ComboSelect,
    Cred,
    Del,
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
    hidden: Child<Label>,
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

        init! {
            window: Window = (()) => {
                size: Size::new(300.0, 500.0),
                text: "清华校园网",
                loc: {
                    let monitors = Monitor::all();
                    let region = monitors[0].client_scaled();
                    region.origin + region.size / 2.0 - window.size() / 2.0
                }
            },
            state_combo: ComboBox = (&window),
            canvas: Canvas = (&window),
            hidden: Label = (&window) => { text: "HIDDEN", visible: false },
            username: Label = (&window) => { text: "用户：" },
            flux: Label = (&window) => { text: "流量：" },
            online_time: Label = (&window) => { text: "时长：" },
            balance: Label = (&window) => { text: "余额：" },
            status: Label = (&window) => { text: "网络：" },
            log: Label = (&window) => { halign: HAlign::Center },
            login_button: Button = (&window) => { text: "登录" },
            logout_button: Button = (&window) => { text: "注销" },
            refresh_button: Button = (&window) => { text: "刷新" },
            username_label: Label = (&window) => { text: "用户：" },
            password_label: Label = (&window) => { text: "密码：" },
            username_input: Edit = (&window),
            password_input: Edit = (&window) => { password: true },
            cred_button: Button = (&window) => { text: "更新凭据" },
            del_button: Button = (&window) => { text: "删除并退出" },
            info1: Label = (&window) => {
                text: "服务热线（8:00~20:00）010-62784859",
                halign: HAlign::Center,
            },
            info2: Label = (&window) => {
                text: format!(
                    "版本 {} 版权所有 © 2021-2025 Berrysoft",
                    env!("CARGO_PKG_VERSION")
                ),
                halign: HAlign::Center,
            }
        }

        #[cfg(windows)]
        window.set_icon_by_id(1);

        state_combo.insert(0, "Auth4");
        state_combo.insert(1, "Auth6");

        window.show();

        Self {
            model,
            window,
            canvas,
            state_combo,
            hidden,
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
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Refresh,
            },
            self.state_combo => {
                ComboBoxEvent::Select => MainMessage::ComboSelect,
            },
            self.login_button => {
                ButtonEvent::Click => MainMessage::Action(Action::Login),
            },
            self.logout_button => {
                ButtonEvent::Click => MainMessage::Action(Action::Logout),
            },
            self.refresh_button => {
                ButtonEvent::Click => MainMessage::Action(Action::Flux),
            },
            self.cred_button => {
                ButtonEvent::Click => MainMessage::Cred,
            },
            self.del_button => {
                ButtonEvent::Click => MainMessage::Del,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MainMessage::Noop => false,
            MainMessage::Refresh => true,
            MainMessage::Close => {
                sender.output(MainEvent::Save);
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
            MainMessage::Del => {
                let res = MessageBox::new()
                    .title("删除并退出")
                    .message("将删除保存的凭据并退出应用程序")
                    .buttons(MessageBoxButton::Ok | MessageBoxButton::Cancel)
                    .style(MessageBoxStyle::Warning)
                    .show(Some(&self.window))
                    .await;
                if let MessageBoxResponse::Ok = res {
                    sender.output(MainEvent::Delete(self.username_input.text()));
                }
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
                    true
                }
                UpdateMsg::Log => {
                    self.log.set_text(&self.model.log);
                    true
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
            let mut grid =
                Grid::from_str("1*,1*,1*", "auto,auto,1*,auto,auto,auto,auto,auto").unwrap();
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
                .row_span(3)
                .margin(margin)
                .finish();

            grid.push(&mut self.hidden)
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
                .row(2)
                .halign(HAlign::Center)
                .valign(VAlign::Center)
                .finish();

            grid.push(&mut self.log)
                .column(0)
                .column_span(3)
                .row(3)
                .margin(margin)
                .finish();

            grid.push(&mut self.login_button)
                .column(0)
                .row(4)
                .margin(margin)
                .finish();
            grid.push(&mut self.logout_button)
                .column(1)
                .row(4)
                .margin(margin)
                .finish();
            grid.push(&mut self.refresh_button)
                .column(2)
                .row(4)
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
                .row(5)
                .finish();

            grid.push(&mut self.info1)
                .column(0)
                .column_span(3)
                .row(6)
                .margin(margin)
                .finish();
            grid.push(&mut self.info2)
                .column(0)
                .column_span(3)
                .row(7)
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

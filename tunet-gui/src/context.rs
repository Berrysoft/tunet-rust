use crate::{accent_color, App, DetailModel, HomeModel, NetInfo, SettingsModel};
use anyhow::Result;
use mac_address2::MacAddress;
use plotters::{
    prelude::{ChartBuilder, IntoDrawingArea, RangedDate, SVGBackend},
    series::LineSeries,
    style::{Color as PlotColor, FontFamily, IntoTextStyle, RGBColor, ShapeStyle},
};
use slint::{ComponentHandle, Image, ModelRc, SharedString, StandardListViewItem, VecModel};
use std::{cmp::Reverse, rc::Rc, sync::Arc, sync::Mutex as SyncMutex};
use tokio::sync::{
    mpsc::{self, channel},
    Mutex,
};
use tunet_helper::{
    usereg::{NetDetail, NetUser},
    Datelike, Flux, NetFlux, NetState,
};
use tunet_model::{Action, DetailDaily, Model, UpdateMsg};

#[derive(Clone)]
pub struct UpdateContext {
    weak_app: slint::Weak<App>,
    data: Arc<SyncMutex<UpdateData>>,
}

impl UpdateContext {
    pub fn new(app: &App) -> Self {
        Self {
            weak_app: app.as_weak(),
            data: Arc::new(SyncMutex::new(UpdateData::default())),
        }
    }

    pub fn weak_app(&self) -> &slint::Weak<App> {
        &self.weak_app
    }

    pub async fn create_model(
        &self,
        action_sender: mpsc::Sender<Action>,
    ) -> Result<Arc<Mutex<Model>>> {
        let (update_sender, mut update_receiver) = channel(32);
        let model = Arc::new(Mutex::new(Model::new(action_sender, update_sender)?));
        {
            let model = model.clone();
            let context = self.clone();
            tokio::spawn(async move {
                while let Some(msg) = update_receiver.recv().await {
                    let model = model.lock().await;
                    context.update(&model, msg)
                }
            });
        }
        Ok(model)
    }

    fn update_username(&self, username: impl Into<SharedString>) {
        let username = username.into();
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<SettingsModel>().set_username(username);
            })
            .unwrap();
    }

    fn update_state(&self, state: NetState) {
        let state = state as i32 - 1;
        self.weak_app
            .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_state(state))
            .unwrap();
    }

    fn update_status(&self, status: impl Into<SharedString>) {
        let status = status.into();
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_status(status);
            })
            .unwrap();
    }

    fn update_log(&self, log: impl Into<SharedString>) {
        let log = log.into();
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_log(log);
            })
            .unwrap();
    }

    fn update_info(&self, flux: &NetFlux) {
        let info = NetInfo {
            username: flux.username.as_str().into(),
            flux_gb: flux.flux.to_gb() as _,
            flux_str: flux.flux.to_string().into(),
            online_time: flux.online_time.to_string().into(),
            balance: flux.balance.0 as _,
            balance_str: flux.balance.to_string().into(),
        };
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_info(info);
            })
            .unwrap();
    }

    fn update_online(&self, onlines: &[NetUser], mac_addrs: &[MacAddress]) {
        let onlines = onlines.to_vec();
        let is_local = onlines
            .iter()
            .map(|user| {
                mac_addrs
                    .iter()
                    .any(|it| Some(it) == user.mac_address.as_ref())
            })
            .collect::<Vec<_>>();
        self.weak_app
            .upgrade_in_event_loop(move |app| update_online(app, onlines, is_local))
            .unwrap();
    }

    fn update_details(&self, details: &[NetDetail]) {
        self.data.lock().unwrap().update_details(details.to_vec());
        self.weak_app
            .upgrade_in_event_loop({
                let data = self.data.clone();
                move |app| {
                    let data = data.lock().unwrap();
                    update_details(&app, &data.sorted_details);
                }
            })
            .unwrap();
    }

    pub fn sort_details(&self, column: i32, descending: bool) {
        self.weak_app
            .upgrade_in_event_loop({
                let data = self.data.clone();
                move |app| {
                    let mut data = data.lock().unwrap();
                    data.sort(column, descending);
                    update_details(&app, &data.sorted_details);
                }
            })
            .unwrap();
    }

    fn update_log_busy(&self, busy: bool) {
        self.weak_app
            .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_busy(busy))
            .unwrap();
    }

    fn update_online_busy(&self, busy: bool) {
        self.weak_app
            .upgrade_in_event_loop(move |app| app.global::<SettingsModel>().set_busy(busy))
            .unwrap();
    }

    fn update_detail_busy(&self, busy: bool) {
        self.weak_app
            .upgrade_in_event_loop(move |app| app.global::<DetailModel>().set_busy(busy))
            .unwrap();
    }

    pub fn update(&self, model: &Model, msg: UpdateMsg) {
        match msg {
            UpdateMsg::Credential => {
                model.queue(Action::State(None));
                model.queue(Action::Online);
                model.queue(Action::Details);
                self.update_username(&model.username);
            }
            UpdateMsg::State => {
                model.queue(Action::Flux);
                self.update_state(model.state);
            }
            UpdateMsg::Status => {
                model.queue(Action::State(None));
                self.update_status(model.status.to_string());
            }
            UpdateMsg::Log => {
                self.update_log(model.log.as_ref());
            }
            UpdateMsg::Flux => {
                self.update_info(&model.flux);
            }
            UpdateMsg::Online => {
                self.update_online(&model.users, &model.mac_addrs);
            }
            UpdateMsg::Details => {
                self.update_details(&model.details);
            }
            UpdateMsg::LogBusy => {
                self.update_log_busy(model.log_busy());
            }
            UpdateMsg::OnlineBusy => {
                self.update_online_busy(model.online_busy());
            }
            UpdateMsg::DetailBusy => {
                self.update_detail_busy(model.detail_busy());
            }
        };
    }

    pub fn set_del_at_exit(&self) {
        self.data.lock().unwrap().del_at_exit = true;
    }

    pub fn del_at_exit(&self) -> bool {
        self.data.lock().unwrap().del_at_exit
    }

    pub fn draw_daily_chart(&self, width: f32, height: f32, text_color: slint::Color) -> Image {
        let app = self.weak_app.upgrade().unwrap();
        let data = self.data.lock().unwrap();
        draw_daily(&app, width, height, text_color, &data.daily).unwrap()
    }
}

#[derive(Debug, Default)]
struct UpdateData {
    del_at_exit: bool,
    sorted_details: Vec<NetDetail>,
    daily: DetailDaily,
}

impl UpdateData {
    pub fn update_details(&mut self, details: Vec<NetDetail>) {
        self.daily = DetailDaily::new(&details);
        self.sorted_details = details;
    }

    pub fn sort(&mut self, column: i32, descending: bool) {
        if descending {
            match column {
                0 => self.sorted_details.sort_by_key(|d| Reverse(d.login_time)),
                1 => self.sorted_details.sort_by_key(|d| Reverse(d.logout_time)),
                2 => self.sorted_details.sort_by_key(|d| Reverse(d.flux)),
                _ => unreachable!(),
            }
        } else {
            match column {
                0 => self.sorted_details.sort_by_key(|d| d.login_time),
                1 => self.sorted_details.sort_by_key(|d| d.logout_time),
                2 => self.sorted_details.sort_by_key(|d| d.flux),
                _ => unreachable!(),
            }
        }
    }
}

fn update_online(app: App, onlines: Vec<NetUser>, is_local: Vec<bool>) {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for (user, is_local) in onlines.into_iter().zip(is_local) {
        let items: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::default());
        items.push(user.address.to_string().as_str().into());
        items.push(user.address_v6.to_string().as_str().into());
        items.push(user.login_time.to_string().as_str().into());
        items.push(user.flux.to_string().as_str().into());
        items.push(
            user.mac_address
                .map(|addr| addr.to_string())
                .unwrap_or_default()
                .as_str()
                .into(),
        );
        items.push(if is_local { "本机" } else { "未知" }.into());
        row_data.push(items.into());
    }
    app.global::<SettingsModel>().set_onlines(row_data.into());
}

fn update_details(app: &App, details: &[NetDetail]) {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for d in details {
        let items: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::default());
        items.push(d.login_time.to_string().as_str().into());
        items.push(d.logout_time.to_string().as_str().into());
        items.push(d.flux.to_string().as_str().into());
        row_data.push(items.into());
    }
    app.global::<DetailModel>().set_details(row_data.into());
}

fn draw_daily(
    app: &App,
    width: f32,
    height: f32,
    text_color: slint::Color,
    details: &DetailDaily,
) -> Result<Image> {
    let color = accent_color();
    let color = RGBColor(color.r, color.g, color.b);
    let scale = app.window().scale_factor();
    let (width, height) = ((width * scale) as u32, (height * scale) as u32);
    let text_color = RGBColor(text_color.red(), text_color.green(), text_color.blue());

    let date_range = (details.now.with_day(1).unwrap(), details.now);
    let flux_range = (0, details.max_flux.0);

    let mut buffer = String::new();
    let backend = SVGBackend::with_string(&mut buffer, (width, height));
    {
        let root = backend.into_drawing_area();

        let label_style = (FontFamily::SansSerif, 20.0 * scale)
            .with_color(text_color)
            .into_text_style(&root);

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35.0 * scale)
            .y_label_area_size(75.0 * scale)
            .margin_top(10.0 * scale)
            .margin_right(20.0 * scale)
            .build_cartesian_2d(
                RangedDate::from(date_range.0..date_range.1),
                flux_range.0..flux_range.1,
            )?;
        chart
            .configure_mesh()
            .disable_mesh()
            .axis_style(ShapeStyle {
                color: text_color.to_rgba(),
                filled: false,
                stroke_width: scale as _,
            })
            .x_label_style(label_style.clone())
            .x_label_formatter(&|d| d.format("%m-%d").to_string())
            .y_label_style(label_style)
            .y_label_formatter(&|f| Flux(*f).to_string())
            .y_labels(5)
            .draw()?;
        chart.draw_series(
            LineSeries::new(
                details.details.iter().map(|(d, f)| (*d, f.0)),
                ShapeStyle {
                    color: color.to_rgba(),
                    filled: true,
                    stroke_width: (scale * 2.0) as _,
                },
            )
            .point_size((scale * 3.0) as _),
        )?;

        root.present()?;
    }

    Image::load_from_svg_data(buffer.as_bytes()).map_err(|_| anyhow::anyhow!("Cannot load SVG."))
}

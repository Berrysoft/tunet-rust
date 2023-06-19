use anyhow::Result;
use chrono::Datelike;
use flutter_rust_bridge::{frb, RustOpaque, StreamSink};

pub use netstatus::NetStatus;
pub use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};
pub use tokio::{runtime::Handle, sync::mpsc};
pub use tunet_helper::{
    usereg::{NetDateTime, NetDetail, NetUser},
    Balance, Duration as NewDuration, Flux, NaiveDate, NaiveDateTime, NaiveDuration as Duration,
    NetFlux, NetState,
};
pub use tunet_model::{Action, DetailDaily, Model, UpdateMsg};

#[frb(mirror(UpdateMsg))]
pub enum _UpdateMsg {
    Credential,
    State,
    Status,
    Log,
    Flux,
    Online,
    Details,
    LogBusy,
    OnlineBusy,
    DetailBusy,
}

pub struct UpdateMsgWrap(pub UpdateMsg);

#[frb(mirror(NetState))]
pub enum _NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
}

pub struct NetStateWrap(pub NetState);

#[frb(mirror(NetStatus))]
pub enum _NetStatus {
    Unknown,
    Wwan,
    Wlan(String),
    Lan,
}

pub struct NetStatusWrap(pub NetStatus);

#[frb(mirror(NetFlux))]
pub struct _NetFlux {
    pub username: String,
    pub flux: Flux,
    pub online_time: NewDuration,
    pub balance: Balance,
}

#[frb(mirror(Flux))]
pub struct _Flux(pub u64);

pub fn flux_to_string(f: u64) -> String {
    Flux(f).to_string()
}

#[frb(mirror(NewDuration))]
pub struct _NewDuration(pub Duration);

#[frb(mirror(Balance))]
pub struct _Balance(pub f64);

#[frb(mirror(NetDateTime))]
pub struct _NetDateTime(pub NaiveDateTime);

#[frb(mirror(NetDetail))]
pub struct _NetDetail {
    pub login_time: NetDateTime,
    pub logout_time: NetDateTime,
    pub flux: Flux,
}

pub struct DetailDailyPoint {
    pub day: u32,
    pub flux: Flux,
}

pub struct DetailDailyWrap {
    pub details: Vec<DetailDailyPoint>,
    pub now_month: u32,
    pub now_day: u32,
    pub max_flux: Flux,
}

pub struct NetUserWrap {
    pub address: Ipv4AddrWrap,
    pub login_time: NetDateTime,
    pub mac_address: String,
    pub flux: Flux,
    pub is_local: bool,
}

pub struct Ipv4AddrWrap {
    pub octets: [u8; 4],
}

impl From<Ipv4Addr> for Ipv4AddrWrap {
    fn from(value: Ipv4Addr) -> Self {
        Self {
            octets: value.octets(),
        }
    }
}

pub struct RuntimeStartConfig {
    pub status: NetStatusWrap,
    pub username: String,
    pub password: String,
}

pub struct Runtime {
    pub rx: RustOpaque<Mutex<Option<mpsc::Receiver<Action>>>>,
    pub model: RustOpaque<Mutex<Model>>,
    pub handle: RustOpaque<Mutex<Option<Handle>>>,
}

impl Runtime {
    pub fn new() -> Result<Runtime> {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Trace)
                .with_filter(
                    android_logger::FilterBuilder::new()
                        .parse("warn,tunet=trace,native=trace")
                        .build(),
                ),
        );
        #[cfg(target_os = "ios")]
        oslog::OsLogger::new("com.berrysoft.tunet_flutter")
            .level_filter(log::LevelFilter::Trace)
            .init()?;

        let (tx, rx) = mpsc::channel(32);
        let model = Model::new(tx)?;
        Ok(Self {
            rx: RustOpaque::new(Mutex::new(Some(rx))),
            model: RustOpaque::new(Mutex::new(model)),
            handle: RustOpaque::new(Mutex::new(None)),
        })
    }

    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>, config: RuntimeStartConfig) {
        let model = self.model.clone();
        let mut rx = self.rx.lock().unwrap().take().unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        *self.handle.lock().unwrap() = Some(handle);
        std::thread::spawn(move || {
            runtime.block_on(async {
                {
                    let mut model = model.lock().unwrap();
                    model.update = Some(Box::new(move |msg| {
                        sink.add(UpdateMsgWrap(msg));
                    }));

                    if (!config.username.is_empty()) && (!config.password.is_empty()) {
                        model.queue(Action::Credential(config.username, config.password));
                    }
                    model.queue(Action::Status(Some(config.status.0)));
                    model.queue(Action::Timer);
                }
                while let Some(action) = rx.recv().await {
                    log::debug!("received action: {:?}", action);
                    model.lock().unwrap().handle(action);
                }
            });
        });
    }

    fn queue(&self, a: Action) {
        let _guard = self.handle.lock().unwrap().as_ref().unwrap().enter();
        self.model.lock().unwrap().queue(a);
    }

    pub fn queue_credential(&self, u: String, p: String) {
        self.queue(Action::Credential(u, p));
    }

    pub fn queue_login(&self) {
        self.queue(Action::Login);
    }

    pub fn queue_logout(&self) {
        self.queue(Action::Logout);
    }

    pub fn queue_flux(&self) {
        self.queue(Action::Flux);
    }

    pub fn queue_state(&self, s: Option<NetStateWrap>) {
        self.queue(Action::State(s.map(|s| s.0)));
    }

    pub fn queue_details(&self) {
        self.queue(Action::Details);
    }

    pub fn queue_onlines(&self) {
        self.queue(Action::Online);
    }

    pub fn queue_connect(&self, ip: Ipv4AddrWrap) {
        self.queue(Action::Connect(Ipv4Addr::from(ip.octets)));
    }

    pub fn queue_drop(&self, ips: Vec<Ipv4AddrWrap>) {
        for ip in ips {
            self.queue(Action::Drop(Ipv4Addr::from(ip.octets)));
        }
    }

    pub fn log_busy(&self) -> bool {
        self.model.lock().unwrap().log_busy()
    }

    pub fn log_text(&self) -> String {
        self.model.lock().unwrap().log.clone().into_owned()
    }

    pub fn flux(&self) -> NetFlux {
        self.model.lock().unwrap().flux.clone()
    }

    pub fn state(&self) -> NetStateWrap {
        NetStateWrap(self.model.lock().unwrap().state)
    }

    pub fn status(&self) -> String {
        self.model.lock().unwrap().status.to_string()
    }

    pub fn detail_busy(&self) -> bool {
        self.model.lock().unwrap().detail_busy()
    }

    pub fn details(&self) -> Vec<NetDetail> {
        self.model.lock().unwrap().details.clone()
    }

    pub fn detail_daily(&self) -> Option<DetailDailyWrap> {
        let data = {
            let model = self.model.lock().unwrap();
            if model.details.is_empty() {
                None
            } else {
                Some(DetailDaily::new(&model.details))
            }
        };
        data.map(|data| DetailDailyWrap {
            details: data
                .details
                .into_iter()
                .map(|(date, flux)| DetailDailyPoint {
                    day: date.day(),
                    flux,
                })
                .collect(),
            now_month: data.now.month(),
            now_day: data.now.day(),
            max_flux: data.max_flux,
        })
    }

    pub fn username(&self) -> String {
        self.model.lock().unwrap().username.clone()
    }

    pub fn online_busy(&self) -> bool {
        self.model.lock().unwrap().online_busy()
    }

    pub fn onlines(&self) -> Vec<NetUserWrap> {
        let model = self.model.lock().unwrap();
        model
            .users
            .iter()
            .map(|u| NetUserWrap {
                address: u.address.into(),
                login_time: u.login_time,
                mac_address: u
                    .mac_address
                    .map(|addr| addr.to_string())
                    .unwrap_or_default(),
                flux: u.flux,
                is_local: model
                    .mac_addrs
                    .iter()
                    .any(|it| Some(it) == u.mac_address.as_ref()),
            })
            .collect()
    }
}

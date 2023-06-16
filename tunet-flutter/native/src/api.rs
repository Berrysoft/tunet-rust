use anyhow::Result;
use chrono::Datelike;
use flutter_rust_bridge::{frb, RustOpaque, StreamSink};

pub use netstatus::NetStatus;
pub use std::sync::{Arc, Mutex};
pub use tokio::{runtime::Handle, sync::mpsc};
pub use tunet_helper::{
    Balance, Duration as NewDuration, Flux, NaiveDate, NaiveDuration as Duration, NetFlux, NetState,
};
use tunet_model::DetailDaily;
pub use tunet_model::{Action, Model, UpdateMsg};

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

pub enum NetStatusSimp {
    Unknown,
    Wwan,
    Wlan,
    Lan,
}

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

pub struct Runtime {
    pub rx: RustOpaque<Mutex<Option<mpsc::Receiver<Action>>>>,
    pub model: RustOpaque<Mutex<Model>>,
    pub handle: RustOpaque<Mutex<Option<Handle>>>,
    pub init_status: RustOpaque<Mutex<NetStatus>>,
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
            init_status: RustOpaque::new(Mutex::new(NetStatus::Unknown)),
        })
    }

    pub fn initialize_status(&self, t: NetStatusSimp, ssid: Option<String>) {
        let status = match t {
            NetStatusSimp::Unknown => NetStatus::Unknown,
            NetStatusSimp::Wwan => NetStatus::Wwan,
            NetStatusSimp::Wlan => match ssid {
                Some(s) => NetStatus::Wlan(s),
                None => NetStatus::Unknown,
            },
            NetStatusSimp::Lan => NetStatus::Lan,
        };
        *self.init_status.lock().unwrap() = status;
    }

    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>) {
        let model = self.model.clone();
        let mut rx = self.rx.lock().unwrap().take().unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        *self.handle.lock().unwrap() = Some(handle);
        let status = self.init_status.lock().unwrap().clone();
        std::thread::spawn(move || {
            runtime.block_on(async {
                {
                    let mut model = model.lock().unwrap();
                    model.update = Some(Box::new(move |msg| {
                        sink.add(UpdateMsgWrap(msg));
                    }));

                    model.queue(Action::Status(Some(status)));
                    model.queue(Action::Timer);
                }
                while let Some(action) = rx.recv().await {
                    log::info!("received action: {:?}", action);
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

    pub fn log_busy(&self) -> bool {
        self.model.lock().unwrap().log_busy()
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
}

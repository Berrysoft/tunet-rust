use jni::{errors::Result, objects::JObject, refs::Global};

use crate::*;

jni::bind_java_type! {
    Context => android.content.Context,
    methods {
        fn get_system_service(name: JString) -> JObject,
    },
    fields {
        #[allow(non_snake_case)]
        static CONNECTIVITY_SERVICE: JString,
        #[allow(non_snake_case)]
        static WIFI_SERVICE: JString,
    }
}

jni::bind_java_type! {
    ConnectivityManager => android.net.ConnectivityManager,
    type_map {
        Network => android.net.Network,
        NetworkInfo => android.net.NetworkInfo,
    },
    methods {
        fn get_all_networks() -> Network[],
        fn get_network_info(network: Network) -> NetworkInfo,
    }
}

const TYPE_MOBILE: i32 = 0;
const TYPE_WIFI: i32 = 1;
const TYPE_ETHERNET: i32 = 9;

jni::bind_java_type! {
    Network => android.net.Network,
}

jni::bind_java_type! {
    NetworkInfo => android.net.NetworkInfo,
    methods {
        fn get_type() -> jint,
    }
}

jni::bind_java_type! {
    WifiManager => android.net.wifi.WifiManager,
    type_map {
        WifiInfo => android.net.wifi.WifiInfo,
    },
    methods {
        fn get_connection_info() -> WifiInfo,
    }
}

jni::bind_java_type! {
    WifiInfo => android.net.wifi.WifiInfo,
    methods {
        fn get_s_s_i_d() -> JString,
    }
}

pub fn current() -> NetStatus {
    let context = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) };
    vm.attach_current_thread(|env| {
        let ctx = context.context().cast();
        let ctx = unsafe { env.as_cast_raw::<Global<JObject>>(&ctx)? };
        let ctx = env.new_local_ref(ctx)?;
        let ctx = unsafe { Context::from_raw(env, ctx.into_raw()) };
        let name = Context::CONNECTIVITY_SERVICE(env)?;
        let manager = ctx.get_system_service(env, name)?;
        let manager = env.cast_local::<ConnectivityManager>(manager)?;
        let networks = manager.get_all_networks(env)?;
        let networks_len = networks.len(env)?;
        let mut types = vec![];
        for i in 0..networks_len {
            let network = networks.get_element(env, i)?;
            let info = manager.get_network_info(env, network)?;
            if !info.is_null() {
                let ty = info.get_type(env)?;
                types.push(ty);
            }
        }
        let status = if types.contains(&TYPE_WIFI) {
            let name = Context::WIFI_SERVICE(env)?;
            let manager = ctx.get_system_service(env, name)?;
            let manager = env.cast_local::<WifiManager>(manager)?;
            let info = manager.get_connection_info(env)?;
            let ssid = info.get_s_s_i_d(env)?.try_to_string(env)?;
            if ssid.is_empty() || ssid == "<unknown ssid>" {
                NetStatus::Unknown
            } else {
                NetStatus::Wlan(ssid.trim_matches('\"').to_string())
            }
        } else if types.contains(&TYPE_MOBILE) {
            NetStatus::Wwan
        } else if types.contains(&TYPE_ETHERNET) {
            NetStatus::Lan
        } else {
            NetStatus::Unknown
        };
        Result::Ok(status)
    })
    .unwrap_or(NetStatus::Unknown)
}

pub fn watch() -> impl Stream<Item = ()> {
    futures_util::stream::pending()
}

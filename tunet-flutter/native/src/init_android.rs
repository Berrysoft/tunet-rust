use jni::{
    EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::{JClass, JObject},
};

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_berrysoft_tunet_1flutter_InitPlugin_init_1android<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    context: JObject<'caller>,
) {
    unowned_env
        .with_env(|env| rustls_platform_verifier::android::init_with_env(env, context))
        .resolve::<ThrowRuntimeExAndDefault>();
}

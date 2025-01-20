use jni::{
    objects::{JClass, JObject},
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_io_github_berrysoft_tunet_1flutter_InitPlugin_init_1android(
    mut env: JNIEnv,
    _class: JClass,
    context: JObject,
) {
    rustls_platform_verifier::android::init_hosted(&mut env, context).ok();
}

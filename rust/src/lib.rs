mod engine;
mod config;
mod logger;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jint, jstring};
use once_cell::sync::{Lazy};
use std::sync::Mutex;
use engine::AdEngine;
use logger::SkipLogger;

static ENGINE: Lazy<Mutex<AdEngine>> = Lazy::new(|| Mutex::new(AdEngine::new()));
static LOG: Lazy<Mutex<SkipLogger>> = Lazy::new(|| Mutex::new(SkipLogger::new()));

#[no_mangle] pub extern "system" fn Java_com_adskipper_MainService_nativeOnServiceConnected(_: JNIEnv, _: JClass) {}
#[no_mangle] pub extern "system" fn Java_com_adskipper_MainService_nativeHandleEvent(mut env: JNIEnv, _: JClass, root_node: JObject, package_name: JString) {
    let pkg: String = env.get_string(&package_name).map(|s| s.into()).unwrap_or_default();
    let mut e = ENGINE.lock().unwrap();
    match e.process(&mut env, &root_node, &pkg) {
        Ok(Some(kw)) => { if let Ok(mut l) = LOG.lock() { l.log(&pkg, &kw, true, "clicked"); } }
        Ok(None) => {}
        Err(err) => log::error!("{}", err),
    }
}
#[no_mangle] pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetConfig(mut env: JNIEnv, _: JClass, ctx: JObject) -> jstring {
    let c = config::load(&mut env, &ctx).unwrap_or_default();
    let j = serde_json::to_string(&c).unwrap_or_default();
    env.new_string(&j).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}
#[no_mangle] pub extern "system" fn Java_com_adskipper_MainActivity_nativeSetEnabled(mut env: JNIEnv, _: JClass, ctx: JObject, en: jni::sys::jboolean) {
    let mut c = config::load(&mut env, &ctx).unwrap_or_default();
    c.enabled = en != 0;
    let _ = config::save(&mut env, &ctx, &c);
    if let Ok(mut e) = ENGINE.lock() { e.update_config(c); }
}
#[no_mangle] pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetTodayCount(_: JNIEnv, _: JClass) -> jint { LOG.lock().map(|l| l.today_count() as jint).unwrap_or(0) }
#[no_mangle] pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetRecentLogs(mut env: JNIEnv, _: JClass, max: jint) -> jstring {
    let t = LOG.lock().map(|l| {
        let e = l.recent(max as usize);
        if e.is_empty() { "No logs".into() }
        else { e.iter().rev().map(|x| format!("[{}] {} | {}", x.timestamp, x.package, x.keyword_matched)).collect::<Vec<_>>().join("\n") }
    }).unwrap_or_default();
    env.new_string(&t).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jint, jstring};

mod engine;
mod config;
mod logger;

// Minimal JNI stubs - will be filled after basic compile succeeds
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainService_nativeOnServiceConnected(_e: JNIEnv, _c: JClass) {}
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainService_nativeHandleEvent(_e: JNIEnv, _c: JClass, _r: JObject, _p: JString) {}
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetConfig(_e: JNIEnv, _c: JClass, _ctx: JObject) -> jstring { std::ptr::null_mut() }
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeSetEnabled(_e: JNIEnv, _c: JClass, _ctx: JObject, _en: jni::sys::jboolean) {}
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetTodayCount(_e: JNIEnv, _c: JClass) -> jint { 0 }
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetRecentLogs(mut e: JNIEnv, _c: JClass, _m: jint) -> jstring {
    e.new_string("").map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

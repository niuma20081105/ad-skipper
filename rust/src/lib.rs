mod engine;
mod config;
mod logger;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jint, jstring};
use log::info;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

use engine::AdEngine;
use config::AppConfig;
use logger::SkipLogger;

static ENGINE: OnceCell<Mutex<AdEngine>> = OnceCell::new();
static LOGGER: OnceCell<Mutex<SkipLogger>> = OnceCell::new();

fn get_engine() -> &Mutex<AdEngine> {
    ENGINE.get_or_init(|| Mutex::new(AdEngine::new()))
}

fn get_logger() -> &Mutex<SkipLogger> {
    LOGGER.get_or_init(|| Mutex::new(SkipLogger::new()))
}

// ─── AccessibilityService JNI ──────────────────────────────────

#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainService_nativeOnServiceConnected(
    _env: JNIEnv,
    _class: JClass,
) {
    info!("AccessibilityService connected");
}

#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainService_nativeHandleEvent(
    mut env: JNIEnv,
    _class: JClass,
    root_node: JObject,
    package_name: JString,
) {
    let pkg: String = match env.get_string(&package_name) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get package name: {e}");
            return;
        }
    };

    let engine = get_engine();
    let mut engine = match engine.lock() {
        Ok(e) => e,
        Err(e) => {
            log::error!("Mutex poisoned: {e}");
            return;
        }
    };

    match engine.process(&mut env, root_node, &pkg) {
        Ok(Some(keyword)) => {
            // 记录跳过日志
            if let Ok(mut logger) = get_logger().lock() {
                logger.log(&pkg, &keyword, true, "clicked");
            }
        }
        Ok(None) => {} // 没找到跳过按钮
        Err(e) => log::error!("Engine process error: {e}"),
    }
}

// ─── MainActivity JNI ─────────────────────────────────────────

/// 返回配置 JSON 字符串
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetConfig(
    mut env: JNIEnv,
    _class: JClass,
    context: JObject,
) -> jstring {
    let config = config::load_config(&mut env, context).unwrap_or_default();
    let json = serde_json::to_string(&config).unwrap_or_default();
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// 设置启用/禁用状态
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeSetEnabled(
    mut env: JNIEnv,
    _class: JClass,
    context: JObject,
    enabled: jni::sys::jboolean,
) {
    let mut config = config::load_config(&mut env, context).unwrap_or_default();
    config.enabled = enabled != 0;
    let _ = config::save_config(&mut env, context, &config);

    // 同步更新引擎
    if let Ok(mut engine) = get_engine().lock() {
        engine.update_config(config);
    }
}

/// 返回今日跳过次数
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetTodayCount(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    get_logger()
        .lock()
        .map(|l| l.today_count() as jint)
        .unwrap_or(0)
}

/// 返回最近 N 条日志的格式化文本
#[no_mangle]
pub extern "system" fn Java_com_adskipper_MainActivity_nativeGetRecentLogs(
    mut env: JNIEnv,
    _class: JClass,
    max: jint,
) -> jstring {
    let text = match get_logger().lock() {
        Ok(logger) => {
            let entries = logger.recent(max as usize);
            if entries.is_empty() {
                "暂无跳过记录".to_string()
            } else {
                entries
                    .iter()
                    .rev()
                    .map(|e| {
                        format!(
                            "[{}] {} — 匹配: {} | 结果: {}",
                            e.timestamp, e.package, e.keyword_matched, e.result
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        Err(_) => "日志不可用".to_string(),
    };

    env.new_string(text)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

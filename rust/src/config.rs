use jni::JNIEnv;
use jni::objects::{JObject, JString};
use serde::{Deserialize, Serialize};

const PREFS_NAME: &str = "ad_skipper_prefs";
const KEY_ENABLED: &str = "enabled";
const KEY_KEYWORDS: &str = "keywords";
const KEY_DELAY_MS: &str = "delay_ms";
const KEY_WHITELIST: &str = "whitelist";
const KEY_BLACKLIST: &str = "blacklist";
const KEY_LOG_ENABLED: &str = "log_enabled";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_keywords")]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub auto_click_delay_ms: u64,
    #[serde(default)]
    pub app_whitelist: Vec<String>,
    #[serde(default)]
    pub app_blacklist: Vec<String>,
    #[serde(default = "default_true")]
    pub log_enabled: bool,
}

fn default_enabled() -> bool { true }
fn default_true() -> bool { true }
fn default_keywords() -> Vec<String> { Vec::new() }

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            enabled: true,
            keywords: Vec::new(),
            auto_click_delay_ms: 0,
            app_whitelist: Vec::new(),
            app_blacklist: Vec::new(),
            log_enabled: true,
        }
    }
}

pub fn load_config(env: &mut JNIEnv, context: JObject) -> Result<AppConfig, String> {
    let prefs = open_prefs(env, context)?;
    Ok(AppConfig {
        enabled: pref_get_bool(env, prefs, KEY_ENABLED, true)?,
        keywords: parse_csv(&pref_get_string(env, prefs, KEY_KEYWORDS, "")?),
        auto_click_delay_ms: pref_get_long(env, prefs, KEY_DELAY_MS, 0)? as u64,
        app_whitelist: parse_csv(&pref_get_string(env, prefs, KEY_WHITELIST, "")?),
        app_blacklist: parse_csv(&pref_get_string(env, prefs, KEY_BLACKLIST, "")?),
        log_enabled: pref_get_bool(env, prefs, KEY_LOG_ENABLED, true)?,
    })
}

pub fn save_config(
    env: &mut JNIEnv,
    context: JObject,
    config: &AppConfig,
) -> Result<(), String> {
    let prefs = open_prefs(env, context)?;
    let editor = open_editor(env, prefs)?;
    put_bool(env, editor, KEY_ENABLED, config.enabled)?;
    put_string(env, editor, KEY_KEYWORDS, &config.keywords.join(","))?;
    put_long(env, editor, KEY_DELAY_MS, config.auto_click_delay_ms as i64)?;
    put_string(env, editor, KEY_WHITELIST, &config.app_whitelist.join(","))?;
    put_string(env, editor, KEY_BLACKLIST, &config.app_blacklist.join(","))?;
    put_bool(env, editor, KEY_LOG_ENABLED, config.log_enabled)?;
    apply_editor(env, editor)?;
    Ok(())
}

// ─── JNI helpers — always use the raw JObject from .l() immediately ─────

fn open_prefs(env: &mut JNIEnv, context: JObject) -> Result<JObject, String> {
    let mode: i32 = 0;
    let key = env.new_string(PREFS_NAME).map_err(|e| format!("new_string: {e}"))?;
    let result = env
        .call_method(
            context,
            "getSharedPreferences",
            "(Ljava/lang/String;I)Landroid/content/SharedPreferences;",
            &[key.into(), mode.into()],
        )
        .map_err(|e| format!("getSharedPreferences failed: {e}"))?;
    result.l().map_err(|e| format!("getSharedPreferences l(): {e}"))
}

fn open_editor(env: &mut JNIEnv, prefs: JObject) -> Result<JObject, String> {
    let result = env
        .call_method(prefs, "edit", "()Landroid/content/SharedPreferences$Editor;", &[])
        .map_err(|e| format!("edit() failed: {e}"))?;
    result.l().map_err(|e| format!("edit l(): {e}"))
}

fn pref_get_bool(env: &mut JNIEnv, prefs: JObject, key: &str, default: bool) -> Result<bool, String> {
    let key_j = env.new_string(key).map_err(|e| format!("new_string: {e}"))?;
    let val = env
        .call_method(prefs, "getBoolean", "(Ljava/lang/String;Z)Z", &[key_j.into(), default.into()])
        .map_err(|e| format!("getBoolean({key}) failed: {e}"))?;
    Ok(val.z().unwrap_or(default))
}

fn pref_get_string(env: &mut JNIEnv, prefs: JObject, key: &str, default: &str) -> Result<String, String> {
    let key_j = env.new_string(key).map_err(|e| format!("new_string: {e}"))?;
    let def_j = env.new_string(default).map_err(|e| format!("new_string: {e}"))?;
    let val = env
        .call_method(prefs, "getString", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;", &[key_j.into(), def_j.into()])
        .map_err(|e| format!("getString({key}) failed: {e}"))?;
    let jobj = val.l().map_err(|e| format!("getString l(): {e}"))?;
    if jobj.is_null() {
        return Ok(default.to_string());
    }
    let jstr = JString::from(jobj);
    let s: String = env.get_string(&jstr).map_err(|e| format!("get_string: {e}"))?.into();
    Ok(s)
}

fn pref_get_long(env: &mut JNIEnv, prefs: JObject, key: &str, default: i64) -> Result<i64, String> {
    let key_j = env.new_string(key).map_err(|e| format!("new_string: {e}"))?;
    let val = env
        .call_method(prefs, "getLong", "(Ljava/lang/String;J)J", &[key_j.into(), default.into()])
        .map_err(|e| format!("getLong({key}) failed: {e}"))?;
    Ok(val.j().unwrap_or(default))
}

fn put_bool(env: &mut JNIEnv, editor: JObject, key: &str, value: bool) -> Result<(), String> {
    let key_j = env.new_string(key).map_err(|e| format!("new_string: {e}"))?;
    env.call_method(editor, "putBoolean", "(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;", &[key_j.into(), value.into()])
        .map_err(|e| format!("putBoolean({key}) failed: {e}"))?;
    Ok(())
}

fn put_string(env: &mut JNIEnv, editor: JObject, key: &str, value: &str) -> Result<(), String> {
    let key_j = env.new_string(key).map_err(|e| format!("new_string: {e}"))?;
    let val_j = env.new_string(value).map_err(|e| format!("new_string: {e}"))?;
    env.call_method(editor, "putString", "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;", &[key_j.into(), val_j.into()])
        .map_err(|e| format!("putString({key}) failed: {e}"))?;
    Ok(())
}

fn put_long(env: &mut JNIEnv, editor: JObject, key: &str, value: i64) -> Result<(), String> {
    let key_j = env.new_string(key).map_err(|e| format!("new_string: {e}"))?;
    env.call_method(editor, "putLong", "(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;", &[key_j.into(), value.into()])
        .map_err(|e| format!("putLong({key}) failed: {e}"))?;
    Ok(())
}

fn apply_editor(env: &mut JNIEnv, editor: JObject) -> Result<(), String> {
    env.call_method(editor, "apply", "()V", &[])
        .map_err(|e| format!("apply() failed: {e}"))?;
    Ok(())
}

fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

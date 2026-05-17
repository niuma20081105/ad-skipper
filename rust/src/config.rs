use jni::JNIEnv;
use jni::objects::JObject;
use serde::{Deserialize, Serialize};

const PREFS_NAME: &str = "ad_skipper_prefs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_enabled")] pub enabled: bool,
    #[serde(default = "default_keywords")] pub keywords: Vec<String>,
    #[serde(default)] pub auto_click_delay_ms: u64,
    #[serde(default)] pub app_whitelist: Vec<String>,
    #[serde(default)] pub app_blacklist: Vec<String>,
    #[serde(default = "default_true")] pub log_enabled: bool,
}

fn default_enabled() -> bool { true }
fn default_true() -> bool { true }
fn default_keywords() -> Vec<String> { Vec::new() }

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig { enabled: true, keywords: Vec::new(), auto_click_delay_ms: 0,
            app_whitelist: Vec::new(), app_blacklist: Vec::new(), log_enabled: true }
    }
}

pub fn load_config(env: &mut JNIEnv, context: &JObject) -> Result<AppConfig, String> {
    let prefs = open_prefs(env, context)?;
    Ok(AppConfig {
        enabled: pref_bool(env, &prefs, "enabled", true)?,
        keywords: csv(&pref_str(env, &prefs, "keywords", "")?),
        auto_click_delay_ms: pref_long(env, &prefs, "delay_ms", 0)? as u64,
        app_whitelist: csv(&pref_str(env, &prefs, "whitelist", "")?),
        app_blacklist: csv(&pref_str(env, &prefs, "blacklist", "")?),
        log_enabled: pref_bool(env, &prefs, "log_enabled", true)?,
    })
}

pub fn save_config(env: &mut JNIEnv, context: &JObject, config: &AppConfig) -> Result<(), String> {
    let prefs = open_prefs(env, context)?;
    let editor = edit(env, &prefs)?;
    put_bool(env, &editor, "enabled", config.enabled)?;
    put_str(env, &editor, "keywords", &config.keywords.join(","))?;
    put_long(env, &editor, "delay_ms", config.auto_click_delay_ms as i64)?;
    put_str(env, &editor, "whitelist", &config.app_whitelist.join(","))?;
    put_str(env, &editor, "blacklist", &config.app_blacklist.join(","))?;
    put_bool(env, &editor, "log_enabled", config.log_enabled)?;
    env.call_method(&editor, "apply", "()V", &[]).map_err(|e| format!("apply: {e}"))?;
    Ok(())
}

fn open_prefs<'a>(env: &'a mut JNIEnv, ctx: &JObject) -> Result<JObject<'a>, String> {
    let key = env.new_string(PREFS_NAME).map_err(|e| format!("new_string: {e}"))?;
    env.call_method(ctx, "getSharedPreferences", "(Ljava/lang/String;I)Landroid/content/SharedPreferences;", &[key.into(), 0i32.into()])
        .map_err(|e| format!("getSharedPreferences: {e}"))?
        .l().map_err(|e| format!("l: {e}"))
}

fn edit<'a>(env: &'a mut JNIEnv, prefs: &JObject) -> Result<JObject<'a>, String> {
    env.call_method(prefs, "edit", "()Landroid/content/SharedPreferences$Editor;", &[])
        .map_err(|e| format!("edit: {e}"))?
        .l().map_err(|e| format!("l: {e}"))
}

fn pref_bool(env: &mut JNIEnv, prefs: &JObject, k: &str, d: bool) -> Result<bool, String> {
    let key = env.new_string(k).map_err(|e| format!("ns: {e}"))?;
    env.call_method(prefs, "getBoolean", "(Ljava/lang/String;Z)Z", &[key.into(), d.into()])
        .map_err(|e| format!("getBoolean: {e}"))
        .map(|v| v.z().unwrap_or(d))
}

fn pref_str(env: &mut JNIEnv, prefs: &JObject, k: &str, d: &str) -> Result<String, String> {
    let key = env.new_string(k).map_err(|e| format!("ns: {e}"))?;
    let def = env.new_string(d).map_err(|e| format!("ns: {e}"))?;
    let val = env.call_method(prefs, "getString", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;", &[key.into(), def.into()])
        .map_err(|e| format!("getString: {e}"))?;
    let obj = val.l().map_err(|e| format!("l: {e}"))?;
    if obj.is_null() { return Ok(d.to_string()); }
    unsafe {
        let jstr = jni::objects::JString::from_raw(obj.into_raw());
        Ok(env.get_string(&jstr).map_err(|e| format!("gs: {e}"))?.into())
    }
}

fn pref_long(env: &mut JNIEnv, prefs: &JObject, k: &str, d: i64) -> Result<i64, String> {
    let key = env.new_string(k).map_err(|e| format!("ns: {e}"))?;
    env.call_method(prefs, "getLong", "(Ljava/lang/String;J)J", &[key.into(), d.into()])
        .map_err(|e| format!("getLong: {e}"))
        .map(|v| v.j().unwrap_or(d))
}

fn put_bool(env: &mut JNIEnv, ed: &JObject, k: &str, v: bool) -> Result<(), String> {
    let key = env.new_string(k).map_err(|e| format!("ns: {e}"))?;
    env.call_method(ed, "putBoolean", "(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;", &[key.into(), v.into()])
        .map_err(|e| format!("putBool: {e}"))?;
    Ok(())
}

fn put_str(env: &mut JNIEnv, ed: &JObject, k: &str, v: &str) -> Result<(), String> {
    let key = env.new_string(k).map_err(|e| format!("ns: {e}"))?;
    let val = env.new_string(v).map_err(|e| format!("ns: {e}"))?;
    env.call_method(ed, "putString", "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;", &[key.into(), val.into()])
        .map_err(|e| format!("putStr: {e}"))?;
    Ok(())
}

fn put_long(env: &mut JNIEnv, ed: &JObject, k: &str, v: i64) -> Result<(), String> {
    let key = env.new_string(k).map_err(|e| format!("ns: {e}"))?;
    env.call_method(ed, "putLong", "(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;", &[key.into(), v.into()])
        .map_err(|e| format!("putLong: {e}"))?;
    Ok(())
}

fn csv(s: &str) -> Vec<String> {
    s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

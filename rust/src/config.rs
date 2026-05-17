use jni::JNIEnv;
use jni::objects::{JObject, JString};
use serde::{Deserialize, Serialize};

const PREFS: &str = "ad_skipper_prefs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default="yes")] pub enabled: bool,
    #[serde(default="empty")] pub keywords: Vec<String>,
    #[serde(default)] pub auto_click_delay_ms: u64,
    #[serde(default="empty")] pub app_whitelist: Vec<String>,
    #[serde(default="empty")] pub app_blacklist: Vec<String>,
    #[serde(default="yes")] pub log_enabled: bool,
}
fn yes() -> bool { true }
fn empty() -> Vec<String> { vec![] }
impl Default for AppConfig {
    fn default() -> Self { AppConfig { enabled:true, keywords:vec![], auto_click_delay_ms:0, app_whitelist:vec![], app_blacklist:vec![], log_enabled:true } }
}

pub fn load(env: &mut JNIEnv, ctx: &JObject) -> Result<AppConfig, String> {
    let p = prefs(env, ctx)?;
    Ok(AppConfig {
        enabled: gb(env, &p, "enabled", true)?,
        keywords: csv(&gs(env, &p, "keywords", "")?),
        auto_click_delay_ms: gl(env, &p, "delay_ms", 0)? as u64,
        app_whitelist: csv(&gs(env, &p, "whitelist", "")?),
        app_blacklist: csv(&gs(env, &p, "blacklist", "")?),
        log_enabled: gb(env, &p, "log_enabled", true)?,
    })
}

pub fn save(env: &mut JNIEnv, ctx: &JObject, c: &AppConfig) -> Result<(), String> {
    let p = prefs(env, ctx)?;
    let e = ed(env, &p)?;
    sb(env, &e, "enabled", c.enabled)?;
    ss(env, &e, "keywords", &c.keywords.join(","))?;
    sl(env, &e, "delay_ms", c.auto_click_delay_ms as i64)?;
    ss(env, &e, "whitelist", &c.app_whitelist.join(","))?;
    ss(env, &e, "blacklist", &c.app_blacklist.join(","))?;
    sb(env, &e, "log_enabled", c.log_enabled)?;
    env.call_method(&e, "apply", "()V", &[]).map_err(|e| format!("ap:{}",e))?;
    Ok(())
}

fn prefs<'a>(env: &'a mut JNIEnv, ctx: &JObject) -> Result<JObject<'a>, String> {
    let k = env.new_string(PREFS).map_err(|e| format!("ns:{}",e))?;
    env.call_method(ctx, "getSharedPreferences", "(Ljava/lang/String;I)Landroid/content/SharedPreferences;", &[k.into(), 0i32.into()])
        .map_err(|e| format!("gsp:{}",e))?.l().map_err(|e| format!("l:{}",e))
}

fn ed<'a>(env: &'a mut JNIEnv, p: &JObject) -> Result<JObject<'a>, String> {
    env.call_method(p, "edit", "()Landroid/content/SharedPreferences$Editor;", &[])
        .map_err(|e| format!("ed:{}",e))?.l().map_err(|e| format!("l:{}",e))
}

fn gb(env: &mut JNIEnv, p: &JObject, key: &str, def: bool) -> Result<bool, String> {
    let k = env.new_string(key).map_err(|e| format!("ns:{}",e))?;
    env.call_method(p, "getBoolean", "(Ljava/lang/String;Z)Z", &[k.into(), def.into()])
        .map_err(|e| format!("gb:{}",e)).map(|v| v.z().unwrap_or(def))
}

fn gs(env: &mut JNIEnv, p: &JObject, key: &str, def: &str) -> Result<String, String> {
    let k = env.new_string(key).map_err(|e| format!("ns:{}",e))?;
    let d = env.new_string(def).map_err(|e| format!("ns:{}",e))?;
    let v = env.call_method(p, "getString", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;", &[k.into(), d.into()])
        .map_err(|e| format!("gs:{}",e))?;
    let o = v.l().map_err(|e| format!("l:{}",e))?;
    if o.is_null() { return Ok(def.into()); }
    let js = JString::from(o);
    Ok(env.get_string(&js).map_err(|e| format!("gs2:{}",e))?.into())
}

fn gl(env: &mut JNIEnv, p: &JObject, key: &str, def: i64) -> Result<i64, String> {
    let k = env.new_string(key).map_err(|e| format!("ns:{}",e))?;
    env.call_method(p, "getLong", "(Ljava/lang/String;J)J", &[k.into(), def.into()])
        .map_err(|e| format!("gl:{}",e)).map(|v| v.j().unwrap_or(def))
}

fn sb(env: &mut JNIEnv, e: &JObject, key: &str, val: bool) -> Result<(), String> {
    let k = env.new_string(key).map_err(|e| format!("ns:{}",e))?;
    env.call_method(e, "putBoolean", "(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;", &[k.into(), val.into()])
        .map_err(|e| format!("pb:{}",e))?;
    Ok(())
}

fn ss(env: &mut JNIEnv, e: &JObject, key: &str, val: &str) -> Result<(), String> {
    let k = env.new_string(key).map_err(|e| format!("ns:{}",e))?;
    let v = env.new_string(val).map_err(|e| format!("ns:{}",e))?;
    env.call_method(e, "putString", "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;", &[k.into(), v.into()])
        .map_err(|e| format!("ps:{}",e))?;
    Ok(())
}

fn sl(env: &mut JNIEnv, e: &JObject, key: &str, val: i64) -> Result<(), String> {
    let k = env.new_string(key).map_err(|e| format!("ns:{}",e))?;
    env.call_method(e, "putLong", "(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;", &[k.into(), val.into()])
        .map_err(|e| format!("pl:{}",e))?;
    Ok(())
}

fn csv(s: &str) -> Vec<String> { s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect() }

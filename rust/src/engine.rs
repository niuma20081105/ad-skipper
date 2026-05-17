use jni::JNIEnv;
use jni::objects::{JObject, JString};
use log::{info, debug, warn};
use regex::RegexSet;
use std::time::{Duration, Instant};
use crate::config::AppConfig;

const DEFAULT_KEYWORDS: &[&str] = &[r"跳过", r"SKIP", r"Skip", r"跳過", r"关闭", r"Close", r"×"];
const ACTION_CLICK: i32 = 16;

pub struct AdEngine {
    last_process_time: Instant, cooldown_ms: u64, match_count: u64,
    keyword_set: RegexSet, config: AppConfig,
}

impl AdEngine {
    pub fn new() -> Self {
        let kw: Vec<&str> = DEFAULT_KEYWORDS.iter().copied().collect();
        AdEngine { last_process_time: Instant::now(), cooldown_ms: 50, match_count: 0,
            keyword_set: RegexSet::new(&kw).unwrap(), config: AppConfig::default() }
    }
    pub fn update_config(&mut self, c: AppConfig) {
        let p: Vec<String> = if c.keywords.is_empty() { DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect() } else { c.keywords.clone() };
        self.keyword_set = RegexSet::new(&p).unwrap();
        self.config = c;
    }
    pub fn process(&mut self, env: &mut JNIEnv, root: &JObject, pkg: &str) -> Result<Option<String>, String> {
        if !self.config.enabled { return Ok(None); }
        if self.config.app_blacklist.iter().any(|a| a == pkg) { return Ok(None); }
        if !self.config.app_whitelist.is_empty() && !self.config.app_whitelist.iter().any(|a| a == pkg) { return Ok(None); }
        if self.last_process_time.elapsed() < Duration::from_millis(self.cooldown_ms) { return Ok(None); }
        self.last_process_time = Instant::now();
        match self.walk(env, root, 0) {
            Ok(Some(kw)) => { self.match_count += 1; info!("Clicked in [{}]: {}", pkg, kw); Ok(Some(kw)) }
            Ok(None) => Ok(None),
            Err(e) => { warn!("Error in [{}]: {}", pkg, e); Err(e) }
        }
    }
    fn walk(&self, env: &mut JNIEnv, node: &JObject, depth: u32) -> Result<Option<String>, String> {
        if depth > 50 { return Ok(None); }
        let txt = txt(env, node).unwrap_or_default();
        let desc = desc(env, node).unwrap_or_default();
        let s = format!("{} {}", txt, desc);
        if !s.trim().is_empty() && self.keyword_set.is_match(&s) {
            let kw = s.trim().to_string();
            let mut clicked = false;
            if ok(env, node) { match clk(env, node) { Ok(true) => clicked = true, _ => {} } }
            if !clicked {
                if let Ok(p) = parent(env, node) { if ok(env, &p) { if let Ok(true) = clk(env, &p) { clicked = true; } } }
            }
            if clicked { return Ok(Some(kw)); }
        }
        let n = cnt(env, node).unwrap_or(0);
        for i in 0..n {
            if let Ok(c) = child(env, node, i) {
                if let Some(kw) = self.walk(env, &c, depth + 1)? { return Ok(Some(kw)); }
            }
        }
        Ok(None)
    }
}

fn txt(env: &mut JNIEnv, n: &JObject) -> Result<String, String> {
    let v = env.call_method(n, "getText", "()Ljava/lang/CharSequence;", &[]).map_err(|e| format!("gt:{}",e))?;
    let o = v.l().map_err(|e| format!("l:{}",e))?;
    if o.is_null() { return Ok(String::new()); }
    let t = env.call_method(o, "toString", "()Ljava/lang/String;", &[]).map_err(|e| format!("ts:{}",e))?;
    let j = t.l().map_err(|e| format!("l2:{}",e))?;
    let js = JString::from(j);
    Ok(env.get_string(&js).map_err(|e| format!("gs:{}",e))?.into())
}

fn desc(env: &mut JNIEnv, n: &JObject) -> Result<String, String> {
    let v = env.call_method(n, "getContentDescription", "()Ljava/lang/CharSequence;", &[]).map_err(|e| format!("gcd:{}",e))?;
    let o = v.l().map_err(|e| format!("l:{}",e))?;
    if o.is_null() { return Ok(String::new()); }
    let t = env.call_method(o, "toString", "()Ljava/lang/String;", &[]).map_err(|e| format!("ts:{}",e))?;
    let j = t.l().map_err(|e| format!("l2:{}",e))?;
    let js = JString::from(j);
    Ok(env.get_string(&js).map_err(|e| format!("gs:{}",e))?.into())
}

fn ok(env: &mut JNIEnv, n: &JObject) -> bool {
    env.call_method(n, "isClickable", "()Z", &[]).map(|v| v.z().unwrap_or(false)).unwrap_or(false)
}

fn cnt(env: &mut JNIEnv, n: &JObject) -> Result<i32, String> {
    env.call_method(n, "getChildCount", "()I", &[]).map_err(|e| format!("cc:{}",e)).map(|v| v.i().unwrap_or(0))
}

fn child<'a>(env: &'a mut JNIEnv, n: &JObject, i: i32) -> Result<JObject<'a>, String> {
    env.call_method(n, "getChild", "(I)Landroid/view/accessibility/AccessibilityNodeInfo;", &[i.into()])
        .map_err(|e| format!("gc:{}",e))?.l().map_err(|e| format!("l:{}",e))
}

fn parent<'a>(env: &'a mut JNIEnv, n: &JObject) -> Result<JObject<'a>, String> {
    env.call_method(n, "getParent", "()Landroid/view/accessibility/AccessibilityNodeInfo;", &[])
        .map_err(|e| format!("gp:{}",e))?.l().map_err(|e| format!("l:{}",e))
}

fn clk(env: &mut JNIEnv, n: &JObject) -> Result<bool, String> {
    env.call_method(n, "performAction", "(I)Z", &[(ACTION_CLICK).into()])
        .map_err(|e| format!("pa:{}",e)).map(|v| v.z().unwrap_or(false))
}

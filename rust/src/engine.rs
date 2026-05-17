use jni::JNIEnv;
use jni::objects::JObject;
use log::{info, debug, warn};
use regex::RegexSet;
use std::time::{Duration, Instant};

use crate::config::AppConfig;

const DEFAULT_KEYWORDS: &[&str] = &[
    r"跳过", r"SKIP", r"Skip", r"跳過", r"关闭", r"Close", r"×",
];

const ACTION_CLICK: i32 = 16;

pub struct AdEngine {
    last_process_time: Instant,
    cooldown_ms: u64,
    match_count: u64,
    keyword_set: RegexSet,
    config: AppConfig,
}

impl AdEngine {
    pub fn new() -> Self {
        let patterns: Vec<&str> = DEFAULT_KEYWORDS.iter().copied().collect();
        let keyword_set = RegexSet::new(&patterns).expect("Failed to compile keyword regex");
        AdEngine {
            last_process_time: Instant::now(),
            cooldown_ms: 50,
            match_count: 0,
            keyword_set,
            config: AppConfig::default(),
        }
    }

    pub fn update_config(&mut self, config: AppConfig) {
        let patterns: Vec<String> = if config.keywords.is_empty() {
            DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect()
        } else {
            config.keywords.clone()
        };
        self.keyword_set = RegexSet::new(&patterns).expect("Failed to compile keyword regex");
        self.config = config;
    }

    pub fn process(
        &mut self,
        env: &mut JNIEnv,
        root: &JObject,
        package: &str,
    ) -> Result<Option<String>, String> {
        if !self.config.enabled { return Ok(None); }
        if self.config.app_blacklist.iter().any(|a| a == package) { return Ok(None); }
        if !self.config.app_whitelist.is_empty()
            && !self.config.app_whitelist.iter().any(|a| a == package) { return Ok(None); }
        let elapsed = self.last_process_time.elapsed();
        if elapsed < Duration::from_millis(self.cooldown_ms) { return Ok(None); }
        self.last_process_time = Instant::now();

        debug!("Processing window: package={}", package);
        match self.traverse_node(env, root, 0) {
            Ok(Some(kw)) => {
                self.match_count += 1;
                info!("Clicked skip in [{}] matched='{}' (total: {})", package, kw, self.match_count);
                Ok(Some(kw))
            }
            Ok(None) => { debug!("No skip in [{}]", package); Ok(None) }
            Err(e) => { warn!("Traversal error in [{}]: {}", package, e); Err(e) }
        }
    }

    fn traverse_node(
        &self,
        env: &mut JNIEnv,
        node: &JObject,
        depth: u32,
    ) -> Result<Option<String>, String> {
        if depth > 50 { return Ok(None); }

        let text = node_text(env, node).unwrap_or_default();
        let desc = node_desc(env, node).unwrap_or_default();
        let combined = format!("{} {}", text, desc);

        if !combined.trim().is_empty() && self.keyword_set.is_match(&combined) {
            let kw = combined.trim().to_string();
            debug!("Matched: '{}' depth={}", kw, depth);

            if node_clickable(env, node).unwrap_or(false) {
                if let Ok(true) = do_click(env, node) {
                    info!("Clicked: '{}'", kw);
                    return Ok(Some(kw));
                }
            } else if let Ok(parent) = node_parent(env, node) {
                if node_clickable(env, &parent).unwrap_or(false) {
                    if let Ok(true) = do_click(env, &parent) {
                        info!("Clicked parent of: '{}'", kw);
                        return Ok(Some(kw));
                    }
                }
            }
        }

        let n = child_count(env, node).unwrap_or(0);
        for i in 0..n {
            if let Ok(child) = get_child(env, node, i) {
                if let Some(kw) = self.traverse_node(env, &child, depth + 1)? {
                    return Ok(Some(kw));
                }
            }
        }
        Ok(None)
    }
}

// ─── JNI helpers (jni 0.20: &JObject works in call_method) ─────

fn node_text(env: &mut JNIEnv, node: &JObject) -> Result<String, String> {
    let v = env.call_method(node, "getText", "()Ljava/lang/CharSequence;", &[])
        .map_err(|e| format!("getText: {e}"))?;
    let obj = match v.l() { Ok(o) => o, Err(_) => return Ok(String::new()) };
    if obj.is_null() { return Ok(String::new()); }
    let s = env.call_method(obj, "toString", "()Ljava/lang/String;", &[])
        .map_err(|e| format!("toString: {e}"))?;
    let jobj = s.l().map_err(|e| format!("l: {e}"))?;
    let jstr = jni::objects::JString::from(jobj);
    Ok(env.get_string(&jstr).map_err(|e| format!("get_string: {e}"))?.into())
}

fn node_desc(env: &mut JNIEnv, node: &JObject) -> Result<String, String> {
    let v = env.call_method(node, "getContentDescription", "()Ljava/lang/CharSequence;", &[])
        .map_err(|e| format!("getDesc: {e}"))?;
    let obj = match v.l() { Ok(o) => o, Err(_) => return Ok(String::new()) };
    if obj.is_null() { return Ok(String::new()); }
    let s = env.call_method(obj, "toString", "()Ljava/lang/String;", &[])
        .map_err(|e| format!("toString: {e}"))?;
    let jobj = s.l().map_err(|e| format!("l: {e}"))?;
    let jstr = jni::objects::JString::from(jobj);
    Ok(env.get_string(&jstr).map_err(|e| format!("get_string: {e}"))?.into())
}

fn node_clickable(env: &mut JNIEnv, node: &JObject) -> Result<bool, String> {
    let v = env.call_method(node, "isClickable", "()Z", &[])
        .map_err(|e| format!("isClickable: {e}"))?;
    Ok(v.z().unwrap_or(false))
}

fn child_count(env: &mut JNIEnv, node: &JObject) -> Result<i32, String> {
    let v = env.call_method(node, "getChildCount", "()I", &[])
        .map_err(|e| format!("childCount: {e}"))?;
    Ok(v.i().unwrap_or(0))
}

fn get_child<'a>(env: &'a mut JNIEnv, node: &JObject, idx: i32) -> Result<JObject<'a>, String> {
    env.call_method(node, "getChild", "(I)Landroid/view/accessibility/AccessibilityNodeInfo;", &[idx.into()])
        .map_err(|e| format!("getChild: {e}"))?
        .l().map_err(|e| format!("getChild l: {e}"))
}

fn node_parent<'a>(env: &'a mut JNIEnv, node: &JObject) -> Result<JObject<'a>, String> {
    env.call_method(node, "getParent", "()Landroid/view/accessibility/AccessibilityNodeInfo;", &[])
        .map_err(|e| format!("getParent: {e}"))?
        .l().map_err(|e| format!("getParent l: {e}"))
}

fn do_click(env: &mut JNIEnv, node: &JObject) -> Result<bool, String> {
    env.call_method(node, "performAction", "(I)Z", &[(ACTION_CLICK).into()])
        .map_err(|e| format!("performAction: {e}"))
        .map(|v| v.z().unwrap_or(false))
}

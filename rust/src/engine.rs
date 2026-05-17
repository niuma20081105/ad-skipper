use jni::JNIEnv;
use jni::objects::JObject;
use log::{info, debug, warn};
use regex::RegexSet;
use std::time::{Duration, Instant};

use crate::config::AppConfig;

/// 默认匹配关键词
const DEFAULT_KEYWORDS: &[&str] = &[
    r"跳过",
    r"SKIP",
    r"Skip",
    r"跳過",
    r"关闭",
    r"Close",
    r"×",
];

/// ACTION_CLICK 常量值，来自 AccessibilityNodeInfo
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
        let patterns: Vec<&str> = DEFAULT_KEYWORDS.iter().map(|s| *s).collect();
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
        self.keyword_set =
            RegexSet::new(&patterns).expect("Failed to compile keyword regex");
        self.config = config;
    }

    pub fn process(
        &mut self,
        env: &mut JNIEnv,
        root: &JObject,
        package: &str,
    ) -> Result<Option<String>, String> {
        // 检查是否启用
        if !self.config.enabled {
            return Ok(None);
        }

        // 检查黑名单
        if self.config.app_blacklist.iter().any(|a| a == package) {
            return Ok(None);
        }

        // 检查白名单（如果有设置）
        if !self.config.app_whitelist.is_empty()
            && !self.config.app_whitelist.iter().any(|a| a == package)
        {
            return Ok(None);
        }

        // 防抖：避免高频重复处理
        let elapsed = self.last_process_time.elapsed();
        if elapsed < Duration::from_millis(self.cooldown_ms) {
            return Ok(None);
        }
        self.last_process_time = Instant::now();

        debug!("Processing window: package={}", package);

        // 递归遍历节点树
        match self.traverse_node(env, root, 0) {
            Ok(Some(keyword)) => {
                self.match_count += 1;
                info!(
                    "Skip button clicked in [{}] matched='{}' (total: {})",
                    package, keyword, self.match_count
                );
                Ok(Some(keyword))
            }
            Ok(None) => {
                debug!("No skip button found in [{}]", package);
                Ok(None)
            }
            Err(e) => {
                warn!("Traversal error in [{}]: {}", package, e);
                Err(e)
            }
        }
    }

    /// 递归遍历节点树，返回 Some(keyword) 表示找到并点击了跳过按钮
    fn traverse_node(
        &self,
        env: &mut JNIEnv,
        node: &JObject,
        depth: u32,
    ) -> Result<Option<String>, String> {
        if depth > 50 {
            return Ok(None); // 防止过深递归
        }

        // 获取节点文本
        let text = get_node_text(env, node).unwrap_or_default();
        let content_desc = get_content_description(env, node).unwrap_or_default();
        let combined = format!("{} {}", text, content_desc);

        // 检查文本是否匹配跳过关键词
        if !combined.trim().is_empty() && self.keyword_set.is_match(&combined) {
            debug!(
                "Matched node: text='{}' desc='{}' depth={}",
                text, content_desc, depth
            );

            let keyword = combined.trim().to_string();

            // 检查是否可点击
            if is_node_clickable(env, node).unwrap_or(false) {
                match perform_click(env, node) {
                    Ok(true) => {
                        info!("Clicked skip button: '{}'", keyword);
                        return Ok(Some(keyword));
                    }
                    Ok(false) => debug!("Click performed but returned false"),
                    Err(e) => warn!("Click failed: {}", e),
                }
            } else {
                // 如果节点本身不可点击，尝试点击其父节点
                debug!("Node not clickable, trying parent");
                if let Ok(parent) = get_parent(env, node) {
                    if is_node_clickable(env, &parent).unwrap_or(false) {
                        match perform_click(env, &parent) {
                            Ok(true) => {
                                info!("Clicked parent of skip button: '{}'", keyword);
                                return Ok(Some(keyword));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // 递归处理子节点
        let child_count = get_child_count(env, node).unwrap_or(0);
        for i in 0..child_count {
            match get_child(env, node, i) {
                Ok(child) => {
                    if let Some(keyword) = self.traverse_node(env, &child, depth + 1)? {
                        return Ok(Some(keyword));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }
}

// ─── JNI 辅助函数 ──────────────────────────────────────────────

fn get_node_text(env: &mut JNIEnv, node: &JObject) -> Result<String, String> {
    let text_obj = env
        .call_method(node, "getText", "()Ljava/lang/CharSequence;", &[])
        .map_err(|e| format!("getText failed: {e}"))?;

    // getText 可能返回 null
    let text_jobj = match text_obj.l() {
        Ok(jobj) => jobj,
        Err(_) => return Ok(String::new()),
    };

    if text_jobj.is_null() {
        return Ok(String::new());
    }

    let text_str = env
        .call_method(text_jobj, "toString", "()Ljava/lang/String;", &[])
        .map_err(|e| format!("toString failed: {e}"))?;

    let s: String = env
        .get_string(&jni::objects::JString::from(
            text_str.l().map_err(|e| format!("get_string: {e}"))?,
        ))
        .map_err(|e| format!("get_string: {e}"))?
        .into();

    Ok(s)
}

fn get_content_description(env: &mut JNIEnv, node: &JObject) -> Result<String, String> {
    let desc_obj = env
        .call_method(
            node,
            "getContentDescription",
            "()Ljava/lang/CharSequence;",
            &[],
        )
        .map_err(|e| format!("getContentDescription failed: {e}"))?;

    let desc_jobj = match desc_obj.l() {
        Ok(jobj) => jobj,
        Err(_) => return Ok(String::new()),
    };

    if desc_jobj.is_null() {
        return Ok(String::new());
    }

    let desc_str = env
        .call_method(desc_jobj, "toString", "()Ljava/lang/String;", &[])
        .map_err(|e| format!("toString failed: {e}"))?;

    let s: String = env
        .get_string(&jni::objects::JString::from(
            desc_str.l().map_err(|e| format!("get_string: {e}"))?,
        ))
        .map_err(|e| format!("get_string: {e}"))?
        .into();

    Ok(s)
}

fn is_node_clickable(env: &mut JNIEnv, node: &JObject) -> Result<bool, String> {
    let clickable = env
        .call_method(node, "isClickable", "()Z", &[])
        .map_err(|e| format!("isClickable failed: {e}"))?;
    Ok(clickable.z().unwrap_or(false))
}

fn get_child_count(env: &mut JNIEnv, node: &JObject) -> Result<i32, String> {
    let count = env
        .call_method(node, "getChildCount", "()I", &[])
        .map_err(|e| format!("getChildCount failed: {e}"))?;
    Ok(count.i().unwrap_or(0))
}

fn get_child<'a>(
    env: &'a mut JNIEnv,
    node: &JObject,
    index: i32,
) -> Result<JObject<'a>, String> {
    let child = env
        .call_method(node, "getChild", "(I)Landroid/view/accessibility/AccessibilityNodeInfo;", &[index.into()])
        .map_err(|e| format!("getChild({}) failed: {}", index, e))?;
    child.l().map_err(|e| format!("getChild l(): {e}"))
}

fn get_parent<'a>(env: &'a mut JNIEnv, node: &JObject) -> Result<JObject<'a>, String> {
    let parent = env
        .call_method(node, "getParent", "()Landroid/view/accessibility/AccessibilityNodeInfo;", &[])
        .map_err(|e| format!("getParent failed: {e}"))?;
    parent.l().map_err(|e| format!("getParent l(): {e}"))
}

fn perform_click(env: &mut JNIEnv, node: &JObject) -> Result<bool, String> {
    let result = env
        .call_method(node, "performAction", "(I)Z", &[(ACTION_CLICK).into()])
        .map_err(|e| format!("performAction(CLICK) failed: {e}"))?;
    Ok(result.z().unwrap_or(false))
}

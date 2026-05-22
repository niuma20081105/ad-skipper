package com.adskipper;

import android.view.accessibility.AccessibilityNodeInfo;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.regex.Pattern;

public class AdEngine {

    private static final String[] DEFAULT_KEYWORDS = {
            "跳过", "SKIP", "Skip", "跳過", "关闭", "Close", "×"
    };
    private static final int ACTION_CLICK = 16;
    private static final int MAX_DEPTH = 50;

    private boolean enabled = true;
    private long coolDownMs = 50;
    private long lastProcessTime = 0;
    private long matchCount = 0;
    private Pattern keywordPattern;
    private List<String> keywords = new ArrayList<>();
    private List<String> appWhitelist = new ArrayList<>();
    private List<String> appBlacklist = new ArrayList<>();

    public AdEngine() {
        compileKeywords(Arrays.asList(DEFAULT_KEYWORDS));
    }

    public void updateConfig(Config.AppConfig config) {
        this.enabled = config.enabled;
        this.appWhitelist = config.appWhitelist;
        this.appBlacklist = config.appBlacklist;
        if (config.keywords != null && !config.keywords.isEmpty()) {
            compileKeywords(config.keywords);
        } else {
            compileKeywords(Arrays.asList(DEFAULT_KEYWORDS));
        }
    }

    private void compileKeywords(List<String> kwList) {
        this.keywords = new ArrayList<>(kwList);
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < kwList.size(); i++) {
            if (i > 0) sb.append('|');
            sb.append(Pattern.quote(kwList.get(i)));
        }
        this.keywordPattern = sb.length() > 0 ? Pattern.compile(sb.toString()) : null;
    }

    public ProcessResult process(AccessibilityNodeInfo root, String pkg) {
        if (!enabled) return ProcessResult.SKIPPED;
        if (appBlacklist.contains(pkg)) return ProcessResult.SKIPPED;
        if (!appWhitelist.isEmpty() && !appWhitelist.contains(pkg)) return ProcessResult.SKIPPED;

        long now = System.currentTimeMillis();
        if (now - lastProcessTime < coolDownMs) return ProcessResult.SKIPPED;
        lastProcessTime = now;

        String matched = walk(root, 0);
        if (matched != null) {
            matchCount++;
            android.util.Log.i("AdEngine", "Clicked in [" + pkg + "]: " + matched);
            return new ProcessResult(true, matched);
        }
        return ProcessResult.NO_MATCH;
    }

    private String walk(AccessibilityNodeInfo node, int depth) {
        if (node == null || depth > MAX_DEPTH) return null;

        String text = safeText(node);
        String desc = safeContentDescription(node);
        String combined = (text != null ? text : "") + " " + (desc != null ? desc : "");
        combined = combined.trim();

        String matchedKw = null;
        if (!combined.isEmpty() && keywordPattern != null) {
            java.util.regex.Matcher m = keywordPattern.matcher(combined);
            if (m.find()) {
                matchedKw = combined;
            }
        }

        if (matchedKw != null) {
            boolean clicked = false;
            // Try clicking the node itself
            if (node.isClickable()) {
                clicked = node.performAction(ACTION_CLICK);
            }
            // If not clicked, try parent
            if (!clicked) {
                AccessibilityNodeInfo parent = node.getParent();
                if (parent != null) {
                    if (parent.isClickable()) {
                        clicked = parent.performAction(ACTION_CLICK);
                    }
                    parent.recycle();
                }
            }
            if (clicked) return matchedKw;
        }

        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child != null) {
                String result = walk(child, depth + 1);
                child.recycle();
                if (result != null) return result;
            }
        }

        return null;
    }

    private String safeText(AccessibilityNodeInfo node) {
        CharSequence t = node.getText();
        return t != null ? t.toString() : null;
    }

    private String safeContentDescription(AccessibilityNodeInfo node) {
        CharSequence d = node.getContentDescription();
        return d != null ? d.toString() : null;
    }

    public long getMatchCount() {
        return matchCount;
    }

    public static class ProcessResult {
        public static final ProcessResult SKIPPED = new ProcessResult(false, null);
        public static final ProcessResult NO_MATCH = new ProcessResult(false, null);

        public final boolean clicked;
        public final String matchedKeyword;

        public ProcessResult(boolean clicked, String matchedKeyword) {
            this.clicked = clicked;
            this.matchedKeyword = matchedKeyword;
        }
    }
}

package com.adskipper;

import android.content.Context;
import android.content.SharedPreferences;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class Config {

    private static final String PREFS_NAME = "ad_skipper_prefs";

    public static class AppConfig {
        public boolean enabled = true;
        public List<String> keywords = new ArrayList<>();
        public long autoClickDelayMs = 0;
        public List<String> appWhitelist = new ArrayList<>();
        public List<String> appBlacklist = new ArrayList<>();
        public boolean logEnabled = true;
    }

    public static AppConfig load(Context context) {
        SharedPreferences p = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
        AppConfig c = new AppConfig();
        c.enabled = p.getBoolean("enabled", true);
        c.keywords = csv(p.getString("keywords", ""));
        c.autoClickDelayMs = p.getLong("delay_ms", 0);
        c.appWhitelist = csv(p.getString("whitelist", ""));
        c.appBlacklist = csv(p.getString("blacklist", ""));
        c.logEnabled = p.getBoolean("log_enabled", true);
        return c;
    }

    public static void save(Context context, AppConfig c) {
        SharedPreferences.Editor e = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit();
        e.putBoolean("enabled", c.enabled);
        e.putString("keywords", joinCsv(c.keywords));
        e.putLong("delay_ms", c.autoClickDelayMs);
        e.putString("whitelist", joinCsv(c.appWhitelist));
        e.putString("blacklist", joinCsv(c.appBlacklist));
        e.putBoolean("log_enabled", c.logEnabled);
        e.apply();
    }

    private static List<String> csv(String s) {
        if (s == null || s.isEmpty()) return new ArrayList<>();
        String[] parts = s.split(",");
        List<String> result = new ArrayList<>();
        for (String p : parts) {
            String t = p.trim();
            if (!t.isEmpty()) result.add(t);
        }
        return result;
    }

    private static String joinCsv(List<String> list) {
        if (list == null || list.isEmpty()) return "";
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < list.size(); i++) {
            if (i > 0) sb.append(',');
            sb.append(list.get(i));
        }
        return sb.toString();
    }
}

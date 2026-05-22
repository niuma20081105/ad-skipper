package com.adskipper;

import java.util.ArrayList;
import java.util.List;

public class SkipLogger {

    public static class Entry {
        public long timestamp;
        public String pkg;
        public String keywordMatched;
        public boolean clicked;
        public String result;

        public Entry(long timestamp, String pkg, String keywordMatched, boolean clicked, String result) {
            this.timestamp = timestamp;
            this.pkg = pkg;
            this.keywordMatched = keywordMatched;
            this.clicked = clicked;
            this.result = result;
        }
    }

    private final List<Entry> entries = new ArrayList<>();

    public synchronized void log(String pkg, String kw, boolean clicked, String result) {
        entries.add(new Entry(System.currentTimeMillis() / 1000, pkg, kw, clicked, result));
    }

    public synchronized int todayCount() {
        int count = 0;
        for (Entry e : entries) {
            if (e.clicked) count++;
        }
        return count;
    }

    public synchronized List<Entry> recent(int n) {
        int len = entries.size();
        if (len == 0) return new ArrayList<>();
        int start = Math.max(0, len - n);
        return new ArrayList<>(entries.subList(start, len));
    }

    public synchronized String formatRecent(int n) {
        List<Entry> list = recent(n);
        if (list.isEmpty()) return "暂无日志";
        StringBuilder sb = new StringBuilder();
        for (Entry e : list) {
            sb.append('[').append(e.timestamp).append("] ")
                    .append(e.pkg).append(" | ")
                    .append(e.keywordMatched).append('\n');
        }
        return sb.toString();
    }
}

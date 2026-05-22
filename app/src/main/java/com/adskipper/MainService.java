package com.adskipper;

import android.accessibilityservice.AccessibilityService;
import android.accessibilityservice.AccessibilityServiceInfo;
import android.util.Log;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;

public class MainService extends AccessibilityService {

    private static final String TAG = "AdSkipper";
    private static MainService instance;

    private final AdEngine engine = new AdEngine();
    private final SkipLogger logger = new SkipLogger();

    public static MainService getInstance() {
        return instance;
    }

    @Override
    public void onCreate() {
        super.onCreate();
        instance = this;
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        instance = null;
    }

    @Override
    public void onServiceConnected() {
        super.onServiceConnected();
        AccessibilityServiceInfo info = new AccessibilityServiceInfo();
        info.eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
                | AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED;
        info.feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC;
        info.flags = AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS
                | AccessibilityServiceInfo.DEFAULT
                | AccessibilityServiceInfo.FLAG_INCLUDE_NOT_IMPORTANT_VIEWS;
        info.notificationTimeout = 100;
        setServiceInfo(info);

        // Load config from SharedPreferences
        Config.AppConfig config = Config.load(this);
        engine.updateConfig(config);

        Log.i(TAG, "Accessibility service connected");
    }

    @Override
    public void onAccessibilityEvent(AccessibilityEvent event) {
        if (event == null) return;

        int type = event.getEventType();
        if (type == AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
                || type == AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED) {

            AccessibilityNodeInfo root = getRootInActiveWindow();
            if (root == null) return;

            String packageName = event.getPackageName() != null
                    ? event.getPackageName().toString() : "";

            AdEngine.ProcessResult result = engine.process(root, packageName);
            if (result.clicked && result.matchedKeyword != null) {
                Config.AppConfig config = Config.load(this);
                if (config.logEnabled) {
                    logger.log(packageName, result.matchedKeyword, true, "clicked");
                }
            }

            root.recycle();
        }
    }

    @Override
    public void onInterrupt() {
        Log.i(TAG, "Service interrupted");
    }

    public SkipLogger getLogger() {
        return logger;
    }

    public AdEngine getEngine() {
        return engine;
    }
}

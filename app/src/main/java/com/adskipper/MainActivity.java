package com.adskipper;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.os.Bundle;
import android.provider.Settings;
import android.widget.Button;
import android.widget.Switch;
import android.widget.TextView;

public class MainActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        Switch switchEnabled = findViewById(R.id.switch_enabled);
        TextView tvCount = findViewById(R.id.tv_skip_count);
        TextView tvStatus = findViewById(R.id.tv_service_status);
        Button btnSettings = findViewById(R.id.btn_accessibility_settings);
        Button btnLog = findViewById(R.id.btn_view_log);

        // Load config
        Config.AppConfig config = Config.load(this);
        switchEnabled.setChecked(config.enabled);

        // Update service status
        updateServiceStatus(tvStatus);

        switchEnabled.setOnCheckedChangeListener((buttonView, isChecked) -> {
            Config.AppConfig c = Config.load(this);
            c.enabled = isChecked;
            Config.save(this, c);
            // Also update engine config if service is running
            MainService service = MainService.getInstance();
            if (service != null) {
                service.getEngine().updateConfig(c);
            }
            tvCount.setText("今日跳过: 0");
        });

        btnSettings.setOnClickListener(v -> {
            startActivity(new Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS));
        });

        btnLog.setOnClickListener(v -> {
            Intent intent = new Intent(this, LogActivity.class);
            intent.putExtra("log_data", getRecentLogs());
            startActivity(intent);
        });

        // Update stats
        SkipLogger logger = getLogger();
        if (logger != null) {
            tvCount.setText("今日跳过: " + logger.todayCount());
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        TextView tvStatus = findViewById(R.id.tv_service_status);
        updateServiceStatus(tvStatus);

        TextView tvCount = findViewById(R.id.tv_skip_count);
        SkipLogger logger = getLogger();
        if (logger != null) {
            tvCount.setText("今日跳过: " + logger.todayCount());
        }
    }

    private void updateServiceStatus(TextView tv) {
        boolean enabled = isAccessibilityServiceEnabled(this, MainService.class);
        tv.setText(enabled ? "服务状态: 已开启" : "服务状态: 未开启");
    }

    private boolean isAccessibilityServiceEnabled(Context context, Class<?> serviceClass) {
        String enabledServices = Settings.Secure.getString(
                context.getContentResolver(),
                Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES
        );
        if (enabledServices == null) return false;
        return enabledServices.contains(context.getPackageName() + "/" + serviceClass.getName());
    }

    private SkipLogger getLogger() {
        MainService service = MainService.getInstance();
        return service != null ? service.getLogger() : null;
    }

    private String getRecentLogs() {
        SkipLogger logger = getLogger();
        if (logger != null) {
            return logger.formatRecent(20);
        }
        return "暂无日志";
    }
}

package com.adskipper;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public class LogActivity extends Activity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_log);

        String logData = getIntent().getStringExtra("log_data");
        if (logData == null) logData = "暂无日志";

        TextView tvLog = findViewById(R.id.tv_log_content);
        tvLog.setText(logData);
    }
}

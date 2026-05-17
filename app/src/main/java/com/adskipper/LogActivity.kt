package com.adskipper

import android.app.Activity
import android.os.Bundle
import android.widget.TextView

class LogActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_log)

        val logData = intent.getStringExtra("log_data") ?: "暂无日志"
        findViewById<TextView>(R.id.tv_log_content).text = logData
    }
}

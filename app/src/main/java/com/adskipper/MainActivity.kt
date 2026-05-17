package com.adskipper

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.widget.Button
import android.widget.Switch
import android.widget.TextView

class MainActivity : Activity() {

    init {
        System.loadLibrary("ad_skipper")
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val switchEnabled = findViewById<Switch>(R.id.switch_enabled)
        val tvCount = findViewById<TextView>(R.id.tv_skip_count)
        val tvStatus = findViewById<TextView>(R.id.tv_service_status)
        val btnSettings = findViewById<Button>(R.id.btn_accessibility_settings)
        val btnLog = findViewById<Button>(R.id.btn_view_log)

        // 从 Rust 加载当前配置
        val configJson = nativeGetConfig(this)
        val config = parseSimpleConfig(configJson)

        // 初始化开关状态
        switchEnabled.isChecked = config.enabled
        updateServiceStatus(tvStatus)

        switchEnabled.setOnCheckedChangeListener { _, isChecked ->
            nativeSetEnabled(this, isChecked)
            tvCount.text = "今日跳过: 0"
        }

        btnSettings.setOnClickListener {
            startActivity(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS))
        }

        btnLog.setOnClickListener {
            val log = nativeGetRecentLogs(20)
            val intent = Intent(this, LogActivity::class.java)
            intent.putExtra("log_data", log)
            startActivity(intent)
        }

        // 更新统计
        tvCount.text = "今日跳过: ${nativeGetTodayCount()}"
    }

    override fun onResume() {
        super.onResume()
        val tvStatus = findViewById<TextView>(R.id.tv_service_status)
        updateServiceStatus(tvStatus)
        val tvCount = findViewById<TextView>(R.id.tv_skip_count)
        tvCount.text = "今日跳过: ${nativeGetTodayCount()}"
    }

    private fun updateServiceStatus(tv: TextView) {
        val enabled = isAccessibilityServiceEnabled(this, MainService::class.java)
        tv.text = if (enabled) "服务状态: 已开启" else "服务状态: 未开启"
    }

    private fun isAccessibilityServiceEnabled(context: Context, service: Class<*>): Boolean {
        val enabledServices = Settings.Secure.getString(
            context.contentResolver,
            Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES
        ) ?: return false
        return enabledServices.contains(context.packageName + "/" + service.name)
    }

    private data class SimpleConfig(val enabled: Boolean)

    private fun parseSimpleConfig(json: String): SimpleConfig {
        val enabled = json.contains("\"enabled\":true")
        return SimpleConfig(enabled)
    }

    // JNI native methods
    private external fun nativeGetConfig(context: Context): String
    private external fun nativeSetEnabled(context: Context, enabled: Boolean)
    private external fun nativeGetTodayCount(): Int
    private external fun nativeGetRecentLogs(max: Int): String
}

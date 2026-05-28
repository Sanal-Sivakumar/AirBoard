package com.example.clipboard

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.widget.Toast

class ClipboardWriteActivity : Activity() {
    private var hasCopied = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus && !hasCopied) {
            hasCopied = true
            val text = intent?.getStringExtra("text")
            if (text != null) {
                try {
                    val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                    val clip = ClipData.newPlainText("Synced Clipboard", text)
                    clipboard.setPrimaryClip(clip)
                    Toast.makeText(this, "Copied to clipboard!", Toast.LENGTH_SHORT).show()
                } catch (e: Exception) {
                    Toast.makeText(this, "Failed to copy: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
            finish()
        }
    }
}

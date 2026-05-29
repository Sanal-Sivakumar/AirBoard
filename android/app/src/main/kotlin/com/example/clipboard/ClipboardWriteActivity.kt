package com.example.clipboard

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.util.Log
import android.widget.Toast

class ClipboardWriteActivity : Activity() {
    private var hasCopied = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i("ClipboardWriteActivity", "onCreate: started helper activity")
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        Log.i("ClipboardWriteActivity", "onWindowFocusChanged: hasFocus = $hasFocus, hasCopied = $hasCopied")
        if (hasFocus && !hasCopied) {
            hasCopied = true
            val text = intent?.getStringExtra("text")
            Log.i("ClipboardWriteActivity", "onWindowFocusChanged: text length = ${text?.length}")
            if (text != null) {
                try {
                    val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                    val clip = ClipData.newPlainText("Synced Clipboard", text)
                    clipboard.setPrimaryClip(clip)
                    Log.i("ClipboardWriteActivity", "onWindowFocusChanged: successfully wrote to clipboard")
                    Toast.makeText(this, "Copied to clipboard!", Toast.LENGTH_SHORT).show()
                } catch (e: Exception) {
                    Log.e("ClipboardWriteActivity", "onWindowFocusChanged: failed to write to clipboard", e)
                    Toast.makeText(this, "Failed to copy: ${e.message}", Toast.LENGTH_LONG).show()
                }
            }
            Log.i("ClipboardWriteActivity", "onWindowFocusChanged: finishing activity")
            finish()
        }
    }
}

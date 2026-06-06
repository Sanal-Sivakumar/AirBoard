package com.example.clipboard

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.util.Log
import android.widget.Toast
import kotlin.concurrent.thread

class ClipboardWriteActivity : Activity() {
    private var hasCopied = false

    companion object {
        @Volatile
        var lastSentText: String? = null
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i("ClipboardWriteActivity", "onCreate: started helper activity")
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        Log.i("ClipboardWriteActivity", "onWindowFocusChanged: hasFocus = $hasFocus, hasCopied = $hasCopied")
        if (hasFocus && !hasCopied) {
            hasCopied = true
            val action = intent?.getStringExtra("action")
            Log.i("ClipboardWriteActivity", "onWindowFocusChanged: action = $action")
            
            if (action == "read_and_send") {
                try {
                    val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                    if (clipboard.hasPrimaryClip()) {
                        val clip = clipboard.primaryClip
                        if (clip != null && clip.itemCount > 0) {
                            val text = clip.getItemAt(0).text?.toString()
                            if (!text.isNullOrEmpty()) {
                                if (text == lastSentText) {
                                    Log.i("ClipboardWriteActivity", "onWindowFocusChanged: text matches lastSentText, skipping sync")
                                } else {
                                    lastSentText = text
                                    Log.i("ClipboardWriteActivity", "onWindowFocusChanged: read text from clipboard, sending to Rust socket")
                                    thread {
                                        try {
                                            val socket = java.net.Socket("127.0.0.1", 45457)
                                            val writer = socket.getOutputStream().bufferedWriter(Charsets.UTF_8)
                                            writer.write(text)
                                            writer.flush()
                                            socket.close()
                                            Log.i("ClipboardWriteActivity", "Successfully sent local clip to Rust socket")
                                        } catch (e: Exception) {
                                            Log.e("ClipboardWriteActivity", "Failed to send local clip to Rust socket", e)
                                        }
                                    }
                                    try {
                                        MainActivity.methodChannel?.invokeMethod("sendClipboardToPC", text)
                                    } catch (ignored: Exception) {}
                                    Toast.makeText(this, "Synced clipboard!", Toast.LENGTH_SHORT).show()
                                }
                            } else {
                                Log.i("ClipboardWriteActivity", "onWindowFocusChanged: clipboard content is empty")
                                Toast.makeText(this, "Clipboard is empty!", Toast.LENGTH_SHORT).show()
                            }
                        }
                    } else {
                        Log.i("ClipboardWriteActivity", "onWindowFocusChanged: no clip on clipboard")
                        Toast.makeText(this, "No text on clipboard!", Toast.LENGTH_SHORT).show()
                    }
                } catch (e: Exception) {
                    Log.e("ClipboardWriteActivity", "onWindowFocusChanged: failed to read clipboard", e)
                    Toast.makeText(this, "Failed to read clipboard: ${e.message}", Toast.LENGTH_LONG).show()
                }
            } else {
                val text = intent?.getStringExtra("text")
                Log.i("ClipboardWriteActivity", "onWindowFocusChanged: text length = ${text?.length}")
                if (text != null) {
                    try {
                        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        lastSentText = text
                        ClipboardSyncService.ignoreNextClipChange = true
                        val clip = ClipData.newPlainText("Synced Clipboard", text)
                        clipboard.setPrimaryClip(clip)
                        Log.i("ClipboardWriteActivity", "onWindowFocusChanged: successfully wrote to clipboard")
                        Toast.makeText(this, "Copied to clipboard!", Toast.LENGTH_SHORT).show()
                    } catch (e: Exception) {
                        Log.e("ClipboardWriteActivity", "onWindowFocusChanged: failed to write to clipboard", e)
                        Toast.makeText(this, "Failed to copy: ${e.message}", Toast.LENGTH_LONG).show()
                    }
                }
            }
            Log.i("ClipboardWriteActivity", "onWindowFocusChanged: finishing activity")
            finish()
        }
    }
}

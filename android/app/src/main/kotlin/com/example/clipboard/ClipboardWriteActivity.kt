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
            val action = intent?.getStringExtra("action")
            Log.i("ClipboardWriteActivity", "onWindowFocusChanged: action = $action")
            
            if (action == "read_and_send") {
                try {
                    val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                    if (clipboard.hasPrimaryClip()) {
                        val clip = clipboard.primaryClip
                        if (clip != null && clip.itemCount > 0) {
                            val item = clip.getItemAt(0)
                            val text = item.text?.toString()
                            val uri = item.uri
                            if (!text.isNullOrEmpty()) {
                                Log.i("ClipboardWriteActivity", "onWindowFocusChanged: read text from clipboard, invoking method channel")
                                MainActivity.methodChannel?.invokeMethod("sendClipboardToPC", text)
                                Toast.makeText(this, "Synced clipboard to PC!", Toast.LENGTH_SHORT).show()
                            } else if (uri != null) {
                                try {
                                    val inputStream = contentResolver.openInputStream(uri)
                                    val bytes = inputStream?.readBytes()
                                    if (bytes != null) {
                                        val base64 = android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
                                        val base64Str = "data:image/png;base64,$base64"
                                        MainActivity.methodChannel?.invokeMethod("sendClipboardToPC", base64Str)
                                        Toast.makeText(this, "Synced image to PC!", Toast.LENGTH_SHORT).show()
                                    }
                                } catch (e: Exception) {
                                    Log.e("ClipboardWriteActivity", "onWindowFocusChanged: failed to read image URI", e)
                                    Toast.makeText(this, "Failed to read image URI: ${e.message}", Toast.LENGTH_SHORT).show()
                                }
                            } else {
                                Log.i("ClipboardWriteActivity", "onWindowFocusChanged: clipboard content is empty")
                                Toast.makeText(this, "Clipboard is empty!", Toast.LENGTH_SHORT).show()
                            }
                        }
                    } else {
                        Log.i("ClipboardWriteActivity", "onWindowFocusChanged: no clip on clipboard")
                        Toast.makeText(this, "No content on clipboard!", Toast.LENGTH_SHORT).show()
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

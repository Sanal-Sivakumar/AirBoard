package com.example.clipboard

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import java.net.ServerSocket
import kotlin.concurrent.thread

class ClipboardSyncService : Service() {

    private var serverSocket: ServerSocket? = null
    private var isRunning = false

    companion object {
        private const val CHANNEL_ID = "clipboard_sync_channel"
        private const val NOTIFICATION_ID = 101
        
        const val ACTION_UPDATE_NOTIFICATION = "com.example.clipboard.UPDATE_NOTIFICATION"
        const val EXTRA_TEXT = "clipboard_text"
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startLocalServer()
    }

    override fun onDestroy() {
        stopLocalServer()
        super.onDestroy()
    }

    private fun startLocalServer() {
        isRunning = true
        thread {
            try {
                serverSocket = ServerSocket(45456, 50, java.net.InetAddress.getByName("127.0.0.1"))
                while (isRunning) {
                    val client = serverSocket?.accept() ?: break
                    thread {
                        try {
                            val reader = client.getInputStream().bufferedReader(Charsets.UTF_8)
                            val text = reader.readText()
                            if (text.isNotEmpty()) {
                                updateNotification(text)
                            }
                        } catch (e: Exception) {
                            e.printStackTrace()
                        } finally {
                            try { client.close() } catch (ignored: Exception) {}
                        }
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    private fun stopLocalServer() {
        isRunning = false
        try {
            serverSocket?.close()
        } catch (e: Exception) {
            // ignore
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent != null && intent.action == ACTION_UPDATE_NOTIFICATION) {
            val text = intent.getStringExtra(EXTRA_TEXT)
            if (text != null) {
                updateNotification(text)
            }
        } else {
            val notification = createNotification("Clipboard Sync Active")
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                    startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
                } else {
                    startForeground(NOTIFICATION_ID, notification)
                }
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
        }

        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    private fun createNotification(contentText: String, syncText: String? = null): Notification {
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
        }
            .setContentTitle("Clipboard Sync")
            .setContentText(contentText)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setOngoing(true)
            .setOnlyAlertOnce(true)

        if (syncText != null) {
            val copyIntent = Intent(this, ClipboardWriteActivity::class.java).apply {
                putExtra("text", syncText)
                action = "com.example.clipboard.COPY_ACTION_" + System.currentTimeMillis()
            }
            val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            } else {
                PendingIntent.FLAG_UPDATE_CURRENT
            }
            val pendingIntent = PendingIntent.getActivity(this, 0, copyIntent, flags)
            
            val action = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT_WATCH) {
                Notification.Action.Builder(
                    android.R.drawable.ic_menu_edit,
                    "Copy",
                    pendingIntent
                ).build()
            } else {
                null
            }
            
            if (action != null) {
                builder.addAction(action)
            }
        }

        return builder.build()
    }

    private fun updateNotification(syncText: String) {
        val previewText = if (syncText.length > 30) syncText.substring(0, 30) + "..." else syncText
        val notification = createNotification("Synced: \"$previewText\"", syncText)
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.notify(NOTIFICATION_ID, notification)
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val serviceChannel = NotificationChannel(
                CHANNEL_ID,
                "Clipboard Sync Service Channel",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(serviceChannel)
        }
    }
}

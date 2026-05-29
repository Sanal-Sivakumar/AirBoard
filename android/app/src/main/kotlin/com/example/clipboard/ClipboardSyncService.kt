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
import android.os.PowerManager
import android.net.wifi.WifiManager
import android.provider.Settings
import android.util.Log
import java.net.ServerSocket
import kotlin.concurrent.thread

class ClipboardSyncService : Service() {

    private var serverSocket: ServerSocket? = null
    private var isRunning = false
    private var wakeLock: PowerManager.WakeLock? = null
    private var wifiLock: WifiManager.WifiLock? = null

    companion object {
        private const val CHANNEL_ID = "clipboard_sync_channel_v2"
        private const val NOTIFICATION_ID = 101
        
        const val ACTION_UPDATE_NOTIFICATION = "com.example.clipboard.UPDATE_NOTIFICATION"
        const val EXTRA_TEXT = "clipboard_text"
    }

    override fun onCreate() {
        super.onCreate()
        Log.i("ClipboardSyncService", "onCreate: initializing service")

        try {
            val powerManager = getSystemService(Context.POWER_SERVICE) as PowerManager
            wakeLock = powerManager.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "AirBoard:WakeLock").apply {
                acquire()
            }
            Log.i("ClipboardSyncService", "onCreate: acquired PARTIAL_WAKE_LOCK")
        } catch (e: Exception) {
            Log.e("ClipboardSyncService", "onCreate: failed to acquire WakeLock", e)
        }

        try {
            val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
            wifiLock = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                wifiManager.createWifiLock(WifiManager.WIFI_MODE_FULL_HIGH_PERF, "AirBoard:WifiLock")
            } else {
                @Suppress("DEPRECATION")
                wifiManager.createWifiLock(WifiManager.WIFI_MODE_FULL, "AirBoard:WifiLock")
            }.apply {
                acquire()
            }
            Log.i("ClipboardSyncService", "onCreate: acquired WIFI_LOCK")
        } catch (e: Exception) {
            Log.e("ClipboardSyncService", "onCreate: failed to acquire WifiLock", e)
        }

        createNotificationChannel()
        startLocalServer()
    }

    override fun onDestroy() {
        Log.i("ClipboardSyncService", "onDestroy: stopping service")
        stopLocalServer()

        try {
            if (wakeLock?.isHeld == true) {
                wakeLock?.release()
                Log.i("ClipboardSyncService", "onDestroy: released WakeLock")
            }
        } catch (e: Exception) {
            Log.e("ClipboardSyncService", "onDestroy: failed to release WakeLock", e)
        }

        try {
            if (wifiLock?.isHeld == true) {
                wifiLock?.release()
                Log.i("ClipboardSyncService", "onDestroy: released WifiLock")
            }
        } catch (e: Exception) {
            Log.e("ClipboardSyncService", "onDestroy: failed to release WifiLock", e)
        }

        super.onDestroy()
    }

    private fun startLocalServer() {
        Log.i("ClipboardSyncService", "startLocalServer: starting local socket bridge on 127.0.0.1:45456")
        isRunning = true
        thread {
            try {
                serverSocket = ServerSocket(45456, 50, java.net.InetAddress.getByName("127.0.0.1"))
                Log.i("ClipboardSyncService", "startLocalServer: ServerSocket bound on port 45456")
                while (isRunning) {
                    val client = serverSocket?.accept() ?: break
                    Log.i("ClipboardSyncService", "startLocalServer: accepted connection from ${client.remoteSocketAddress}")
                    thread {
                        try {
                            val reader = client.getInputStream().bufferedReader(Charsets.UTF_8)
                            val text = reader.readText()
                            Log.i("ClipboardSyncService", "startLocalServer: read ${text.length} chars from socket: \"${if (text.length > 30) text.substring(0, 30) + "..." else text}\"")
                            if (text.isNotEmpty()) {
                                updateNotification(text)
                            }
                        } catch (e: Exception) {
                            Log.e("ClipboardSyncService", "startLocalServer: connection error", e)
                            e.printStackTrace()
                        } finally {
                            try { client.close() } catch (ignored: Exception) {}
                        }
                    }
                }
            } catch (e: Exception) {
                Log.e("ClipboardSyncService", "startLocalServer: server socket error", e)
                e.printStackTrace()
            }
        }
    }

    private fun stopLocalServer() {
        Log.i("ClipboardSyncService", "stopLocalServer: stopping server socket")
        isRunning = false
        try {
            serverSocket?.close()
        } catch (e: Exception) {
            // ignore
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.i("ClipboardSyncService", "onStartCommand: action = ${intent?.action}")
        if (intent != null && intent.action == ACTION_UPDATE_NOTIFICATION) {
            val text = intent.getStringExtra(EXTRA_TEXT)
            Log.i("ClipboardSyncService", "onStartCommand: update request, text = ${if (text != null) "\"$text\"" else "null"}")
            if (text != null) {
                updateNotification(text)
            }
        } else {
            Log.i("ClipboardSyncService", "onStartCommand: normal service startup, launching foreground notification")
            val notification = createNotification("Clipboard Sync Active")
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
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
        Log.i("ClipboardSyncService", "createNotification: contentText = \"$contentText\", hasSyncText = ${syncText != null}")
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
        }
            .setContentTitle("AirBoard")
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
        Log.i("ClipboardSyncService", "updateNotification: syncText length = ${syncText.length}")
        val previewText = if (syncText.length > 30) syncText.substring(0, 30) + "..." else syncText
        
        val hasOverlay = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            Settings.canDrawOverlays(this)
        } else {
            true
        }

        if (hasOverlay) {
            Log.i("ClipboardSyncService", "updateNotification: overlay permission granted, launching ClipboardWriteActivity automatically")
            try {
                val writeIntent = Intent(this, ClipboardWriteActivity::class.java).apply {
                    putExtra("text", syncText)
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    action = "com.example.clipboard.WRITE_ACTION_" + System.currentTimeMillis()
                }
                startActivity(writeIntent)
            } catch (e: Exception) {
                Log.e("ClipboardSyncService", "updateNotification: failed to launch ClipboardWriteActivity automatically", e)
            }
        } else {
            Log.i("ClipboardSyncService", "updateNotification: overlay permission not granted, will require manual notification copy action")
        }

        val notification = createNotification("Synced: \"$previewText\"", syncText)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val serviceChannel = NotificationChannel(
                CHANNEL_ID,
                "AirBoard Sync Service Channel",
                NotificationManager.IMPORTANCE_DEFAULT
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(serviceChannel)
        }
    }
}

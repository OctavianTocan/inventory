package com.octaviantocan.mymind.update

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import com.octaviantocan.mymind.MymindApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

/**
 * Keeps the app running, with a notification, while uploads are in flight, so leaving the app or the share sheet does
 * not cut an upload off. It stops itself when no upload is left.
 */
class UploadService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private var watcher: Job? = null
    private var lastStart = 0

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        ServiceCompat.startForeground(this, NOTIFICATION_ID, notification(), ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        lastStart = startId
        if (watcher?.isActive != true) {
            val uploads = (application as MymindApp).container.uploadActivity
            watcher = scope.launch {
                uploads.count.first { it == 0 }
                // A start that is still on its way has a newer id, so it keeps the service and arms a new wait.
                stopSelf(lastStart)
            }
        }
        return START_NOT_STICKY
    }

    override fun onTimeout(startId: Int, fgsType: Int) = stopSelf()

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    private fun notification(): Notification {
        getSystemService(NotificationManager::class.java)
            .createNotificationChannel(NotificationChannel(CHANNEL, "Uploads", NotificationManager.IMPORTANCE_LOW))
        return NotificationCompat.Builder(this, CHANNEL)
            .setSmallIcon(android.R.drawable.stat_sys_upload)
            .setContentTitle("Saving to your mind")
            .setProgress(0, 0, true)
            .setOngoing(true)
            .setSilent(true)
            .build()
    }

    /** How the app starts the service. */
    companion object {
        private const val CHANNEL = "uploads"
        private const val NOTIFICATION_ID = 2

        /**
         * Starts the service for the uploads in flight; does nothing when Android does not let the app start it now.
         *
         * @param context Any context of the app.
         */
        fun start(context: Context) {
            try {
                ContextCompat.startForegroundService(context, Intent(context, UploadService::class.java))
            } catch (_: IllegalStateException) {
            }
        }
    }
}

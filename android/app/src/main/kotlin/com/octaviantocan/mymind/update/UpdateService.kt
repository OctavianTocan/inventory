package com.octaviantocan.mymind.update

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import com.octaviantocan.mymind.MymindApp
import com.octaviantocan.mymind.R
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

/** What the update's progress notification says. */
sealed interface UpdateProgress {
    /** Asking the server for the package. */
    data object Preparing : UpdateProgress

    /**
     * Downloading the package.
     *
     * @property percent Whole percent from 0 to 100.
     */
    data class Downloading(val percent: Int) : UpdateProgress

    /** The package is ready and waits for an upload in flight. */
    data object WaitingForUpload : UpdateProgress

    /** Android is installing the package. */
    data object Installing : UpdateProgress
}

/**
 * The notification's step for an update the user started.
 *
 * @param state The updater's state.
 * @param uploading True while an upload is in flight.
 * @return What the progress notification shows.
 */
fun updateProgress(state: UpdateState, uploading: Boolean): UpdateProgress = when (state) {
    is UpdateState.Downloading -> UpdateProgress.Downloading((state.progress * 100).toInt().coerceIn(0, 100))
    is UpdateState.Ready -> if (uploading) UpdateProgress.WaitingForUpload else UpdateProgress.Installing
    is UpdateState.Installing -> UpdateProgress.Installing
    else -> UpdateProgress.Preparing
}

/**
 * Keeps the app running, with a progress notification, while an update the user started downloads, waits for uploads
 * and is handed to Android, so leaving the app does not stop it. It stops itself when the run ends; Android then
 * reports the install to [InstallStatusReceiver], which asks for a tap in a notification when one is needed.
 */
class UpdateService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private var watcher: Job? = null
    private var lastStart = 0

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val container = (application as MymindApp).container
        val release = container.updater.offer.value
        ServiceCompat.startForeground(
            this,
            NOTIFICATION_ID,
            notification(release, updateProgress(container.updater.state.value, container.uploadActivity.count.value > 0)),
            ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
        )
        lastStart = startId
        if (watcher?.isActive != true) {
            val manager = getSystemService(NotificationManager::class.java)
            val progress = scope.launch {
                combine(container.updater.state, container.uploadActivity.count) { state, uploads -> updateProgress(state, uploads > 0) }
                    .distinctUntilChanged()
                    .collect { step -> manager.notify(NOTIFICATION_ID, notification(container.updater.offer.value, step)) }
            }
            watcher = scope.launch {
                container.updateInstaller.running.first { !it }
                progress.cancel()
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

    private fun notification(release: ReleaseInfo?, step: UpdateProgress): Notification {
        getSystemService(NotificationManager::class.java)
            .createNotificationChannel(NotificationChannel(CHANNEL, "Update progress", NotificationManager.IMPORTANCE_LOW))
        val name = listOfNotNull("Inventory", release?.versionName).joinToString(" ")
        val open = PendingIntent.getActivity(this, REQUEST_OPEN, openUpdateIntent(this), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val builder = NotificationCompat.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_notification_download)
            .setContentIntent(open)
            .setOngoing(true)
            .setSilent(true)
            .setOnlyAlertOnce(true)
        when (step) {
            UpdateProgress.Preparing -> builder.setContentTitle("Updating to $name").setProgress(0, 0, true)
            is UpdateProgress.Downloading -> builder.setContentTitle("Downloading $name").setContentText("${step.percent}%").setProgress(100, step.percent, false)
            UpdateProgress.WaitingForUpload -> builder.setContentTitle("Updating to $name").setContentText("Waiting for your upload to finish").setProgress(0, 0, true)
            UpdateProgress.Installing -> builder.setContentTitle("Installing $name").setProgress(0, 0, true)
        }
        return builder.build()
    }

    /** How the app starts the service. */
    companion object {
        private const val CHANNEL = "update-progress"
        private const val NOTIFICATION_ID = 3
        private const val REQUEST_OPEN = 3

        /**
         * Starts the service for the update that is starting; does nothing when Android does not let the app start it
         * now, and the update still runs while the app is on screen.
         *
         * @param context Any context of the app.
         */
        fun start(context: Context) {
            try {
                ContextCompat.startForegroundService(context, Intent(context, UpdateService::class.java))
            } catch (_: IllegalStateException) {
            }
        }
    }
}

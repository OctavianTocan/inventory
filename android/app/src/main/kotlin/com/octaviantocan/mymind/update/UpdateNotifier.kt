package com.octaviantocan.mymind.update

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat

/** Posts the notifications of the updater. */
class UpdateNotifier(private val context: Context) {
    /** @return True when the app may post notifications. */
    fun allowed(): Boolean =
        NotificationManagerCompat.from(context).areNotificationsEnabled() &&
            (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
                ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED)

    /**
     * Asks the user to let the app install updates, once, so [release] and later versions install by themselves.
     *
     * @param release The downloaded release that waits.
     */
    fun allowNeeded(release: ReleaseInfo) {
        val open = PendingIntent.getActivity(
            context,
            REQUEST_ALLOW,
            allowInstallsIntent(context),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        post(
            NotificationCompat.Builder(context, CHANNEL)
                .setSmallIcon(android.R.drawable.stat_sys_download_done)
                .setContentTitle("Allow mymind to install updates")
                .setContentText("mymind ${release.versionName} is ready. Allow it once and updates install by themselves.")
                .setContentIntent(open)
                .setAutoCancel(true),
        )
    }

    /**
     * Tells the user that Android wants a tap to finish the install.
     *
     * @param release The release being installed, or null when unknown.
     * @param prompt The system prompt's intent, opened by the tap.
     */
    fun promptNeeded(release: ReleaseInfo?, prompt: Intent) {
        val open = PendingIntent.getActivity(
            context,
            REQUEST_PROMPT,
            prompt.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        post(
            NotificationCompat.Builder(context, CHANNEL)
                .setSmallIcon(android.R.drawable.stat_sys_download_done)
                .setContentTitle("Finish installing mymind ${release?.versionName.orEmpty()}".trim())
                .setContentText("Tap to confirm with Android.")
                .setContentIntent(open)
                .setAutoCancel(true),
        )
    }

    /** Removes the update notification. */
    fun cancel() = NotificationManagerCompat.from(context).cancel(NOTIFICATION_ID)

    private fun post(builder: NotificationCompat.Builder) {
        if (!allowed()) return
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(NotificationChannel(CHANNEL, "App updates", NotificationManager.IMPORTANCE_DEFAULT))
        manager.notify(NOTIFICATION_ID, builder.build())
    }

    private companion object {
        const val CHANNEL = "updates"
        const val NOTIFICATION_ID = 1
        const val REQUEST_ALLOW = 1
        const val REQUEST_PROMPT = 2
    }
}

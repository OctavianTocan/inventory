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
     * Tells the user that [release] is downloaded, with its notes and an Install action.
     *
     * @param release The downloaded release.
     */
    fun ready(release: ReleaseInfo) {
        val install = PendingIntent.getBroadcast(
            context,
            REQUEST_INSTALL,
            Intent(context, InstallActionReceiver::class.java).setPackage(context.packageName),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        post(
            NotificationCompat.Builder(context, CHANNEL)
                .setSmallIcon(android.R.drawable.stat_sys_download_done)
                .setContentTitle("mymind ${release.versionName} is ready to install")
                .setContentText(release.notes.firstOrNull().orEmpty())
                .setStyle(NotificationCompat.BigTextStyle().bigText(release.notes.joinToString("\n") { "- $it" }))
                .setContentIntent(install)
                .addAction(0, "Install", install)
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
        const val REQUEST_INSTALL = 1
        const val REQUEST_PROMPT = 2
    }
}

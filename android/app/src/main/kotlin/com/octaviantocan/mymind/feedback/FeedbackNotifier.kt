package com.octaviantocan.mymind.feedback

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import androidx.core.app.NotificationCompat
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.data.isFinal
import com.octaviantocan.mymind.update.UpdateNotifier
import com.octaviantocan.mymind.update.openUpdateIntent

/**
 * Shows one notification per feedback job on the `feedback` channel: it is posted when the job starts and updated in
 * place as the job moves, alerting only the first time, and it ends in a final state that can be swiped away. Its
 * words are the server's `title` and `detail`.
 *
 * @param context Any context of the app.
 * @param allowed Whether the app may post notifications; false when POST_NOTIFICATIONS was denied, so nothing is posted.
 */
class FeedbackNotifier(private val context: Context, private val allowed: () -> Boolean = UpdateNotifier(context)::allowed) {
    /**
     * Posts or updates the job's notification; its tap opens the update screen when the job shipped, the app otherwise.
     *
     * @param status The job and its state, with the server's title and detail.
     */
    fun show(status: FeedbackStatus) {
        if (!allowed()) return
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        manager.createNotificationChannel(NotificationChannel(CHANNEL, "Feedback", NotificationManager.IMPORTANCE_DEFAULT))
        val open = PendingIntent.getActivity(
            context,
            id(status.id),
            openUpdateIntent(context),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val builder = NotificationCompat.Builder(context, CHANNEL)
            .setSmallIcon(R.drawable.ic_notification_upload)
            .setContentTitle(status.title.ifBlank { "Your feedback" })
            .setContentText(status.detail.ifBlank { null })
            .setContentIntent(open)
            .setOnlyAlertOnce(true)
            .setOngoing(!status.isFinal)
            .setAutoCancel(status.isFinal)
        manager.notify(id(status.id), builder.build())
    }

    companion object {
        /** Id of the channel every feedback notification uses. */
        const val CHANNEL = "feedback"

        /**
         * @param jobId A feedback job's id.
         * @return The id of its one notification.
         */
        fun id(jobId: String): Int = jobId.hashCode()
    }
}

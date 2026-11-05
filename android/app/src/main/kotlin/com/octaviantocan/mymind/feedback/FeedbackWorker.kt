package com.octaviantocan.mymind.feedback

import android.content.Context
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.octaviantocan.mymind.MymindApp
import java.util.concurrent.TimeUnit

/**
 * Background check of the feedback jobs: about every 5 minutes while any job is active, it asks the server where each
 * stands and updates its notification. It is one-time work that enqueues itself again only while a job is active.
 */
class FeedbackWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        container.session.restore()
        if (container.session.token == null) return Result.success()
        checkAndContinue(container.feedback) { schedule(applicationContext, appendToRunning = true) }
        return Result.success()
    }

    companion object {
        private const val NAME = "mymind-feedback-check"

        /** Minutes between two checks while a job is active. */
        const val INTERVAL_MINUTES = 5L

        /**
         * One check: asks about every active job and schedules the next check only while one is still active.
         *
         * @param tracker The jobs to ask about.
         * @param scheduleNext Enqueues the next check.
         * @return True when the next check was scheduled.
         */
        suspend fun checkAndContinue(tracker: FeedbackTracker, scheduleNext: () -> Unit): Boolean {
            val more = tracker.pollActive()
            if (more) scheduleNext()
            return more
        }

        /**
         * Enqueues the check [INTERVAL_MINUTES] from now as unique one-time work, keeping one that is already waiting.
         * Does nothing when WorkManager did not start, as in unit tests.
         *
         * @param context Any context of the app.
         * @param appendToRunning True when called by the running check itself, so the next one follows it.
         */
        fun schedule(context: Context, appendToRunning: Boolean = false) {
            if (!WorkManager.isInitialized()) return
            val request = OneTimeWorkRequestBuilder<FeedbackWorker>()
                .setInitialDelay(INTERVAL_MINUTES, TimeUnit.MINUTES)
                .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
                .build()
            val policy = when {
                appendToRunning -> ExistingWorkPolicy.APPEND_OR_REPLACE
                else -> ExistingWorkPolicy.KEEP
            }
            WorkManager.getInstance(context).enqueueUniqueWork(NAME, policy, request)
        }
    }
}

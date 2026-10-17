package com.octaviantocan.mymind.update

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.octaviantocan.mymind.MymindApp
import java.util.concurrent.TimeUnit

/** One-time job that installs a downloaded release once the app is off screen and idle. */
class UpdateInstallWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        if (container.updater.state.value !is UpdateState.Ready) {
            container.session.restore()
            if (container.session.token == null) return Result.success()
            container.updater.check()
        }
        return when (container.autoInstaller.installIfIdle()) {
            AutoInstallResult.Uploading -> Result.retry()
            else -> Result.success()
        }
    }

    companion object {
        private const val NAME = "mymind-update-install"

        /**
         * Schedules one install attempt, replacing a pending one. Does nothing when WorkManager did not start, as in unit tests.
         *
         * @param context Any context of the app.
         * @param delaySeconds How long to wait before the attempt.
         */
        fun schedule(context: Context, delaySeconds: Long) {
            if (!WorkManager.isInitialized()) return
            val request = OneTimeWorkRequestBuilder<UpdateInstallWorker>()
                .setInitialDelay(delaySeconds, TimeUnit.SECONDS)
                .setBackoffCriteria(BackoffPolicy.LINEAR, RETRY_SECONDS, TimeUnit.SECONDS)
                .build()
            WorkManager.getInstance(context).enqueueUniqueWork(NAME, ExistingWorkPolicy.REPLACE, request)
        }

        /** Seconds the app may be off screen before the install runs. */
        const val LEAVE_DELAY_SECONDS = 15L

        /** Seconds to wait before trying again after an upload held the install back. */
        const val RETRY_DELAY_SECONDS = 60L

        private const val RETRY_SECONDS = RETRY_DELAY_SECONDS
    }
}

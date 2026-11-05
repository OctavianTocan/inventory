package com.octaviantocan.mymind.update

import android.content.Context
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.octaviantocan.mymind.MymindApp
import java.util.concurrent.TimeUnit

/** Background check that downloads a newer release and installs it when the app is off screen and idle. */
class UpdateWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        container.session.restore()
        if (container.session.token == null) return Result.success()
        val state = container.updater.check()
        if (state is UpdateState.Ready && container.autoInstaller.installIfIdle() == AutoInstallResult.Uploading) {
            UpdateInstallWorker.schedule(applicationContext, UpdateInstallWorker.RETRY_DELAY_SECONDS)
        }
        return if (state is UpdateState.Failed) Result.retry() else Result.success()
    }

    companion object {
        private const val NAME = "mymind-update-check"

        /**
         * Schedules the check every 6 hours while the network is available; a schedule that exists is kept. Does nothing when WorkManager did not start, as in unit tests.
         *
         * @param context Any context of the app.
         */
        fun schedule(context: Context) {
            if (!WorkManager.isInitialized()) return
            val request = PeriodicWorkRequestBuilder<UpdateWorker>(6, TimeUnit.HOURS)
                .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
                .build()
            WorkManager.getInstance(context).enqueueUniquePeriodicWork(NAME, ExistingPeriodicWorkPolicy.KEEP, request)
        }

        /**
         * Runs the same check once, now, such as when a feedback ships a release. Does nothing when WorkManager did not start.
         *
         * @param context Any context of the app.
         */
        fun checkNow(context: Context) {
            if (!WorkManager.isInitialized()) return
            val request = OneTimeWorkRequestBuilder<UpdateWorker>()
                .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
                .build()
            WorkManager.getInstance(context).enqueueUniqueWork("$NAME-now", ExistingWorkPolicy.REPLACE, request)
        }
    }
}

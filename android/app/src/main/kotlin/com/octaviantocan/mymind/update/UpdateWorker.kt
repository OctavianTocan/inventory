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

/**
 * Background check that downloads a newer release, so Update on the update screen can install it at once, and tells
 * the user once per version while the app is closed. It never installs: the update screen does, when the app opens.
 */
class UpdateWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        container.session.restore()
        if (container.session.token == null) return Result.success()
        val state = container.updater.check()
        val offer = container.updater.offer.value
        if (offer != null && !container.updater.foreground && offer.versionCode > container.updatePrefs.notifiedCode) {
            container.updatePrefs.notifiedCode = offer.versionCode
            container.updateNotifier.ready(offer)
        }
        return if (state is UpdateState.Failed) Result.retry() else Result.success()
    }

    companion object {
        private const val NAME = "mymind-update-check"

        // Older builds queued this job to install on leaving the app; its class is gone.
        private const val LEAVE_INSTALL_NAME = "mymind-update-install"

        /**
         * Cancels the install-on-leave job an older build may have queued. Does nothing when WorkManager did not start.
         *
         * @param context Any context of the app.
         */
        fun cancelLeaveInstall(context: Context) {
            if (!WorkManager.isInitialized()) return
            WorkManager.getInstance(context).cancelUniqueWork(LEAVE_INSTALL_NAME)
        }

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

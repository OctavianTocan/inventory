package com.octaviantocan.mymind.update

import android.content.Context
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.octaviantocan.mymind.MymindApp
import java.util.concurrent.TimeUnit

/** Background check that downloads a newer release and notifies the user when the app is not on screen. */
class UpdateWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        container.session.restore()
        if (container.session.token == null) return Result.success()
        val state = container.updater.check()
        if (state is UpdateState.Ready && !container.updater.foreground) container.updateNotifier.ready(state.release)
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
    }
}

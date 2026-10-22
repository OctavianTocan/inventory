package com.octaviantocan.mymind.data

import android.content.Context
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import coil3.SingletonImageLoader
import coil3.request.CachePolicy
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.MymindApp
import com.octaviantocan.mymind.ui.library.galleryPicture

// Pictures are fetched for the disk cache; decoding them this small keeps the check that they are images cheap.
private const val FETCH_DECODE_PX = 64

/** Sends the changes waiting on the phone once it is online, even with the app closed, then syncs the copy. */
class OutboxWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        container.session.restore()
        if (container.session.token == null) return Result.success()
        return if (container.librarySync.sync() is SyncOutcome.Failed && !container.outbox.isEmpty()) Result.retry() else Result.success()
    }

    companion object {
        private const val NAME = "mymind-outbox"

        /**
         * Sends the outbox the next time the phone is online; a run already waiting is kept. Does nothing when
         * WorkManager did not start, as in unit tests.
         *
         * @param context Any context of the app.
         */
        fun schedule(context: Context) {
            if (!WorkManager.isInitialized()) return
            val request = OneTimeWorkRequestBuilder<OutboxWorker>()
                .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
                .build()
            WorkManager.getInstance(context).enqueueUniqueWork(NAME, ExistingWorkPolicy.KEEP, request)
        }

        /** @param context Any context of the app; drops a waiting run, as at sign-out. */
        fun cancel(context: Context) {
            if (WorkManager.isInitialized()) WorkManager.getInstance(context).cancelUniqueWork(NAME)
        }
    }
}

/** Fetches the pictures of the cards kept for use offline ([KeptPictures]) while the phone is on Wi-Fi. */
class PictureWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val container = (applicationContext as MymindApp).container
        container.session.restore()
        if (container.session.token == null) return Result.success()
        val loader = SingletonImageLoader.get(applicationContext)
        val pictures = KeptPictures(
            container.libraryCopy,
            pictureOf = { card -> galleryPicture(card)?.let { resolveMediaUrl(BuildConfig.SERVER_URL, it) } },
            kept = { url -> loader.diskCache?.openSnapshot(url)?.use { true } ?: false },
            fetch = { url ->
                val request = ImageRequest.Builder(applicationContext).data(url).memoryCachePolicy(CachePolicy.DISABLED).size(FETCH_DECODE_PX).build()
                loader.execute(request) is SuccessResult
            },
        )
        pictures.fetchMissing { isStopped }
        return Result.success()
    }

    companion object {
        private const val NAME = "mymind-kept-pictures"

        /**
         * Fetches the kept pictures the next time the phone is on Wi-Fi with battery to spare; a run already waiting
         * or running is kept. Does nothing when WorkManager did not start, as in unit tests.
         *
         * @param context Any context of the app.
         */
        fun schedule(context: Context) {
            if (!WorkManager.isInitialized()) return
            val constraints = Constraints.Builder().setRequiredNetworkType(NetworkType.UNMETERED).setRequiresBatteryNotLow(true).build()
            val request = OneTimeWorkRequestBuilder<PictureWorker>().setConstraints(constraints).build()
            WorkManager.getInstance(context).enqueueUniqueWork(NAME, ExistingWorkPolicy.KEEP, request)
        }

        /** @param context Any context of the app; stops a run and drops a waiting one, as at sign-out. */
        fun cancel(context: Context) {
            if (WorkManager.isInitialized()) WorkManager.getInstance(context).cancelUniqueWork(NAME)
        }
    }
}

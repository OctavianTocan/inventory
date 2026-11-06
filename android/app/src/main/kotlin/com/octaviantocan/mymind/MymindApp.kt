package com.octaviantocan.mymind

import android.app.Application
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ProcessLifecycleOwner
import com.octaviantocan.mymind.data.PictureWorker
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.feedback.FeedbackWorker
import com.octaviantocan.mymind.update.ForegroundTracker
import com.octaviantocan.mymind.update.UpdateWorker
import okio.Path.Companion.toOkioPath

// Room for the pictures kept for use offline as well as those seen lately: 5% of free space, 64 to 512 MB.
private const val PICTURE_CACHE_SHARE = 0.05
private const val PICTURE_CACHE_MIN_BYTES = 64L * 1024 * 1024
private const val PICTURE_CACHE_MAX_BYTES = 512L * 1024 * 1024

class MymindApp : Application(), SingletonImageLoader.Factory {
    val container by lazy { AppContainer(this) }

    override fun onCreate() {
        super.onCreate()
        UpdateWorker.schedule(this)
        if (container.feedbackStore.hasActive()) FeedbackWorker.schedule(this)
        PictureWorker.schedule(this)
        UpdateWorker.cancelLeaveInstall(this)
        ProcessLifecycleOwner.get().lifecycle.addObserver(ForegroundTracker(container.updater))
        ProcessLifecycleOwner.get().lifecycle.addObserver(object : DefaultLifecycleObserver {
            override fun onStart(owner: LifecycleOwner) = Track.event("start")

            override fun onStop(owner: LifecycleOwner) = container.sendEvents()
        })
    }

    override fun newImageLoader(context: PlatformContext): ImageLoader =
        ImageLoader.Builder(context)
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { container.httpClient })) }
            .diskCache {
                DiskCache.Builder()
                    .directory(cacheDir.resolve("image_cache").toOkioPath())
                    .maxSizePercent(PICTURE_CACHE_SHARE)
                    .minimumMaxSizeBytes(PICTURE_CACHE_MIN_BYTES)
                    .maximumMaxSizeBytes(PICTURE_CACHE_MAX_BYTES)
                    .build()
            }
            .build()
}

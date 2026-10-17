package com.octaviantocan.mymind

import android.app.Application
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import androidx.lifecycle.ProcessLifecycleOwner
import com.octaviantocan.mymind.update.ForegroundTracker
import com.octaviantocan.mymind.update.UpdateInstallWorker
import com.octaviantocan.mymind.update.UpdateState
import com.octaviantocan.mymind.update.UpdateWorker

class MymindApp : Application(), SingletonImageLoader.Factory {
    val container by lazy { AppContainer(this) }

    override fun onCreate() {
        super.onCreate()
        UpdateWorker.schedule(this)
        val updater = container.updater
        ProcessLifecycleOwner.get().lifecycle.addObserver(
            ForegroundTracker(updater) {
                if (updater.state.value is UpdateState.Ready) UpdateInstallWorker.schedule(this, UpdateInstallWorker.LEAVE_DELAY_SECONDS)
            },
        )
    }

    override fun newImageLoader(context: PlatformContext): ImageLoader =
        ImageLoader.Builder(context)
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { container.httpClient })) }
            .build()
}

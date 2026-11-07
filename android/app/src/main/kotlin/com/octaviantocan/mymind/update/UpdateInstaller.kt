package com.octaviantocan.mymind.update

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

/** What [UpdateInstaller.update] did. */
enum class UpdateStart {
    /** The install was handed to Android. */
    Started,

    /** Android has not let the app install packages yet; nothing was downloaded or installed. */
    NeedsAllow,

    /** No newer release could be downloaded; the updater's state says why. */
    NothingToInstall,
}

/**
 * Runs the update the update screen asks for: downloads the offered release when it is not on the phone yet, waits for
 * the uploads in flight so the install does not cut them off, then hands the package to Android.
 *
 * @param updater The shared updater.
 * @param canInstall Whether Android lets this app install packages right now.
 * @param uploads Tells whether an upload is in flight.
 * @param scope Where [start] runs the update; it outlives the screens, so leaving the app does not cancel it.
 * @param onRun Called as [start] begins a run, to keep the app alive with a notification while it lasts.
 */
class UpdateInstaller(
    private val updater: Updater,
    private val canInstall: () -> Boolean,
    private val uploads: UploadActivity,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default),
    private val onRun: () -> Unit = {},
) {
    private val mutableRunning = MutableStateFlow(false)

    /** True while a run [start] began is downloading, waiting for uploads or handing the package to Android. */
    val running: StateFlow<Boolean> = mutableRunning

    /**
     * Runs [update] in the installer's own scope, so it carries on when the user leaves the app. A second call while a
     * run is going does nothing.
     *
     * @param onDone Called with what the run did.
     * @return False when a run was already going.
     */
    fun start(onDone: (UpdateStart) -> Unit = {}): Boolean {
        if (!mutableRunning.compareAndSet(expect = false, update = true)) return false
        onRun()
        scope.launch {
            val result = try {
                update()
            } finally {
                mutableRunning.value = false
            }
            onDone(result)
        }
        return true
    }

    /**
     * Installs the newest release, downloading it first when needed and waiting while an upload is in flight.
     *
     * @return What happened.
     */
    suspend fun update(): UpdateStart {
        if (!canInstall()) return UpdateStart.NeedsAllow
        if (updater.state.value !is UpdateState.Ready) updater.check()
        if (updater.state.value !is UpdateState.Ready) return UpdateStart.NothingToInstall
        uploads.count.first { it == 0 }
        updater.install()
        return UpdateStart.Started
    }
}

package com.octaviantocan.mymind.update

import kotlinx.coroutines.flow.first

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
 */
class UpdateInstaller(
    private val updater: Updater,
    private val canInstall: () -> Boolean,
    private val uploads: UploadActivity,
) {
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

package com.octaviantocan.mymind.update

/** What [AutoInstaller.installIfIdle] did. */
enum class AutoInstallResult {
    /** No verified release is waiting. */
    NothingToInstall,

    /** The install was handed to Android. */
    Started,

    /** The app is on screen; the install waits for it to leave. */
    OnScreen,

    /** An upload is in flight; the install waits for it. */
    Uploading,

    /** Android has not let the app install packages; the user was asked. */
    NeedsAllow,
}

/**
 * Installs a verified release by itself when nothing would be interrupted.
 *
 * @param updater The shared updater.
 * @param canInstall Whether Android lets this app install packages right now.
 * @param uploads Tells whether an upload is in flight.
 * @param askToAllow Asks the user, outside the app, to allow installs; called for a release that waits for the permission.
 */
class AutoInstaller(
    private val updater: Updater,
    private val canInstall: () -> Boolean,
    private val uploads: UploadActivity,
    private val askToAllow: (ReleaseInfo) -> Unit,
) {
    /**
     * Installs the ready release when the app is off screen, no upload is in flight and installs are allowed.
     *
     * @return What happened.
     */
    suspend fun installIfIdle(): AutoInstallResult {
        val ready = updater.state.value as? UpdateState.Ready ?: return AutoInstallResult.NothingToInstall
        if (updater.foreground) return AutoInstallResult.OnScreen
        if (uploads.busy()) return AutoInstallResult.Uploading
        if (!canInstall()) {
            askToAllow(ready.release)
            return AutoInstallResult.NeedsAllow
        }
        updater.install()
        return AutoInstallResult.Started
    }

    /**
     * Installs the ready release now, for a tap in Settings; the app is on screen.
     *
     * @return True when the install started; false with no release, no permission or an upload in flight.
     */
    suspend fun installNow(): Boolean {
        if (updater.state.value !is UpdateState.Ready || uploads.busy() || !canInstall()) return false
        updater.install()
        return true
    }
}

package com.octaviantocan.mymind.ui.update

import android.content.Intent
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.update.ReleaseInfo
import com.octaviantocan.mymind.update.UpdateInstaller
import com.octaviantocan.mymind.update.UpdateStart
import com.octaviantocan.mymind.update.UpdateState
import com.octaviantocan.mymind.update.Updater
import com.octaviantocan.mymind.update.WhatsNew
import com.octaviantocan.mymind.update.isNewer
import com.octaviantocan.mymind.update.updateStatusText
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** How far the update screen's Update got. */
sealed interface UpdateStep {
    /** Nothing started: Update is offered. */
    data object Idle : UpdateStep

    /** Android does not let the app install yet; its settings page was opened and Update is offered again. */
    data object NeedsAllow : UpdateStep

    /** Asking the server for the package before downloading it. */
    data object Preparing : UpdateStep

    /**
     * Downloading the package after Update was tapped.
     *
     * @property progress Fraction from 0 to 1.
     */
    data class Downloading(val progress: Float) : UpdateStep

    /** The package is ready and waits for an upload in flight to finish. */
    data object WaitingForUpload : UpdateStep

    /** Android is installing the package. */
    data object Installing : UpdateStep

    /** Android waits for the user to approve the install; Continue update opens its prompt again. */
    data object Confirm : UpdateStep

    /**
     * The check, download or install failed; Try again starts over.
     *
     * @property reason Short wording of the cause.
     */
    data class Failed(val reason: String) : UpdateStep
}

/**
 * What the update screen shows.
 *
 * @property release The newer release the server offers.
 * @property entries What it brings over the installed version, newest first.
 * @property step How far the update got.
 */
data class UpdateGateModel(val release: ReleaseInfo, val entries: List<ChangelogEntry>, val step: UpdateStep)

/**
 * What the update screen's taps started.
 *
 * @property running True while a tap's download and install run.
 * @property needsAllow True after a tap found that Android does not let the app install yet.
 */
private data class UpdatePress(val running: Boolean = false, val needsAllow: Boolean = false)

/**
 * The versions [release] brings over the installed one, newest first, from the server's release description.
 *
 * @param release The offered release.
 * @param installedCode `versionCode` of the running app.
 * @return One changelog entry per newer version; the release's own notes when the description has no history.
 */
fun offeredEntries(release: ReleaseInfo, installedCode: Int): List<ChangelogEntry> {
    val newer = release.history.filter { isNewer(it.versionCode, installedCode) }.sortedByDescending { it.versionCode }
    if (newer.isEmpty()) return listOf(ChangelogEntry(release.versionName, release.versionCode, release.built.take(10), release.notes))
    return newer.map { ChangelogEntry(it.versionName, it.versionCode, it.date, it.notes) }
}

/**
 * State and actions of the update screens.
 *
 * @param updater The shared updater.
 * @param installer Downloads, waits for uploads and installs when Update is tapped.
 * @param whatsNew The bundled changelog.
 * @param uploads Number of uploads in flight.
 * @param canInstallPackages Whether Android lets this app install packages right now.
 * @param installedCode `versionCode` of the running app.
 * @param versionLabel Installed version shown in Settings, such as `Version 0.2.0 (2)`.
 */
class UpdateViewModel(
    private val updater: Updater,
    private val installer: UpdateInstaller,
    private val whatsNew: WhatsNew,
    uploads: StateFlow<Int>,
    private val canInstallPackages: () -> Boolean,
    private val installedCode: Int,
    private val versionLabel: String,
) : ViewModel() {
    private val installAllowed = MutableStateFlow(canInstallPackages())
    private val press = MutableStateFlow(UpdatePress())
    private var started = false

    /** True while the changelog sheet is open. */
    val changelogOpen = MutableStateFlow(false)

    /** The notes of the version the app was just updated to, until the user closes them. */
    val afterUpdate = MutableStateFlow<ChangelogEntry?>(whatsNew.takePending())

    /** Every version of the bundled changelog, newest first. */
    val entries: List<ChangelogEntry> get() = whatsNew.entries

    /** The update screen, shown over everything while the server offers a newer release; null otherwise. */
    val gate: StateFlow<UpdateGateModel?> =
        combine(updater.offer, updater.state, updater.prompt, uploads, press) { offer, state, prompt, uploading, pressed ->
            offer?.let { UpdateGateModel(it, offeredEntries(it, installedCode), step(state, prompt != null, uploading > 0, pressed)) }
        }.stateIn(viewModelScope, SharingStarted.Eagerly, updater.offer.value?.let { UpdateGateModel(it, offeredEntries(it, installedCode), UpdateStep.Idle) })

    /** Rows the settings drawer adds for updates. */
    val settings: StateFlow<SettingsUpdates> = combine(updater.state, installAllowed) { state, allowed ->
        SettingsUpdates(
            visible = true,
            versionLabel = versionLabel,
            status = updateStatusText(state, allowed),
            showAllow = !allowed && (state is UpdateState.Ready || state is UpdateState.Failed),
            onCheck = ::checkNow,
            onChangelog = { changelogOpen.value = true },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, SettingsUpdates())

    private fun step(state: UpdateState, prompting: Boolean, uploading: Boolean, pressed: UpdatePress): UpdateStep = when {
        state is UpdateState.Installing -> if (prompting) UpdateStep.Confirm else UpdateStep.Installing
        pressed.running && state is UpdateState.Downloading -> UpdateStep.Downloading(state.progress)
        pressed.running && state is UpdateState.Ready && uploading -> UpdateStep.WaitingForUpload
        pressed.running -> UpdateStep.Preparing
        state is UpdateState.Failed -> UpdateStep.Failed(state.reason)
        pressed.needsAllow -> UpdateStep.NeedsAllow
        else -> UpdateStep.Idle
    }

    /**
     * Called each time the app comes to the front: checks at once the first time, then when the last check is a few
     * minutes old. Back from Android's install settings with installs now allowed, the update carries on by itself.
     */
    fun onResume() {
        val allowed = canInstallPackages()
        installAllowed.value = allowed
        if (allowed && press.value.needsAllow) {
            update()
            return
        }
        viewModelScope.launch {
            if (!started) {
                started = true
                updater.check()
            } else {
                updater.checkIfStale()
            }
        }
    }

    /**
     * Starts the update for the screen's Update or Try again: downloads the offered release when needed, waits for
     * uploads in flight, then hands it to Android. Its notes count as read, so they do not open again after the install.
     *
     * @return False when Android does not let the app install yet; the caller opens its settings page.
     */
    fun update(): Boolean {
        if (press.value.running) return true
        if (!canInstallPackages()) {
            press.value = UpdatePress(needsAllow = true)
            return false
        }
        updater.offer.value?.let { whatsNew.markShown(it.versionCode) }
        press.value = UpdatePress(running = true)
        viewModelScope.launch {
            val start = try {
                installer.update()
            } finally {
                press.update { it.copy(running = false) }
            }
            if (start == UpdateStart.NeedsAllow) press.value = UpdatePress(needsAllow = true)
        }
        return true
    }

    /** @return Android's waiting install prompt, for Continue update, or null. */
    fun pendingPrompt(): Intent? = updater.prompt.value

    /** Asks the server again, for "Check for updates" in Settings; a newer release brings up the update screen. */
    fun checkNow() {
        viewModelScope.launch {
            installAllowed.value = canInstallPackages()
            updater.check()
        }
    }

    /** Closes the notes shown after an update. */
    fun dismissAfterUpdate() {
        afterUpdate.value = null
    }

    /** Closes the changelog sheet. */
    fun closeChangelog() {
        changelogOpen.value = false
    }
}

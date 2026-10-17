package com.octaviantocan.mymind.ui.update

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.update.ReleaseInfo
import com.octaviantocan.mymind.update.UpdateState
import com.octaviantocan.mymind.update.Updater
import com.octaviantocan.mymind.update.WhatsNew
import com.octaviantocan.mymind.update.updateStatusText
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/** How far the update has come, for the "What's new" sheet. */
enum class UpdatePhase { Downloading, Ready, Installing, Failed }

/**
 * The foreground update sheet.
 *
 * @property release The release the sheet announces.
 * @property phase Where the update is.
 * @property progress Download fraction from 0 to 1, meaningful while downloading.
 * @property installAllowed False when Android has not yet let this app install packages.
 * @property failure Reason of the last failure, or null.
 */
data class UpdateSheetModel(
    val release: ReleaseInfo,
    val phase: UpdatePhase,
    val progress: Float = 0f,
    val installAllowed: Boolean = true,
    val failure: String? = null,
)

/**
 * State and actions of the update screens.
 *
 * @param updater The shared updater.
 * @param whatsNew The bundled changelog.
 * @param canInstallPackages Whether Android lets this app install packages right now.
 * @param versionLabel Installed version shown in Settings, such as `Version 0.2.0 (2)`.
 */
class UpdateViewModel(
    private val updater: Updater,
    private val whatsNew: WhatsNew,
    private val canInstallPackages: () -> Boolean,
    private val versionLabel: String,
) : ViewModel() {
    private val later = MutableStateFlow<Int?>(null)
    private val installAllowed = MutableStateFlow(canInstallPackages())
    private var started = false

    /** True while the full changelog screen is open. */
    val changelogOpen = MutableStateFlow(false)

    /** The notes of the version the app was just updated to, until the user closes them. */
    val afterUpdate = MutableStateFlow<ChangelogEntry?>(whatsNew.takePending())

    /** Every version of the bundled changelog, newest first. */
    val entries: List<ChangelogEntry> get() = whatsNew.entries

    /** The sheet to show over the app, or null. */
    val sheet: StateFlow<UpdateSheetModel?> = combine(updater.state, later, installAllowed) { state, dismissed, allowed ->
        val model = when (state) {
            is UpdateState.Downloading -> UpdateSheetModel(state.release, UpdatePhase.Downloading, state.progress, allowed)
            is UpdateState.Ready -> UpdateSheetModel(state.release, UpdatePhase.Ready, 1f, allowed)
            is UpdateState.Installing -> UpdateSheetModel(state.release, UpdatePhase.Installing, 1f, allowed)
            is UpdateState.Failed -> state.release?.let { UpdateSheetModel(it, UpdatePhase.Failed, 0f, allowed, state.reason) }
            else -> null
        }
        model?.takeIf { it.release.versionCode != dismissed }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    /** Rows the settings drawer adds for updates. */
    val settings: StateFlow<SettingsUpdates> = combine(updater.state, installAllowed) { state, allowed ->
        SettingsUpdates(
            visible = true,
            versionLabel = versionLabel,
            status = updateStatusText(state),
            showAllow = !allowed && (state is UpdateState.Ready || state is UpdateState.Failed),
            onCheck = ::checkNow,
            onChangelog = { changelogOpen.value = true },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, SettingsUpdates())

    /** Called each time the app comes to the front: checks at once the first time, then when the last check is old. */
    fun onResume() {
        installAllowed.value = canInstallPackages()
        viewModelScope.launch {
            if (!started) {
                started = true
                updater.check()
            } else {
                updater.checkIfStale()
            }
        }
    }

    /** Checks now and shows the sheet again if a release is waiting. */
    fun checkNow() {
        later.value = null
        viewModelScope.launch { updater.check() }
    }

    /** Installs the downloaded release. */
    fun install() {
        viewModelScope.launch { updater.install() }
    }

    /** Retries after a failure. */
    fun retry() = checkNow()

    /** Hides the sheet until the next launch. */
    fun later() {
        later.value = updater.currentRelease()?.versionCode
    }

    /** Closes the notes shown after an update. */
    fun dismissAfterUpdate() {
        afterUpdate.value = null
    }

    /** Closes the full changelog screen. */
    fun closeChangelog() {
        changelogOpen.value = false
    }
}

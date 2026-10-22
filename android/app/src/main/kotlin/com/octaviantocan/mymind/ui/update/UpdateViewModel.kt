package com.octaviantocan.mymind.ui.update

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.update.AutoInstaller
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

/**
 * The request to let the app install updates.
 *
 * @property release The downloaded release that waits for the permission.
 */
data class AllowRequestModel(val release: ReleaseInfo)

/**
 * State and actions of the update screens.
 *
 * @param updater The shared updater.
 * @param autoInstaller Installs a ready release when the user taps for it.
 * @param whatsNew The bundled changelog.
 * @param canInstallPackages Whether Android lets this app install packages right now.
 * @param versionLabel Installed version shown in Settings, such as `Version 0.2.0 (2)`.
 */
class UpdateViewModel(
    private val updater: Updater,
    private val autoInstaller: AutoInstaller,
    private val whatsNew: WhatsNew,
    private val canInstallPackages: () -> Boolean,
    private val versionLabel: String,
) : ViewModel() {
    private val later = MutableStateFlow<Int?>(null)
    private val installAllowed = MutableStateFlow(canInstallPackages())
    private var started = false

    /** True while the changelog sheet is open. */
    val changelogOpen = MutableStateFlow(false)

    /** The notes of the version the app was just updated to, until the user closes them. */
    val afterUpdate = MutableStateFlow<ChangelogEntry?>(whatsNew.takePending())

    /** Every version of the bundled changelog, newest first. */
    val entries: List<ChangelogEntry> get() = whatsNew.entries

    /** The request to allow installs, shown while a release waits for the permission; null otherwise. */
    val sheet: StateFlow<AllowRequestModel?> = combine(updater.state, later, installAllowed) { state, dismissed, allowed ->
        (state as? UpdateState.Ready)?.takeIf { !allowed && it.release.versionCode != dismissed }?.let { AllowRequestModel(it.release) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

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

    /** Installs a ready release at once when allowed, otherwise checks again and shows the allow request if needed. */
    fun checkNow() {
        later.value = null
        viewModelScope.launch {
            installAllowed.value = canInstallPackages()
            if (!autoInstaller.installNow()) updater.check()
        }
    }

    /** Hides the allow request until the next launch. */
    fun later() {
        later.value = updater.currentRelease()?.versionCode
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

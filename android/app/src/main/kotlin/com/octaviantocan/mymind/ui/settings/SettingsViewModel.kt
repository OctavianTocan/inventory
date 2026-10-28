package com.octaviantocan.mymind.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Repository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

/**
 * What the profile sheet shows about the account.
 *
 * @property email Email address of the signed-in account, or null until the server has answered.
 * @property photoUrl Session-bound Google photo, or null when the account has no cached photo.
 * @property displayName Session-bound Google name, or null.
 */
data class SettingsUiState(val email: String? = null, val photoUrl: String? = null, val displayName: String? = null)

/**
 * State and actions of the profile sheet.
 *
 * @param repository Source of the account and the sign-out action.
 */
class SettingsViewModel(private val repository: Repository) : ViewModel() {
    private val mutableState = MutableStateFlow(SettingsUiState())

    /** Account shown in the profile sheet. */
    val state: StateFlow<SettingsUiState> = mutableState

    /**
     * Shows what this session keeps at once, so the photo is there as the sheet opens and offline, then the server's
     * answer. Another session's account is never shown: one with nothing kept starts empty.
     */
    fun refresh() {
        mutableState.value = repository.savedAccount()?.let { SettingsUiState(it.email, it.photoUrl, it.displayName) } ?: SettingsUiState()
        viewModelScope.launch {
            when (val answer = repository.me()) {
                is ApiResult.Ok -> mutableState.value = SettingsUiState(answer.value.email, answer.value.photoUrl, answer.value.displayName)
                is ApiResult.Failed -> if (answer.failure == ApiFailure.Unauthorized || answer.failure == ApiFailure.InvalidToken) {
                    mutableState.value = SettingsUiState()
                }
            }
        }
    }

    /** Signs the account out of this device. */
    fun signOut() {
        mutableState.value = SettingsUiState()
        viewModelScope.launch { repository.signOut() }
    }
}

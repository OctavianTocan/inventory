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

    /** Replaces prior display metadata only after the current account has been read successfully. */
    fun refresh() {
        mutableState.value = SettingsUiState()
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

package com.octaviantocan.mymind.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Repository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

/**
 * What the settings drawer shows about the account.
 *
 * @property email Email address of the signed-in account, or null until the server has answered.
 */
data class SettingsUiState(val email: String? = null)

/**
 * State and actions of the settings drawer.
 *
 * @param repository Source of the account and the sign-out action.
 */
class SettingsViewModel(private val repository: Repository) : ViewModel() {
    private val mutableState = MutableStateFlow(SettingsUiState())

    /** Account shown in the drawer. */
    val state: StateFlow<SettingsUiState> = mutableState

    /** Asks the server for the signed-in account; the shown account stays as it was when the request fails. */
    fun refresh() {
        viewModelScope.launch {
            val answer = repository.me() as? ApiResult.Ok ?: return@launch
            mutableState.value = SettingsUiState(email = answer.value.email)
        }
    }

    /** Signs the account out of this device. */
    fun signOut() {
        viewModelScope.launch { repository.signOut() }
    }
}

package com.octaviantocan.mymind.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.ConnectedAgent
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
 * The Connected agents page: the AI apps signed in to the account through ThyMind's MCP server.
 *
 * @property agents The apps, most recently used first, or null until the server has answered.
 * @property failed True when the last request did not reach the server or was refused.
 */
data class AgentsUiState(val agents: List<ConnectedAgent>? = null, val failed: Boolean = false)

/**
 * State and actions of the profile sheet.
 *
 * @param repository Source of the account and the sign-out action.
 */
class SettingsViewModel(private val repository: Repository) : ViewModel() {
    private val mutableState = MutableStateFlow(SettingsUiState())

    /** Account shown in the profile sheet. */
    val state: StateFlow<SettingsUiState> = mutableState

    private val mutableAgents = MutableStateFlow(AgentsUiState())

    /** The account's connected AI apps. */
    val agents: StateFlow<AgentsUiState> = mutableAgents

    /** Asks the server for the connected AI apps, keeping the list shown until it answers. */
    fun loadAgents() {
        viewModelScope.launch {
            mutableAgents.value = when (val answer = repository.agents()) {
                is ApiResult.Ok -> AgentsUiState(answer.value.agents)
                is ApiResult.Failed -> mutableAgents.value.copy(failed = true)
            }
        }
    }

    /**
     * Signs the AI app [id] out: it leaves the list at once, and the list is read again if the server refuses.
     *
     * @param id The agent's [ConnectedAgent.id].
     */
    fun removeAgent(id: String) {
        val before = mutableAgents.value
        mutableAgents.value = before.copy(agents = before.agents?.filterNot { it.id == id }, failed = false)
        viewModelScope.launch {
            if (repository.removeAgent(id) is ApiResult.Failed) {
                mutableAgents.value = before.copy(failed = true)
                loadAgents()
            }
        }
    }

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
        mutableAgents.value = AgentsUiState()
        viewModelScope.launch { repository.signOut() }
    }
}

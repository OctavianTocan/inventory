package com.octaviantocan.mymind.ui

import android.content.Context
import androidx.credentials.CredentialManager
import androidx.credentials.CustomCredential
import androidx.credentials.GetCredentialRequest
import androidx.credentials.exceptions.GetCredentialCancellationException
import androidx.credentials.exceptions.GetCredentialException
import androidx.credentials.exceptions.NoCredentialException
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.google.android.libraries.identity.googleid.GetGoogleIdOption
import com.google.android.libraries.identity.googleid.GoogleIdTokenCredential
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.SignInOutcome
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class SignInUiState(val busy: Boolean = false, val message: String? = null)

class SignInViewModel(private val repository: Repository) : ViewModel() {
    private val mutableState = MutableStateFlow(SignInUiState())
    val state: StateFlow<SignInUiState> = mutableState

    fun signInWithGoogle(context: Context) = run {
        repository.signInWithGoogle { nonce -> requestGoogleIdToken(context, nonce) }
    }

    fun signInWithPassword(username: String, password: String) = run {
        repository.signInWithPassword(username.trim(), password)
    }

    /** Clears the failure message once the user has seen it. */
    fun dismissMessage() = mutableState.update { it.copy(message = null) }

    private fun run(call: suspend () -> SignInOutcome) {
        if (mutableState.value.busy) return
        mutableState.value = SignInUiState(busy = true)
        viewModelScope.launch {
            val message = try {
                when (val outcome = call()) {
                    is SignInOutcome.Failed -> describe(outcome.failure)
                    SignInOutcome.Cancelled, SignInOutcome.Success -> null
                }
            } catch (_: NoCredentialException) {
                "No Google account on this phone. Add one in Settings, or sign in with a password."
            } catch (e: GetCredentialException) {
                "Google sign-in failed: ${e.message ?: e.type}"
            } catch (e: IllegalStateException) {
                e.message
            }
            mutableState.update { SignInUiState(busy = false, message = message) }
        }
    }

    private suspend fun requestGoogleIdToken(context: Context, nonce: String): String? {
        if (BuildConfig.GOOGLE_WEB_CLIENT_ID.isEmpty()) {
            throw IllegalStateException("Google sign-in is not configured in this build.")
        }
        val option = GetGoogleIdOption.Builder()
            .setFilterByAuthorizedAccounts(false)
            .setServerClientId(BuildConfig.GOOGLE_WEB_CLIENT_ID)
            .setNonce(nonce)
            .build()
        val request = GetCredentialRequest.Builder().addCredentialOption(option).build()
        return try {
            val credential = CredentialManager.create(context).getCredential(context, request).credential
            if (credential is CustomCredential && credential.type == GoogleIdTokenCredential.TYPE_GOOGLE_ID_TOKEN_CREDENTIAL) {
                GoogleIdTokenCredential.createFrom(credential.data).idToken
            } else {
                null
            }
        } catch (_: GetCredentialCancellationException) {
            null
        }
    }
}

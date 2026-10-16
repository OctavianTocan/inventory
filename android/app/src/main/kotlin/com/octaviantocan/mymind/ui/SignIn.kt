package com.octaviantocan.mymind.ui

import android.content.Context
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.credentials.CredentialManager
import androidx.credentials.CustomCredential
import androidx.credentials.GetCredentialRequest
import androidx.credentials.exceptions.GetCredentialCancellationException
import androidx.credentials.exceptions.GetCredentialException
import androidx.credentials.exceptions.NoCredentialException
import androidx.lifecycle.ViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
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

@Composable
fun SignInScreen(viewModel: SignInViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val context = LocalContext.current
    var username by rememberSaveable { mutableStateOf("") }
    var password by rememberSaveable { mutableStateOf("") }
    val canSubmit = !state.busy && username.isNotBlank() && password.isNotEmpty()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .safeDrawingPadding()
            .verticalScroll(rememberScrollState())
            .padding(24.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        Text("mymind", style = MaterialTheme.typography.headlineMedium)
        Spacer(Modifier.height(24.dp))
        Button(
            onClick = { viewModel.signInWithGoogle(context) },
            enabled = !state.busy,
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Continue with Google") }
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(
            value = username,
            onValueChange = { username = it },
            label = { Text("Username") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
            modifier = Modifier.fillMaxWidth(),
        )
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = password,
            onValueChange = { password = it },
            label = { Text("Password") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { if (canSubmit) viewModel.signInWithPassword(username, password) }),
            modifier = Modifier.fillMaxWidth(),
        )
        Spacer(Modifier.height(8.dp))
        OutlinedButton(
            onClick = { viewModel.signInWithPassword(username, password) },
            enabled = canSubmit,
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Sign in") }
        state.message?.let {
            Spacer(Modifier.height(16.dp))
            Text(it, color = MaterialTheme.colorScheme.error)
        }
    }
}

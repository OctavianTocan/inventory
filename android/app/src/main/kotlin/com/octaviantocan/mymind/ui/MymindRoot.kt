package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.octaviantocan.mymind.AppContainer
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.ui.errors.OfflineScreen
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.shell.MainShell
import com.octaviantocan.mymind.ui.signin.SignInScreen
import com.octaviantocan.mymind.ui.splash.LaunchState
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * Root of the app: the theme around the signed-in screens, the splash and sign-in overlay, and the offline notice.
 *
 * @param container Services the screens read their data and session from.
 */
@Composable
fun MymindRoot(container: AppContainer) {
    MymindTheme {
        Surface(Modifier.fillMaxSize(), color = MymindTheme.colors.page) {
            val session by container.session.state.collectAsStateWithLifecycle()
            val online by container.connectivity.online.collectAsStateWithLifecycle(container.connectivity.isOnline())
            val signIn = viewModel { SignInViewModel(container.repository) }
            val signInState by signIn.state.collectAsStateWithLifecycle()
            val launchState = when (session) {
                SessionState.Loading -> LaunchState.Splash
                SessionState.SignedOut -> if (signInState.busy) LaunchState.LoggingIn else LaunchState.Login
                SessionState.SignedIn -> LaunchState.Main
            }
            Box(Modifier.fillMaxSize()) {
                if (session == SessionState.SignedIn) {
                    SignedIn(container)
                    if (!online) OfflineScreen()
                }
                SignInScreen(signIn, launchState)
            }
        }
    }
}

/** The signed-in shell with its view models. */
@Composable
private fun SignedIn(container: AppContainer) {
    val library = viewModel { LibraryViewModel(container.repository, container.downloads) }
    val settings = viewModel { SettingsViewModel(container.repository) }
    MainShell(library, settings)
}

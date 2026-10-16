package com.octaviantocan.mymind.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.octaviantocan.mymind.AppContainer
import com.octaviantocan.mymind.data.SessionState

private const val SIGN_IN = "signin"
private const val LIBRARY = "library"

@Composable
fun MymindRoot(container: AppContainer) {
    MaterialTheme(colorScheme = if (isSystemInDarkTheme()) darkColorScheme() else lightColorScheme()) {
        Surface(Modifier.fillMaxSize()) {
            val session by container.session.state.collectAsStateWithLifecycle()
            if (session == SessionState.Loading) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { CircularProgressIndicator() }
            } else {
                AppNavigation(container, session)
            }
        }
    }
}

@Composable
private fun AppNavigation(container: AppContainer, session: SessionState) {
    val navController = rememberNavController()
    val target = if (session == SessionState.SignedIn) LIBRARY else SIGN_IN

    LaunchedEffect(target) {
        if (navController.currentDestination?.route != target) {
            navController.navigate(target) { popUpTo(0) { inclusive = true } }
        }
    }

    NavHost(navController, startDestination = target) {
        composable(SIGN_IN) {
            val vm = viewModel { SignInViewModel(container.repository) }
            SignInScreen(vm)
        }
        composable(LIBRARY) {
            val vm = viewModel { LibraryViewModel(container.repository) }
            LibraryScreen(vm)
        }
    }
}

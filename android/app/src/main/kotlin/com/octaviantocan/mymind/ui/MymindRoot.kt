package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.update.allowInstallsIntent
import com.octaviantocan.mymind.ui.update.UpdateOverlays
import com.octaviantocan.mymind.ui.update.UpdateViewModel
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.octaviantocan.mymind.AppContainer
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.ui.library.ClosePreviewsOnDispose
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.library.LocalOffline
import com.octaviantocan.mymind.ui.library.LocalVideoPreviews
import com.octaviantocan.mymind.ui.library.VideoPreviews
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.shell.MainShell
import com.octaviantocan.mymind.ui.signin.SignInScreen
import com.octaviantocan.mymind.ui.splash.LaunchState
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

// The pause after a sync the hub asked for, so a burst of announcements brings one more sync, not one each.
private const val HUB_SYNC_GAP_MS = 1_000L

/**
 * Root of the app: the theme around the signed-in screens and the splash and sign-in overlay. Without internet the
 * signed-in screens stay usable from the phone's copy and the gallery controls show an Offline pill.
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
                    CompositionLocalProvider(LocalOffline provides !online) { SignedIn(container, online) }
                }
                SignInScreen(signIn, launchState)
            }
        }
    }
}

/**
 * The signed-in shell with its view models.
 *
 * @param container Services the screens use.
 * @param online True while the phone has internet; the library syncs each time it comes back, and when the app does.
 */
@Composable
private fun SignedIn(container: AppContainer, online: Boolean) {
    val library = viewModel {
        LibraryViewModel(
            container.library,
            container.downloads,
            uploadActivity = container.uploadActivity,
            memory = container.libraryCopy,
            sync = container.librarySync,
            changes = container.library.changes,
            ai = container.ai,
            recentSearches = container.recentSearches,
        )
    }
    LaunchedEffect(online) {
        if (online) {
            library.syncNow()
            container.hub.start()
        }
    }
    // A change the server announces brings the copy up to date and onto the screens; a burst of them, such as a link
    // being read, waits out one sync before the next.
    LaunchedEffect(Unit) {
        container.hub.changes.collect {
            library.syncNow()
            delay(HUB_SYNC_GAP_MS)
        }
    }
    val settings = viewModel { SettingsViewModel(container.repository) }
    val context = LocalContext.current
    val updates = viewModel {
        UpdateViewModel(
            container.updater,
            container.autoInstaller,
            container.whatsNew,
            { context.packageManager.canRequestPackageInstalls() },
            "Version ${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})",
        )
    }
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    val scope = rememberCoroutineScope()
    DisposableEffect(lifecycle) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_START) {
                updates.onResume()
                scope.launch { container.feedback.pollActive() }
                library.syncNow()
                container.hub.start()
            }
            if (event == Lifecycle.Event.ON_STOP) container.hub.stop()
        }
        lifecycle.addObserver(observer)
        onDispose {
            lifecycle.removeObserver(observer)
            container.hub.stop()
        }
    }
    val sheet by updates.sheet.collectAsStateWithLifecycle()
    val askNotifications = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}
    LaunchedEffect(sheet != null) {
        if (sheet != null && !container.updateNotifier.allowed() && !container.updatePrefs.askedNotifications) {
            container.updatePrefs.askedNotifications = true
            askNotifications.launch(android.Manifest.permission.POST_NOTIFICATIONS)
        }
    }
    val allow = { context.startActivity(allowInstallsIntent(context)) }
    val rows by updates.settings.collectAsStateWithLifecycle()
    val previews = remember { VideoPreviews(context.applicationContext, container.httpClient) }
    ClosePreviewsOnDispose(previews)
    CompositionLocalProvider(LocalVideoPreviews provides previews) {
        MainShell(library, settings, rows.copy(onAllow = allow), ai = container.ai, feedback = container.feedback) { UpdateOverlays(updates, allow) }
    }
}

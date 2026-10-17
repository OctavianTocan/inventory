package com.octaviantocan.mymind.ui.signin

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.SignInViewModel
import com.octaviantocan.mymind.ui.errors.ApiErrorDialog
import com.octaviantocan.mymind.ui.splash.LaunchState
import com.octaviantocan.mymind.ui.splash.SplashOverlay
import com.octaviantocan.mymind.ui.splash.SplashPictures
import com.octaviantocan.mymind.ui.splash.defaultSplashPictures

private const val FAILED_TITLE = "Sign in failed"

/**
 * Splash backdrop with the sign-in content on top, and the dialog that reports a failed sign-in.
 *
 * @param viewModel Runs the sign-in calls and holds their outcome.
 * @param state Launch state to show.
 * @param modifier Modifier for the overlay.
 */
@Composable
fun SignInScreen(viewModel: SignInViewModel, state: LaunchState, modifier: Modifier = Modifier) {
    val ui by viewModel.state.collectAsStateWithLifecycle()
    val context = LocalContext.current
    SignInOverlay(
        state = state,
        onGoogle = { viewModel.signInWithGoogle(context) },
        onPassword = viewModel::signInWithPassword,
        modifier = modifier,
    )
    ui.message?.let { ApiErrorDialog(title = FAILED_TITLE, message = it, onDismiss = viewModel::dismissMessage) }
}

/**
 * The sign-in screen without a view model.
 *
 * @param state Launch state to show.
 * @param onGoogle Called when the Google button is tapped.
 * @param onPassword Called with username and password when the password form is submitted.
 * @param modifier Modifier for the overlay.
 * @param pictures Backdrop and logo of the splash.
 * @param googleLogo Logo on the Google button; the shipped picture by default.
 */
@Composable
fun SignInOverlay(
    state: LaunchState,
    onGoogle: () -> Unit,
    onPassword: (String, String) -> Unit,
    modifier: Modifier = Modifier,
    pictures: SplashPictures = defaultSplashPictures(),
    googleLogo: Painter = painterResource(R.drawable.svg_google),
) {
    var username by rememberSaveable { mutableStateOf("") }
    var password by rememberSaveable { mutableStateOf("") }
    var formHeight by remember { mutableIntStateOf(0) }
    var contentHeight by remember { mutableIntStateOf(0) }
    // The window is edge to edge, so the keyboard does not resize it; the IME padding lays everything out in the
    // space above the keyboard, as an adjustResize window would.
    BoxWithConstraints(modifier.fillMaxSize().imePadding()) {
        val lift = overlap(constraints.maxHeight, contentHeight, formHeight)
        // Above the reference phone's keyboard the logo, the title block and the form do not all fit, so the logo
        // gives way while the title block has to move up.
        SplashOverlay(state, pictures = pictures, showIcon = lift == 0) {
            SignInContent(
                showSignIn = state == LaunchState.Login,
                showLoading = state == LaunchState.LoggingIn,
                onGoogle = onGoogle,
                googleLogo = googleLogo,
                modifier = Modifier
                    .onSizeChanged { contentHeight = it.height }
                    .offset { IntOffset(0, -lift) },
            )
        }
        AnimatedVisibility(
            visible = state == LaunchState.Login,
            modifier = Modifier.align(Alignment.BottomCenter),
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            PasswordForm(
                username = username,
                password = password,
                onUsername = { username = it },
                onPassword = { password = it },
                onSubmit = { onPassword(username.trim(), password) },
                modifier = Modifier
                    .onSizeChanged { formHeight = it.height }
                    .navigationBarsPadding()
                    .padding(horizontal = 32.dp, vertical = 24.dp),
            )
        }
    }
}

/** How far the centred title and button must move up to clear the password form under them, in px. */
private fun overlap(height: Int, content: Int, form: Int): Int = ((height + content) / 2 - (height - form)).coerceAtLeast(0)

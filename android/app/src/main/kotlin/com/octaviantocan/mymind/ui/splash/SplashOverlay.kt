package com.octaviantocan.mymind.ui.splash

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.BiasAlignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Launch states of the original app that this app reaches; the splash overlay animates between them. */
enum class LaunchState {
    /** The stored session is still being read. */
    Splash,

    /** Signed out, waiting for the user to sign in. */
    Login,

    /** A sign-in call is running. */
    LoggingIn,

    /** Signed in; the overlay fades out. */
    Main,
}

/**
 * The two pictures of the splash screen.
 *
 * @property background Full-width backdrop, drawn at its own size and blurred.
 * @property icon Logo, drawn 64 dp wide and tinted with the splash icon colour.
 */
@Immutable
class SplashPictures(val background: Painter, val icon: Painter)

/**
 * @return The pictures that ship with the app resources.
 */
@Composable
fun defaultSplashPictures() = SplashPictures(
    background = painterResource(R.drawable.splash_bg),
    icon = painterResource(R.drawable.mymind_splash_icon),
)

private const val FADE_OUT_MS = 500
private const val SCENE_MS = 1_500
private const val ICON_FADE_MS = 500
private const val LOGIN_ICON_BIAS = -0.7f
private const val GRADIENT_START = 0.4f
private const val GRADIENT_END = 0.7f
private const val GRADIENT_WHITE_ALPHA = 0.75f

private class SplashTargets {
    var scale by mutableFloatStateOf(1f)
    var blur by mutableFloatStateOf(0f)
    var iconBias by mutableFloatStateOf(0f)
    var iconScale by mutableFloatStateOf(1f)
    var iconAlpha by mutableFloatStateOf(0f)
    var gradientAlpha by mutableFloatStateOf(0f)

    fun moveTo(state: LaunchState) {
        when (state) {
            LaunchState.Login, LaunchState.LoggingIn -> {
                scale = 1.5f
                blur = 30f
                iconBias = LOGIN_ICON_BIAS
                iconScale = 0.75f
                gradientAlpha = 1f
                iconAlpha = 1f
            }
            LaunchState.Splash, LaunchState.Main -> {
                gradientAlpha = 0f
                blur = 15f
            }
        }
    }
}

/**
 * The pink splash that covers the app until the session is known, and that carries the sign-in content.
 *
 * @param state Launch state to show; the overlay is visible in every state except [LaunchState.Main].
 * @param modifier Modifier for the overlay box.
 * @param pictures Backdrop and logo; the shipped pictures by default.
 * @param showIcon False fades the logo out, for when the sign-in content needs its room.
 * @param content Sign-in content drawn over the backdrop; it fades in with the white gradient.
 */
@Composable
fun SplashOverlay(
    state: LaunchState,
    modifier: Modifier = Modifier,
    pictures: SplashPictures = defaultSplashPictures(),
    showIcon: Boolean = true,
    content: @Composable BoxScope.() -> Unit,
) {
    val targets = remember { SplashTargets() }
    LaunchedEffect(state) { targets.moveTo(state) }

    AnimatedVisibility(
        visible = state != LaunchState.Main,
        modifier = modifier,
        enter = fadeIn(),
        exit = fadeOut(tween(FADE_OUT_MS)),
    ) {
        val scene = tween<Float>(SCENE_MS)
        val scale by animateFloatAsState(targets.scale, scene, label = "animate scale")
        val blur by animateFloatAsState(targets.blur, scene, label = "animate blur")
        val iconScale by animateFloatAsState(targets.iconScale, scene, label = "animate icon scale")
        val iconAlpha by animateFloatAsState(if (showIcon) targets.iconAlpha else 0f, tween(ICON_FADE_MS), label = "animate icon alpha")
        val gradientAlpha by animateFloatAsState(targets.gradientAlpha, scene, label = "animate gradient alpha")

        Box(Modifier.fillMaxSize().background(MymindTheme.colors.onboardingBackground)) {
            Image(
                painter = pictures.background,
                contentDescription = null,
                modifier = Modifier.align(Alignment.Center).scale(scale).blur(blur.dp),
                contentScale = ContentScale.Inside,
            )
            Image(
                painter = pictures.icon,
                contentDescription = null,
                modifier = Modifier
                    .graphicsLayer { alpha = iconAlpha }
                    .width(64.dp)
                    .scale(iconScale)
                    .align(BiasAlignment(0f, targets.iconBias)),
                colorFilter = ColorFilter.tint(MymindTheme.colors.splashIconTint, BlendMode.SrcIn),
            )
            Box(
                Modifier
                    .graphicsLayer { alpha = gradientAlpha }
                    .fillMaxSize()
                    .background(
                        Brush.verticalGradient(
                            GRADIENT_START to Color.Transparent,
                            GRADIENT_END to Color.White.copy(alpha = GRADIENT_WHITE_ALPHA),
                        ),
                    ),
            )
            Box(Modifier.align(Alignment.Center).graphicsLayer { alpha = gradientAlpha }, content = content)
        }
    }
}

package com.octaviantocan.mymind.ui.shell

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

/**
 * Lets the page take the whole screen, as a gallery does in reels: the shell darkens the status bar and slides its bar
 * away as far as the page has gone full screen.
 */
@Stable
class ShellImmersion {
    /** How far the page has gone full screen, from 0 to 1, read while drawing; null for a page that never does. */
    var page: (() -> Float)? by mutableStateOf(null)

    /** How far the page has gone full screen now, from 0 to 1. */
    fun amount(): Float = page?.invoke()?.coerceIn(0f, 1f) ?: 0f

    /**
     * A colour laid over the status bar's strip, read while drawing, for a page whose own top has become a picture,
     * such as the sky; null leaves the strip the page's colour.
     */
    var strip: (() -> Color)? by mutableStateOf(null)
}

/** The shell's [ShellImmersion], or null outside a [ShellFrame]. */
val LocalShellImmersion = staticCompositionLocalOf<ShellImmersion?> { null }

/** Gives the status and navigation bars light icons while [immersion] is more than half full screen, over its black. */
@Composable
internal fun ImmersiveSystemBars(immersion: ShellImmersion) {
    val view = LocalView.current
    val dark by remember(immersion) { derivedStateOf { immersion.amount() > 0.5f } }
    if (!dark || view.isInEditMode) return
    DisposableEffect(view) {
        val window = view.context.activity()?.window
        val controller = window?.let { WindowCompat.getInsetsController(it, view) }
        val status = controller?.isAppearanceLightStatusBars
        val navigation = controller?.isAppearanceLightNavigationBars
        controller?.isAppearanceLightStatusBars = false
        controller?.isAppearanceLightNavigationBars = false
        onDispose {
            if (status != null) controller.isAppearanceLightStatusBars = status
            if (navigation != null) controller.isAppearanceLightNavigationBars = navigation
        }
    }
}

/** The activity this context belongs to, or null. */
private tailrec fun Context.activity(): Activity? = when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.activity()
    else -> null
}

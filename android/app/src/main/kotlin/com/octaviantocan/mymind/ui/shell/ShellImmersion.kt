package com.octaviantocan.mymind.ui.shell

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat
import kotlin.math.max

/**
 * Lets the page take the whole screen, as a gallery does in reels: the shell darkens the status bar and slides its bar
 * away as far as the page has gone full screen. Several parts of one page may each say so (the gallery in reels, the
 * sky looked up at); the furthest counts, whichever said it last.
 */
@Stable
class ShellImmersion {
    private val pages = mutableStateListOf<() -> Float>()
    private val strips = mutableStateListOf<() -> Color>()

    /**
     * Adds [source], read while drawing, to how far the page has gone full screen, from 0 to 1.
     *
     * @return A call that takes [source] away again.
     */
    fun addPage(source: () -> Float): () -> Unit {
        pages += source
        return { pages.remove(source) }
    }

    /**
     * Adds [source], read while drawing, a colour laid over the status bar's strip for a page whose own top has become
     * a picture, such as the sky; the most opaque one shows.
     *
     * @return A call that takes [source] away again.
     */
    fun addStrip(source: () -> Color): () -> Unit {
        strips += source
        return { strips.remove(source) }
    }

    /** How far the page has gone full screen now, from 0 to 1. */
    fun amount(): Float {
        var amount = 0f
        for (source in pages) amount = max(amount, source())
        return amount.coerceIn(0f, 1f)
    }

    /** The colour over the status bar's strip now, or null to leave it the page's colour. */
    fun strip(): Color? {
        var color: Color? = null
        for (source in strips) {
            val own = source()
            if (color == null || own.alpha > color.alpha) color = own
        }
        return color
    }
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

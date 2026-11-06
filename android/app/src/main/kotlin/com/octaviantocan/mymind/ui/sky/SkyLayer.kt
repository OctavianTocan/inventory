package com.octaviantocan.mymind.ui.sky

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.ui.shell.LocalShellImmersion
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlin.math.max

/** How long the sky takes to open from the star button. */
internal const val SKY_OPEN_MS = 420

private const val SKY_CLOSE_MS = 300

/**
 * Whether the sky is open over the gallery, and how far it has opened.
 *
 * @param scope Where the opening runs.
 * @param startOpen Whether it starts open, as when the phone turns while it shows.
 */
@Stable
class SkyOpening internal constructor(private val scope: CoroutineScope, startOpen: Boolean) {
    /** True from Open until Close. */
    var isOpen by mutableStateOf(startOpen)
        private set

    /** How far the sky shows, from 0 to 1. */
    internal val shown = Animatable(if (startOpen) 1f else 0f)

    /** Opens the sky. */
    fun open() {
        isOpen = true
        scope.launch { shown.animateTo(1f, tween(SKY_OPEN_MS, easing = FastOutSlowInEasing)) }
    }

    /** Closes the sky. */
    fun close() {
        isOpen = false
        scope.launch { shown.animateTo(0f, tween(SKY_CLOSE_MS, easing = FastOutSlowInEasing)) }
    }
}

/** Remembers whether the sky is open, across the phone turning. */
@Composable
fun rememberSkyOpening(): SkyOpening {
    val scope = rememberCoroutineScope()
    return rememberSaveable(scope, saver = Saver(save = { it.isOpen }, restore = { SkyOpening(scope, it) })) { SkyOpening(scope, false) }
}

/**
 * The sky over the gallery while [opening] says so: it grows out of the star button, hides the tab bar as reels do,
 * and Back closes it. A find opened from it rises over it, and closing the find comes back to it.
 *
 * @param opening Whether the sky is open.
 * @param sky The sky.
 * @param finds The sky's finds that are known, by id.
 * @param onOpenStar Called with a find's id when it is opened from the sky.
 * @param covered True while something covers the sky, such as a find opened from it.
 */
@Composable
fun SkyLayer(opening: SkyOpening, sky: Sky, finds: Map<String, Card>, onOpenStar: (String) -> Unit, covered: Boolean = false) {
    val visible by remember(opening) { derivedStateOf { opening.isOpen || opening.shown.value > 0f } }
    if (!visible) return
    BackHandler(enabled = opening.isOpen) { opening.close() }
    val immersion = LocalShellImmersion.current
    if (immersion != null) DisposableEffect(immersion, opening) {
        val before = immersion.page
        val source = { max(before?.invoke() ?: 0f, opening.shown.value) }
        immersion.page = source
        onDispose { if (immersion.page === source) immersion.page = before }
    }
    SkyScreen(
        sky, finds, onOpenStar, onClose = opening::close, covered = covered,
        modifier = Modifier.graphicsLayer {
            val shown = opening.shown.value
            alpha = shown
            val scale = 0.9f + 0.1f * shown
            scaleX = scale
            scaleY = scale
            transformOrigin = TransformOrigin(0.8f, 0.03f)
        },
    )
}

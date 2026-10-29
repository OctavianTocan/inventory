package com.octaviantocan.mymind.ui.share

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.Role
import androidx.compose.foundation.clickable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.background
import com.octaviantocan.mymind.ui.card.CardSheetMotion
import com.octaviantocan.mymind.ui.card.CardSheetPosition
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuPanel
import com.octaviantocan.mymind.ui.menu.menuColors
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** How long the sheet waits, untouched, once everything is saved (share-sheet-fades-by-itself). */
const val SAVED_FADE_MS = 5_000L

/** How long a sheet saying nothing was saved waits, untouched (share-sheet-fades-by-itself). */
const val PROBLEM_FADE_MS = 8_000L

/** How long fading away takes. */
const val FADE_OUT_MS = 600

/**
 * What the share target's sheets give their content: how much of the wait is left, and a close that slides the sheet
 * down.
 */
internal class ShareFrameScope(private val left: Animatable<Float, *>, private val counting: () -> Boolean, val close: () -> Unit) {
    /** The part of the wait still to go, from 1 down to 0, or null while nothing is counting. */
    fun left(): Float? = if (counting()) left.value else null
}

/**
 * Pully's sheet over the app that shared, resting at its content's height. Once [counting] turns true it waits
 * [waitMs] and fades away by itself; the first touch anywhere calls [onTouch], which is expected to stop the count by
 * turning [counting] false. Its close, the scrim, Back and a drag slide it down. Either way [onClosed] runs once it has
 * gone.
 *
 * @param title The sheet's title, for accessibility and as the header when [titleContent] is null.
 * @param counting True while the sheet is waiting to fade.
 * @param waitMs How long it waits before it fades.
 * @param onTouch Called on every touch while the sheet is open.
 * @param onClosed Called once the sheet has slid down or faded away.
 * @param titleContent The header's title, such as an icon and the words.
 * @param content The sheet's content.
 */
@Composable
internal fun ShareFrame(
    title: String,
    counting: Boolean,
    waitMs: Long,
    onTouch: () -> Unit,
    onClosed: () -> Unit,
    titleContent: (@Composable () -> Unit)? = null,
    content: @Composable ColumnScope.(ShareFrameScope) -> Unit,
) {
    val haptic = rememberSheetHaptics()
    val position = remember { CardSheetPosition(haptic::land, haptic::tick).apply { expandable = false } }
    val scope = rememberCoroutineScope()
    val motion = rememberSheetMotionEnabled()
    val latestClosed by rememberUpdatedState(onClosed)
    val latestCounting by rememberUpdatedState(counting)
    val latestTouch by rememberUpdatedState(onTouch)
    val left = remember { Animatable(1f) }
    val fade = remember { Animatable(1f) }
    LaunchedEffect(counting, waitMs) {
        if (!counting) return@LaunchedEffect
        left.snapTo(1f)
        // The wait is a delay, not the animation, so it lasts as long with the system's animations turned off.
        launch { left.animateTo(0f, tween(waitMs.toInt(), easing = LinearEasing)) }
        delay(waitMs)
        if (!position.startClosing()) return@LaunchedEffect
        if (motion) fade.animateTo(0f, tween(FADE_OUT_MS, easing = CardSheetMotion.SmoothOut)) else fade.snapTo(0f)
        latestClosed()
    }
    val frame = remember(position) {
        ShareFrameScope(left, { latestCounting }) { position.dismiss(scope, motion) { latestClosed() } }
    }
    Box(
        Modifier.fillMaxSize().graphicsLayer { alpha = fade.value }.pointerInput(Unit) {
            awaitEachGesture {
                awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                if (!position.closing) latestTouch()
            }
        },
    ) {
        // Laid out above the keyboard: a sheet without a tall height to lift to would otherwise sit behind it.
        Box(Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.ime)) {
            CardMenuPanel(title, onDismiss = { latestClosed() }, position = position, scrollState = rememberScrollState(),
                titleContent = titleContent, closeLabel = "Close") {
                content(frame)
            }
        }
    }
}

/**
 * The sheet's ink button; while the sheet waits to fade, a lighter part shrinking from its end shows the time left.
 *
 * @param label The button's words.
 * @param left The part of the wait still to go, or null for none.
 * @param onClick Called when it is tapped.
 */
@Composable
internal fun ShareDoneButton(label: String, left: () -> Float?, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    val wash = colors.sheet.copy(alpha = if (colors.sheet.red > 0.5f) 0.18f else 0.22f)
    Box(
        modifier.fillMaxWidth().height(52.dp).clip(CircleShape).background(colors.rowText)
            .drawBehind { left()?.let { drawRect(wash, size = Size(size.width * it, size.height)) } }
            .clickable(role = Role.Button, onClickLabel = label) { haptic.tick(); onClick() },
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = TextStyle(color = colors.sheet, fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium, fontFamily = CardMenuFont))
    }
}

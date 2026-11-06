package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.PointerInputScope
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/** The dark of the chosen mark, the app's ink, so the gold star reads on any picture. */
private val ChoiceInk = Color(0xFF0E333A)

/** The star's gold, the Inventory accent. */
private val ChoiceGold = Color(0xFFF2C14E)

/** How close to the top or bottom edge the finger must be for the gallery to scroll on under a sweep. */
private val SWEEP_EDGE = 72.dp

/** The fastest a sweep scrolls the gallery, in dp a frame, with the finger at the very edge. */
private val SWEEP_SPEED = 18.dp

/**
 * What a [choiceSweep] needs from its gallery, read at the moment of each touch.
 *
 * @property cards The gallery's cards in their order.
 * @property cardAt The index of the card whose tile is under a point of the gallery, or null.
 * @property enabled False while the gallery cannot be swept, such as while pinching or in reels.
 * @property scrollBy Scrolls the gallery by pixels, positive towards its end.
 * @property onHold Called with a card held and let go without sliding: the card menu.
 */
internal class SweepTarget(
    val cards: () -> List<Card>,
    val cardAt: (Offset) -> Int?,
    val enabled: () -> Boolean,
    val scrollBy: suspend (Float) -> Unit,
    val onHold: ((Card) -> Unit)?,
)

/**
 * Hold a find and slide to gather it and every find up to the finger into [choice], row by row in the gallery's order;
 * sliding back drops them again, and the gallery scrolls on while the finger rests near its top or bottom. Held and let
 * go without sliding, the find's menu opens through [SweepTarget.onHold], so holding keeps its old meaning. Any other
 * touch (a tap, a scroll, a pinch) is left to the gallery.
 *
 * @param choice Where the gathered finds go.
 * @param target The gallery being swept.
 * @param scope Runs the scrolling at the edges.
 * @param haptics Ticks when the hold starts and as finds join.
 */
internal fun Modifier.choiceSweep(choice: CardChoice, target: () -> SweepTarget, scope: CoroutineScope, haptics: HapticFeedback): Modifier =
    pointerInput(choice) { sweepGestures(choice, target, scope, haptics) }

private suspend fun PointerInputScope.sweepGestures(choice: CardChoice, target: () -> SweepTarget, scope: CoroutineScope, haptics: HapticFeedback) {
    val edge = SWEEP_EDGE.toPx()
    val speed = SWEEP_SPEED.toPx()
    awaitEachGesture {
        val down = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
        val gallery = target()
        if (!gallery.enabled()) return@awaitEachGesture
        val start = gallery.cardAt(down.position) ?: return@awaitEachGesture
        val cards = gallery.cards()
        // A finger that moves, lifts or is joined by another before the hold time is the gallery's: a scroll, a tap, a pinch.
        val stillHeld: Boolean? = withTimeoutOrNull<Boolean?>(viewConfiguration.longPressTimeoutMillis) {
            while (true) {
                val event = awaitPointerEvent(PointerEventPass.Initial)
                val change = event.changes.firstOrNull { it.id == down.id }
                if (change == null || !change.pressed || event.changes.count { it.pressed } > 1) return@withTimeoutOrNull false
                if ((change.position - down.position).getDistance() > viewConfiguration.touchSlop) return@withTimeoutOrNull false
            }
            @Suppress("UNREACHABLE_CODE")
            null
        }
        if (stillHeld == false || start !in cards.indices) return@awaitEachGesture
        haptics.performHapticFeedback(HapticFeedbackType.LongPress)
        var point = down.position
        var swept = false
        var last = start
        var edgeJob: Job? = null
        fun follow() {
            val index = target().cardAt(point) ?: return
            if (index == last) return
            last = index
            choice.sweepTo(cards, index)
            haptics.performHapticFeedback(HapticFeedbackType.TextHandleMove)
        }
        while (true) {
            val event = awaitPointerEvent(PointerEventPass.Initial)
            val change = event.changes.firstOrNull { it.id == down.id }
            // From the hold on, every touch is the sweep's: the tile does not take it as a tap, the gallery does not scroll.
            event.changes.forEach { it.consume() }
            if (change == null || !change.pressed) break
            if (event.changes.count { it.pressed } > 1) break
            point = change.position
            if (!swept && (point - down.position).getDistance() > viewConfiguration.touchSlop) {
                swept = true
                choice.beginSweep(cards, start)
                edgeJob = scope.launch {
                    while (isActive) {
                        withFrameNanos { }
                        val height = size.height.toFloat()
                        val step = when {
                            point.y < edge -> -speed * (1f - point.y.coerceAtLeast(0f) / edge)
                            point.y > height - edge -> speed * (1f - (height - point.y).coerceAtLeast(0f) / edge)
                            else -> 0f
                        }
                        if (step != 0f) {
                            target().scrollBy(step)
                            follow()
                        }
                    }
                }
            }
            if (swept) follow()
        }
        edgeJob?.cancel()
        if (swept) choice.endSweep() else target().onHold?.invoke(cards[start])
    }
}

/**
 * The mark a find carries while finds are being chosen: a ring and a gold star when it is in the backpack, an empty
 * circle when it is not. It draws over the tile and takes no touches, so the tile under it is never rebuilt.
 *
 * @param choice The finds chosen.
 * @param id This find's id.
 * @param corner The tile's corner radius.
 */
@Composable
internal fun ChoiceMark(choice: CardChoice, id: String, corner: Dp) {
    if (!choice.active) return
    val chosen = choice.contains(id)
    val ring = LibraryStyle.colors.text
    Box(Modifier.fillMaxSize().clearAndSetSemantics {}
        .then(if (chosen) Modifier.border(2.dp, ring, RoundedCornerShape(corner)) else Modifier)) {
        Box(
            Modifier.align(Alignment.TopEnd).padding(8.dp).size(24.dp).clip(CircleShape)
                .background(if (chosen) ChoiceInk else ChoiceInk.copy(alpha = 0.18f))
                .border(if (chosen) 1.5.dp else 2.dp, Color.White, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            if (chosen) Canvas(Modifier.size(13.dp)) { drawPath(starPath(size.width), ChoiceGold) }
        }
    }
}

/** A four-pointed star filling a square of [side] pixels, the Inventory spark. */
private fun starPath(side: Float): Path {
    val c = side / 2f
    val inner = side * 0.12f
    return Path().apply {
        moveTo(c, 0f)
        lineTo(c + inner, c - inner)
        lineTo(side, c)
        lineTo(c + inner, c + inner)
        lineTo(c, side)
        lineTo(c - inner, c + inner)
        lineTo(0f, c)
        lineTo(c - inner, c - inner)
        close()
    }
}

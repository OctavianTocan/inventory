package com.octaviantocan.mymind.ui.card

import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

/** Where the reader's sheet and the gallery stepped back behind it sit, below the status bar. */
object ReaderSheetLayout {
    /** From the status bar to the top of the gallery once it has stepped back. */
    val GalleryGap = 6.dp

    /** From the status bar to the top of the sheet, so the gallery's edge shows between them. */
    val SheetGap = 22.dp

    /** The sheet's and the stepped-back gallery's top corners. */
    val Corner = 24.dp

    /** How much smaller the gallery gets behind the sheet. */
    const val GALLERY_SHRINK = 0.08f
}

/**
 * Where the reader's sheet is. It rises over the gallery when a card opens and sinks on Back, on Pully's panel timings,
 * and a finger can pull it down; the gallery behind steps back into a card as far as the sheet is up ([recede]).
 *
 * @param risen True for a sheet that starts at rest, as a reader shown on its own does.
 */
@Stable
class ReaderSheetState(risen: Boolean = false) {
    /** How far the sheet has risen on its open and close tweens, from 0 (gone) to 1 (at rest). */
    var rise by mutableFloatStateOf(if (risen) 1f else 0f)
        private set

    /** How far a finger or a throw has taken the sheet down from rest, in px. */
    var pull by mutableFloatStateOf(0f)
        private set

    /** True from the start of a close until the reader is gone, so a second close does nothing. */
    var closing by mutableStateOf(false)
        private set

    /** The sheet's height in px, from its last layout. */
    internal var height by mutableFloatStateOf(0f)

    /** True while the sheet sits below rest, so the next upward drag raises it before the content scrolls. */
    internal val pulled: Boolean get() = pull > 0f

    private var move: Job? = null
    private var rising: Job? = null

    /** How far the gallery behind has stepped back, from 0 to 1; read it while drawing. */
    fun recede(): Float = rise * (1f - if (height > 0f) (pull / height).coerceIn(0f, 1f) else 0f)

    /** Raises the sheet to rest from wherever it is, in 400ms on the smooth-out curve. */
    fun show(scope: CoroutineScope, motion: Boolean) {
        closing = false
        if (!motion) {
            stop()
            rise = 1f
            pull = 0f
            return
        }
        if (pull > 0f) glide(scope) { animate(pull, 0f, animationSpec = CardSheetMotion.SheetSpring) { v, _ -> pull = v } }
        lift(scope) { animate(rise, 1f, animationSpec = tween(CardSheetMotion.SLOW, easing = CardSheetMotion.SmoothOut)) { v, _ -> rise = v } }
    }

    /** Sinks the sheet the way it came, in 350ms, then calls [gone]. A second call while it sinks does nothing. */
    fun hide(scope: CoroutineScope, motion: Boolean, gone: () -> Unit) {
        if (closing) return
        closing = true
        move?.cancel()
        lift(scope) {
            if (motion) animate(rise, 0f, animationSpec = tween(CardSheetMotion.MEDIUM, easing = CardSheetMotion.SmoothOut)) { v, _ -> rise = v }
            else rise = 0f
            gone()
        }
    }

    /** Moves the sheet with a finger, [dy] px down or up, never above rest; returns how much of [dy] it used. */
    internal fun drag(dy: Float): Float {
        if (closing || height <= 0f) return 0f
        move?.cancel()
        val next = (pull + dy).coerceIn(0f, height)
        val used = next - pull
        pull = next
        return used
    }

    /**
     * The finger lets go at [velocity] (px/s, down is positive). Thrown down faster than [speed], or let go more than 30%
     * of the way down, the sheet carries on down the way the finger went, as the card's sheets do, and [gone] runs;
     * otherwise it springs back to rest.
     */
    internal fun release(scope: CoroutineScope, velocity: Float, speed: Float, motion: Boolean, gone: () -> Unit) {
        if (closing || !pulled) return
        if (cardSheetTarget(pull, velocity, speed, listOf(0f), height) < height) {
            glide(scope) {
                if (motion) animate(pull, 0f, initialVelocity = velocity, animationSpec = CardSheetMotion.SheetSpring) { v, _ -> pull = maxOf(v, 0f) }
                else pull = 0f
            }
            return
        }
        closing = true
        glide(scope) {
            if (motion) animate(pull, height, animationSpec = tween(CardSheetMotion.MEDIUM, easing = CardSheetMotion.SmoothOut)) { v, _ -> pull = v }
            else pull = height
            gone()
        }
    }

    /** Puts the sheet back at gone, ready for the next card. */
    fun reset() {
        stop()
        rise = 0f
        pull = 0f
        closing = false
    }

    /** Runs [block] as the only move of [pull], so a finger or a newer move takes over from it. */
    private fun glide(scope: CoroutineScope, block: suspend CoroutineScope.() -> Unit) {
        move?.cancel()
        move = scope.launch(block = block)
    }

    /** Runs [block] as the only move of [rise], which a finger leaves running so the sheet still arrives whole. */
    private fun lift(scope: CoroutineScope, block: suspend CoroutineScope.() -> Unit) {
        rising?.cancel()
        rising = scope.launch(block = block)
    }

    private fun stop() {
        move?.cancel()
        rising?.cancel()
    }
}

/**
 * Lets the reader's sheet follow its content: pulling down at the top of the content lowers the sheet, but only when
 * the drag began with the content at its top, and scrolling up raises a lowered sheet first. A drag that scrolled the
 * content, or a fling that runs into the top, leaves the sheet where it is (Pully's H2, as the card's sheets do).
 */
internal class ReaderSheetScroll(
    private val sheet: ReaderSheetState,
    private val scope: CoroutineScope,
    private val speed: Float,
    private val motion: () -> Boolean,
    private val gone: () -> Unit,
) : NestedScrollConnection {
    /** This drag has scrolled the content, so it belongs to the content until the finger lifts. */
    private var scrolledContent = false

    override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset =
        if (available.y < 0f && sheet.pulled && source == NestedScrollSource.UserInput) Offset(0f, sheet.drag(available.y)) else Offset.Zero

    override fun onPostScroll(consumed: Offset, available: Offset, source: NestedScrollSource): Offset {
        if (source != NestedScrollSource.UserInput) return Offset.Zero
        if (consumed.y != 0f) scrolledContent = true
        return if (available.y > 0f && !scrolledContent) Offset(0f, sheet.drag(available.y)) else Offset.Zero
    }

    override suspend fun onPreFling(available: Velocity): Velocity {
        scrolledContent = false
        if (!sheet.pulled) return Velocity.Zero
        sheet.release(scope, available.y, speed, motion(), gone)
        return available
    }
}

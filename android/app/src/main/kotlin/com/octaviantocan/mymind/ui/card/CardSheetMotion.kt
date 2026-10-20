package com.octaviantocan.mymind.ui.card

import androidx.compose.animation.core.animate
import androidx.compose.animation.core.spring
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
import kotlin.math.abs
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

/** Choose the next sheet stop, following pulls.review's half-height, expanded and dismissed stops. */
internal fun cardSheetTarget(offset: Float, velocity: Float, speed: Float, height: Float): Float {
    val normal = height / 2f
    val rests = listOf(0f, normal)
    return when {
        velocity > speed -> rests.firstOrNull { it > offset + 1f } ?: height
        velocity < -speed -> rests.lastOrNull { it < offset - 1f } ?: 0f
        offset - normal > (height - normal) * 0.3f -> height
        else -> rests.minBy { abs(it - offset) }
    }
}

/** The card panel's position in pixels; only a completed exit removes the card from the shell. */
@Stable
internal class CardSheetPosition {
    var height by mutableFloatStateOf(0f)
        private set
    var offset by mutableFloatStateOf(0f)
        private set
    var closing by mutableStateOf(false)
        private set
    var held = false
        private set
    private var expanded = false
    private var movement: Job? = null

    val canRise: Boolean get() = offset > 0.5f && !closing
    val scrim: Float get() = if (height == 0f) 0f else ((height - offset) / (height / 2f)).coerceIn(0f, 1f)

    /** Open from below the window, or follow the same stop when the window changes size. */
    fun measured(px: Float, scope: CoroutineScope) {
        if (px <= 0f || px == height) return
        val first = height == 0f
        height = px
        if (first) offset = px
        if (!held && !closing) move(scope, if (expanded) 0f else height / 2f)
    }

    /** Move with the finger, bounded by the top and bottom of the panel. */
    fun dragBy(dy: Float): Float {
        if (closing || height == 0f) return 0f
        movement?.cancel()
        held = true
        val next = (offset + dy).coerceIn(0f, height)
        return (next - offset).also { offset = next }
    }

    /** Settle at the next stop with the release velocity, or request the animated exit. */
    fun settle(scope: CoroutineScope, velocity: Float, speed: Float, dismiss: () -> Unit) {
        if (closing) return
        held = false
        val target = cardSheetTarget(offset, velocity, speed, height)
        if (target >= height) dismiss() else {
            expanded = target == 0f
            move(scope, target, velocity)
        }
    }

    /** Move to an explicit resting height for accessibility actions and the grabber. */
    fun expand(scope: CoroutineScope, value: Boolean) {
        if (closing) return
        held = false
        expanded = value
        move(scope, if (value) 0f else height / 2f)
    }

    /** Animate below the window before notifying the owner, once even if close is tapped repeatedly. */
    fun dismiss(scope: CoroutineScope, onClose: () -> Unit) {
        if (closing) return
        closing = true
        movement?.cancel()
        movement = scope.launch {
            animate(offset, height, animationSpec = tween(350)) { value, _ -> offset = value }
            onClose()
        }
    }

    private fun move(scope: CoroutineScope, target: Float, velocity: Float = 0f) {
        movement?.cancel()
        movement = scope.launch {
            animate(offset, target, initialVelocity = velocity, animationSpec = spring(dampingRatio = 0.85f, stiffness = 400f)) { value, _ ->
                offset = value.coerceIn(0f, height)
            }
        }
    }
}

/** Content owns a drag that scrolled it; reaching its top or flinging never pulls the sheet down. */
internal class CardSheetScroll(
    private val position: CardSheetPosition,
    private val scope: CoroutineScope,
    private val speed: Float,
    private val dismiss: () -> Unit,
) : NestedScrollConnection {
    private var scrolledContent = false

    override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset =
        if (source == NestedScrollSource.UserInput && available.y < 0f && position.canRise) {
            Offset(0f, position.dragBy(available.y))
        } else Offset.Zero

    override fun onPostScroll(consumed: Offset, available: Offset, source: NestedScrollSource): Offset {
        if (source != NestedScrollSource.UserInput) return Offset.Zero
        if (consumed.y != 0f) scrolledContent = true
        return if (!scrolledContent && available.y > 0f) Offset(0f, position.dragBy(available.y)) else Offset.Zero
    }

    override suspend fun onPreFling(available: Velocity): Velocity {
        scrolledContent = false
        if (!position.held) return Velocity.Zero
        position.settle(scope, available.y, speed, dismiss)
        return if (available.y < 0f) Velocity.Zero else available
    }
}

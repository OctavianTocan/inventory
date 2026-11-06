package com.octaviantocan.mymind.ui.shell

import androidx.compose.animation.core.animate
import androidx.compose.animation.core.spring
import androidx.compose.foundation.gestures.DraggableState
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.roundToInt

/** The room between two side pages while they move, as Pully's `PullsMotion.PageGap`. */
private val PAGE_GAP = 16.dp

/** How fast a let-go finger must move to throw the page on, as Pully's `PullsMotion.Fling`. */
private val FLING = 500.dp

/** How far (in pages) the finger must move the pages before a throw turns them, as in Pully. */
private const val THROW_AFTER = 0.12f

/** Pully's `PullsMotion.PageSpring`: critically damped, so a page settles without a snap at the end. */
private val PAGE_SPRING = spring(dampingRatio = 1f, stiffness = 300f, visibilityThreshold = 0.0005f)

/**
 * True while the side pages rest on one page, read while drawing. A page can drop costly drawing, such as the gallery's
 * blurred foot fade, while it is being swiped. Outside [SidePages] it is always at rest.
 */
val LocalSidePagesAtRest = compositionLocalOf<() -> Boolean> { { true } }

/**
 * Where a row of side-by-side pages sits, in pages: 0 is the first page exactly, 0.5 halfway to the second. Ported from
 * Pully's `SidePagesState` (pulls.review `ui/design/SidePages.kt`), so the library's pages move the way Pully's Inbox and
 * Tours do.
 */
@Stable
class SidePagesState internal constructor(start: Float) {
    /** Where the pages are, in pages. Read it in a layer or a draw block, so following it never recomposes. */
    var at by mutableFloatStateOf(start)
        internal set

    internal var glide: Job? = null

    /** The page a swipe settled on that the caller has yet to take up, so the change is not glided twice. */
    internal var thrownTo by mutableIntStateOf(-1)

    /** How fast the finger let go, in pages per second, for the glide that follows it. */
    internal var speed = 0f
    internal var width by mutableIntStateOf(0)
    internal var gap = 0f
    private var startAt = 0f
    internal var index = 0
    internal var last = 0
    internal var moving = true
    internal var fling = 0f
    internal var scope: CoroutineScope? = null
    internal var onPage: (Int) -> Unit = {}
    internal var onTurn: () -> Unit = {}

    /** A second finger came down in this gesture: it is a pinch, so the pages stay where they are and settle back. */
    internal var pinched = false

    /** True while the pages rest on a page, neither dragged nor gliding. */
    val atRest: Boolean get() = at == index.toFloat() && !dragging
    internal var dragging by mutableStateOf(false)

    internal fun glideTo(page: Int, speed: Float = 0f) {
        glide?.cancel()
        if (!moving) {
            at = page.toFloat()
            return
        }
        glide = scope?.launch {
            animate(at, page.toFloat(), initialVelocity = speed, animationSpec = PAGE_SPRING) { value, _ -> at = value }
        }
    }

    private val travel get() = width + gap

    // The pages move exactly as far as the finger: Pully divides by the page and the gap ("it doesn't follow my fingers
    // properly", Pully recording 15 Oct 15:37, L6).
    internal val drag = DraggableState { delta ->
        if (width <= 0 || pinched) return@DraggableState
        at = (at - delta / travel).coerceIn(0f, last.toFloat())
    }

    internal fun started() {
        glide?.cancel()
        dragging = true
        startAt = at
    }

    /** A second finger came down: the pages go back to the page they rest on and leave the fingers to the pinch. */
    internal fun pinch() {
        if (pinched) return
        pinched = true
        if (dragging) glideTo(index)
    }

    internal fun stopped(velocity: Float) {
        dragging = false
        val now = at
        val thrown = if (pinched) 0f else velocity
        val speed = if (width > 0) -thrown / travel else 0f
        // A flick turns the page only once the finger has taken it part of the way, so a nudge never turns it by itself.
        val moved = abs(now - startAt) >= THROW_AFTER
        val to = when {
            pinched -> index
            moved && thrown < -fling -> ceil(now).toInt()
            moved && thrown > fling -> floor(now).toInt()
            else -> now.roundToInt()
        }.coerceIn(0, last)
        if (to != index) {
            onTurn()
            thrownTo = to
            this.speed = speed
            onPage(to)
        } else {
            glideTo(to, speed)
        }
    }
}

/**
 * The state of a row of [count] side-by-side pages showing [index]. A change of [index] from elsewhere, a tab, glides
 * the row there; [onPage] takes the page a swipe settled on, as soon as the finger lets go.
 */
@Composable
fun rememberSidePages(index: Int, count: Int, onPage: (Int) -> Unit): SidePagesState {
    val state = remember { SidePagesState(index.toFloat()) }
    val latest by rememberUpdatedState(onPage)
    val haptic = LocalHapticFeedback.current
    state.scope = rememberCoroutineScope()
    state.moving = rememberSheetMotionEnabled()
    with(LocalDensity.current) {
        state.fling = FLING.toPx()
        state.gap = PAGE_GAP.toPx()
    }
    state.index = index
    state.last = (count - 1).coerceAtLeast(0)
    state.onPage = { latest(it) }
    state.onTurn = { haptic.performHapticFeedback(HapticFeedbackType.SegmentTick) }
    LaunchedEffect(index) {
        val speed = if (state.thrownTo == index) state.speed else 0f
        state.thrownTo = -1
        state.speed = 0f
        if (state.at != index.toFloat()) state.glideTo(index, speed)
    }
    return state
}

/**
 * The library's pages side by side, as Pully's Inbox and Tours sit (Pully rule 107, H20): the next one is just past the
 * edge, a sideways drag anywhere a page does not use itself moves both with the finger, and letting go settles on the
 * nearer one, or on the next one the way a throw went. Nothing fades or blurs; the pages only move. A tap elsewhere
 * that changes the page glides the row there.
 *
 * The page at rest and, once the first frame has drawn, the ones beside it are composed, so a swipe starts on a page
 * that is already there; each keeps what it remembers (its scroll, its column count) while it is away. Two fingers are a
 * pinch, never a swipe. Each page gets its own [ShellImmersion], and the shell's follows the one on screen, so a gallery
 * left in reels on one page never hides the bar on another; a page in reels does not swipe.
 *
 * @param state Where the pages are, from [rememberSidePages].
 * @param count How many pages there are.
 * @param modifier Placement of the row.
 * @param content Draws page [Int].
 */
@Composable
fun SidePages(state: SidePagesState, count: Int, modifier: Modifier = Modifier, content: @Composable (Int) -> Unit) {
    val saved = rememberSaveableStateHolder()
    val outer = LocalShellImmersion.current
    val immersions = remember(count) { List(count) { ShellImmersion() } }
    if (outer != null) DisposableEffect(outer, immersions) {
        val removePage = outer.addPage {
            var amount = 0f
            immersions.forEachIndexed { page, immersion ->
                val near = 1f - abs(page - state.at)
                if (near > 0f) amount += immersion.amount() * near
            }
            amount
        }
        val removeStrip = outer.addStrip {
            var color = Color.Transparent
            immersions.forEachIndexed { page, immersion ->
                val near = 1f - abs(page - state.at)
                val own = immersion.strip()
                if (near > 0f && own != null && own.alpha * near > color.alpha) color = own.copy(alpha = own.alpha * near)
            }
            color
        }
        onDispose {
            removePage()
            removeStrip()
        }
    }
    val immersed by remember(immersions, state.index) { derivedStateOf { (immersions.getOrNull(state.index)?.amount() ?: 0f) > 0f } }
    var drawn by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { withFrameNanos { }; drawn = true }
    val shown by remember(count) { derivedStateOf { (0 until count).filter { abs(it - state.at).let { d -> d < 1f || (drawn && d <= 1f) } } } }
    // Only the change between moving and resting reaches a page, never each frame of a swipe.
    val rest = remember(state) { derivedStateOf { state.atRest } }
    val atRest = remember(rest) { { rest.value } }
    Box(
        modifier
            .clipToBounds()
            .onSizeChanged { state.width = it.width }
            .pointerInput(state) {
                // Watches, without taking, how many fingers are down, so a pinch on a page never turns it.
                awaitEachGesture {
                    awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                    state.pinched = false
                    do {
                        val event = awaitPointerEvent(PointerEventPass.Initial)
                        if (event.changes.count { it.pressed } >= 2) state.pinch()
                    } while (event.changes.any { it.pressed })
                }
            }
            .draggable(
                state.drag, Orientation.Horizontal, enabled = !immersed,
                onDragStarted = { state.started() },
                onDragStopped = { velocity -> state.stopped(velocity) },
            ),
    ) {
        CompositionLocalProvider(LocalSidePagesAtRest provides atRest) {
            for (page in shown) {
                key(page) {
                    // A page built ahead, wholly off screen, is not read out.
                    val away by remember(page) { derivedStateOf { abs(page - state.at) >= 1f } }
                    Box(
                        Modifier.fillMaxSize()
                            .graphicsLayer { translationX = (page - state.at) * (size.width + PAGE_GAP.toPx()) }
                            .then(if (away) Modifier.clearAndSetSemantics { } else Modifier),
                    ) {
                        CompositionLocalProvider(LocalShellImmersion provides immersions[page]) {
                            saved.SaveableStateProvider(page) { content(page) }
                        }
                    }
                }
            }
        }
    }
}

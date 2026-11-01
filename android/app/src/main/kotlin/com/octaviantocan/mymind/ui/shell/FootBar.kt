package com.octaviantocan.mymind.ui.shell

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.EnterExitState
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.core.animateDp
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandHorizontally
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.IntrinsicMeasurable
import androidx.compose.ui.layout.IntrinsicMeasureScope
import androidx.compose.ui.node.LayoutModifierNode
import androidx.compose.ui.layout.Measurable
import androidx.compose.ui.layout.MeasureResult
import androidx.compose.ui.layout.MeasureScope
import androidx.compose.ui.layout.layout
import androidx.compose.ui.node.ModifierNodeElement
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlin.math.roundToInt

// Ported from pulls.review ee5cf358333cb1170f1923c7118e12af3f265b74, ui/design/FootBar.kt.
/*
 * The one bar at the foot of the app (Q2). Pages do not draw their own search or Ask bar any more: each says what its
 * bar holds, and [FootBarLayer], drawn once over every page, keeps one pill in place while the pages slide under it. A
 * new page changes only what is inside the pill, its words fading out blurred and the next ones fading in, the way the
 * Inbox and Tours already share theirs ("one component that's kind of over the screen, and then we can swap the text
 * inside it with that nice blur fade ... we want that kind of seamlessness when we go into a PR as well": recording
 * 17 Oct 14:45, 01:06 to 01:51; Q2). A bar of another shape, the diff's file bar, takes the pill's place the same way.
 */

/** Every page's bar, newest last: the newest is the one shown, since a page that opens composes after the one it covers. */
@Stable
class FootBars {
    internal val slots = mutableStateListOf<FootSlot>()
    internal val top: FootSlot? get() = slots.lastOrNull()
}

/** The app's [FootBars], or none where a bar draws in place (the UI Book, a still). */
val LocalFootBars = staticCompositionLocalOf<FootBars?> { null }

/** The pill's field, so a tap anywhere on the pill puts the cursor in it rather than on the page under it (Q6). */
val LocalFootFocus = compositionLocalOf<FocusRequester?> { null }

/** One page's bar, as [FootPill] or [FootFree] last said. */
@Stable
internal class FootSlot {
    var key: Any by mutableStateOf(Unit)
    var free by mutableStateOf(false)
    var search by mutableStateOf(false)
    var button by mutableStateOf(false)
    var compact by mutableStateOf<String?>(null)
    var hug by mutableStateOf(false)
    var onExpand by mutableStateOf<(() -> Unit)?>(null)
    var veil by mutableStateOf<Modifier>(Modifier)
    var inset by mutableStateOf<Dp?>(null)
    var field by mutableStateOf(true)
    var circle by mutableStateOf<FootCircle?>(null)
    var trailing by mutableStateOf<(@Composable () -> Unit)?>(null)
    var content by mutableStateOf<@Composable RowScope.() -> Unit>({})
    var freeContent by mutableStateOf<@Composable () -> Unit>({})
    val focus = FocusRequester()

    /** A tap opened the short pill: its field takes the cursor once it is there. */
    var focusOnOpen = false
}

/** The width of the short pill that reads "Ask" (Q5). */
private val COMPACT_WIDTH = 104.dp

private val CIRCLE_GAP = 8.dp

/** Test tag of the one pill. */
const val FOOT_PILL_TAG = "foot-pill"

/**
 * The round button beside the pill. Pages that both have one share it: it stays in place and only its glyph changes,
 * blurring out and the next one in, the way the pill keeps its place while its words change (Tavi, 28 October 2026).
 *
 * @property icon Drawable of its glyph.
 * @property label What a tap does, read by accessibility services.
 * @property onClick Called when it is tapped.
 * @property iconSize Size of its glyph.
 */
@Immutable
class FootCircle(@DrawableRes val icon: Int, val label: String, val onClick: () -> Unit, val iconSize: Dp = 22.dp)

/**
 * A page's bar in the one pill: [content] is what sits in it, after a search glass when [search], with the pill's end
 * made for a round button when [button], or [inset] from both ends when set, as the gallery's tabs are. Without a [field]
 * to put the cursor in, a tap on the pill's edge does nothing. [circle] is the round button beside the pill that every
 * page with one shares. [trailing] sits beside the pill, as the Inbox's sort does, and leaves with the page. With
 * [compact] the pill is short and reads that one word, centred, until a tap ([onExpand]) opens it to the full width with
 * its field focused ("a shorter bar that says Ask. And when you tap on it ... it kind of expands", Q5).
 * With [hug] the pill is only as wide as what it holds and sits centred with its circle, growing and shrinking as its
 * contents do; the next bar that does not hug widens it to the full width, and back (the gallery's two icons opening
 * into search, Tavi's pick of 03 on 1 November 2026).
 * [veil] dims or holds the bar with its page. Outside a [FootBarLayer] the bar draws here, placed by [modifier].
 */
@Composable
fun FootPill(
    key: Any,
    modifier: Modifier = Modifier,
    search: Boolean = false,
    button: Boolean = false,
    compact: String? = null,
    onExpand: (() -> Unit)? = null,
    hug: Boolean = false,
    veil: Modifier = Modifier,
    inset: Dp? = null,
    field: Boolean = true,
    circle: FootCircle? = null,
    trailing: (@Composable () -> Unit)? = null,
    content: @Composable RowScope.() -> Unit,
) {
    val bars = LocalFootBars.current
    val slot = remember { FootSlot() }
    val fill: FootSlot.() -> Unit = {
        this.key = key
        free = false
        this.search = search
        this.button = button
        this.compact = compact
        this.onExpand = onExpand
        this.hug = hug
        this.veil = veil
        this.inset = inset
        this.field = field
        this.circle = circle
        this.trailing = trailing
        this.content = content
    }
    if (bars == null) {
        slot.fill()
        Box(modifier.floatingBar()) { PillBar(slot, rememberFootMotionEnabled()) }
        return
    }
    SideEffect { slot.fill() }
    Register(bars, slot)
}

/** A page's bar of its own shape (the diff's ‹ file ›), in the pill's place at the foot. */
@Composable
fun FootFree(key: Any, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    val bars = LocalFootBars.current
    if (bars == null) {
        Box(modifier.floatingBar()) { content() }
        return
    }
    val slot = remember { FootSlot() }
    SideEffect {
        slot.key = key
        slot.free = true
        slot.freeContent = content
    }
    Register(bars, slot)
}

@Composable
private fun Register(bars: FootBars, slot: FootSlot) {
    DisposableEffect(bars, slot) {
        bars.slots.add(slot)
        onDispose { bars.slots.remove(slot) }
    }
}

/**
 * The foot of the app: the pill of the page on top, or its own bar, resting where every bar rests ([floatingBar]). Draw
 * it over the pages and under anything that covers them (Settings, Commands, toasts).
 */
@Composable
fun FootBarLayer(bars: FootBars, modifier: Modifier = Modifier) {
    val moving = rememberFootMotionEnabled()
    val top = bars.top
    val pill = top?.takeIf { !it.free }
    // The pill last shown, kept while it fades out under a page with no bar or a bar of its own.
    val kept = remember { arrayOfNulls<FootSlot>(1) }
    if (pill != null) kept[0] = pill
    Box(modifier.floatingBar()) {
        AnimatedVisibility(pill != null, enter = barEnter(moving), exit = barExit(moving)) {
            kept[0]?.let { PillBar(it, moving) }
        }
        AnimatedContent(
            top?.takeIf { it.free },
            contentKey = { it?.key },
            transitionSpec = { blurFade(moving) },
            label = "foot-free",
        ) { slot ->
            if (slot != null) Box(Modifier.blurredBy(transition)) { slot.freeContent() }
        }
    }
}

/** The pill and what sits beside it. */
@Composable
private fun PillBar(slot: FootSlot, moving: Boolean) {
    val c = LibraryStyle.colors
    val short = slot.compact != null
    val open = animateFloatAsState(
        if (short) 0f else 1f,
        if (moving) tween(FootMotion.MEDIUM, easing = FootMotion.SmoothOut) else tween(0),
        label = "foot-open",
    )
    val hug = animateFloatAsState(
        if (slot.hug && !short) 1f else 0f,
        if (moving) tween(FootMotion.MEDIUM, easing = FootMotion.SmoothOut) else tween(0),
        label = "foot-hug",
    )
    val hugging by remember { derivedStateOf { hug.value > 0f } }
    // The hugging contents' width, kept while they fade out so the pill widens from where they were.
    val hugged = remember { IntArray(1) }
    // The field takes the cursor once the pill a tap opened is there.
    LaunchedEffect(slot, short) {
        if (short || !slot.focusOnOpen) return@LaunchedEffect
        slot.focusOnOpen = false
        withFrameNanos { }
        runCatching { slot.focus.requestFocus() }
    }
    val trailing = slot.trailing
    val keptTrailing = remember { arrayOfNulls<@Composable () -> Unit>(1) }
    if (trailing != null) keptTrailing[0] = trailing
    Row(slot.veil.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.Center) {
        // A hugging pill leaves the row's spare width to both sides, so it and its circle sit centred together.
        Box(Modifier.weight(1f, fill = !hugging), contentAlignment = Alignment.Center) {
            Box(
                Modifier
                    .layout { measurable, constraints ->
                        val full = constraints.maxWidth
                        val height = FieldHeight.roundToPx()
                        var width = mix(COMPACT_WIDTH.roundToPx().coerceAtMost(full), full, open.value)
                        val toward = hug.value
                        if (toward > 0f) {
                            val natural = measurable.maxIntrinsicWidth(height)
                            if (natural > 0) hugged[0] = natural
                            if (hugged[0] > 0) width = mix(width, hugged[0].coerceAtMost(full), toward)
                        }
                        val placeable = measurable.measure(Constraints.fixed(width, height))
                        layout(width, height) { placeable.place(0, 0) }
                    }
                    .testTag(FOOT_PILL_TAG)
                    .footShadow(CircleShape, FootShadow.Search)
                    .clip(CircleShape).background(c.raised)
                    // A tap anywhere on the pill is the pill's: it opens a short one, or puts the cursor in the field, and
                    // never reaches the page under it ("I keep trying to tap on the ask about this pull request, and it
                    // keeps getting me to the description": recording 17 Oct 14:45, 03:00 to 03:10; Q6).
                    .pointerInput(slot) {
                        detectTapGestures {
                            val expand = slot.onExpand
                            if (slot.compact != null && expand != null) {
                                slot.focusOnOpen = true
                                expand()
                            } else if (slot.field) {
                                runCatching { slot.focus.requestFocus() }
                            }
                        }
                    }
                    .then(
                        if (short) Modifier.semantics {
                            role = Role.Button
                            onClick(label = slot.compact) { slot.focusOnOpen = true; slot.onExpand?.invoke(); true }
                        } else Modifier,
                    ),
                contentAlignment = Alignment.Center,
            ) {
                AnimatedContent(
                    PillFace(slot, short),
                    contentKey = { it.key },
                    transitionSpec = { blurFade(moving) },
                    contentAlignment = Alignment.Center,
                    label = "foot-pill",
                ) { face ->
                    val wish = if (face.slot.hug && !face.short) Modifier else Modifier.then(NoWidthWish)
                    Box(Modifier.fillMaxSize().then(wish).blurredBy(transition), contentAlignment = Alignment.Center) {
                        if (face.short) {
                            Text(face.slot.compact.orEmpty(), style = LibraryStyle.body.copy(fontWeight = FontWeight.Medium), color = c.text, maxLines = 1)
                        } else {
                            Row(
                                Modifier.fillMaxSize().padding(
                                    start = face.slot.inset ?: FieldInset,
                                    end = face.slot.inset ?: if (face.slot.button) FieldButtonInset else FieldInset,
                                ),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(FieldGap),
                            ) {
                                if (face.slot.search) FootIcon(R.drawable.ic_library_search, tint = c.muted, size = 20.dp)
                                androidx.compose.runtime.CompositionLocalProvider(LocalFootFocus provides face.slot.focus) { face.slot.content(this) }
                            }
                        }
                    }
                }
            }
        }
        val circle = slot.circle
        val keptCircle = remember { arrayOfNulls<FootCircle>(1) }
        if (circle != null) keptCircle[0] = circle
        AnimatedVisibility(circle != null, enter = besideEnter(moving), exit = besideExit(moving)) {
            keptCircle[0]?.let { CircleBar(it, moving, Modifier.padding(start = CIRCLE_GAP)) }
        }
        AnimatedVisibility(trailing != null, enter = besideEnter(moving), exit = besideExit(moving)) { keptTrailing[0]?.invoke() }
    }
}

/** The shared round button: one surface that stays while a page's glyph blur-fades into the next page's. */
@Composable
private fun CircleBar(circle: FootCircle, moving: Boolean, modifier: Modifier = Modifier) {
    val c = LibraryStyle.colors
    FootPressTheme {
        Box(
            modifier.size(FieldHeight).footShadow(CircleShape, FootShadow.Search).clip(CircleShape).background(c.raised)
                .semantics { contentDescription = circle.label }
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = circle.onClick),
            contentAlignment = Alignment.Center,
        ) {
            AnimatedContent(
                circle,
                contentKey = { it.icon },
                transitionSpec = { blurFade(moving) },
                contentAlignment = Alignment.Center,
                label = "foot-circle",
            ) { shown ->
                Box(Modifier.blurredBy(transition), contentAlignment = Alignment.Center) { FootIcon(shown.icon, size = shown.iconSize) }
            }
        }
    }
}

/** Something beside the pill arriving with its page, and leaving with it. */
private fun besideEnter(moving: Boolean): EnterTransition =
    if (!moving) EnterTransition.None else fadeIn(tween(FootMotion.FAST, easing = FootMotion.SmoothOut)) + expandHorizontally(tween(FootMotion.FAST, easing = FootMotion.SmoothOut))
private fun besideExit(moving: Boolean): ExitTransition =
    if (!moving) ExitTransition.None else fadeOut(tween(FootMotion.QUICK, easing = FootMotion.SmoothOut)) + shrinkHorizontally(tween(FootMotion.QUICK, easing = FootMotion.SmoothOut))

private fun mix(from: Int, to: Int, fraction: Float): Int = (from + (to - from) * fraction).roundToInt()

/**
 * Tells a hugging pill that a face wants no width of its own: a field fading in or out under a hugging bar does not
 * widen the pill, which follows only the hugging contents and its own animation toward the full width.
 */
private object NoWidthWish : ModifierNodeElement<NoWidthWishNode>() {
    override fun create() = NoWidthWishNode()
    override fun update(node: NoWidthWishNode) = Unit
    override fun hashCode(): Int = 0
    override fun equals(other: Any?): Boolean = other === this
}

/** The node of [NoWidthWish]: it measures and places as if it were not there. */
private class NoWidthWishNode : Modifier.Node(), LayoutModifierNode {
    override fun MeasureScope.measure(measurable: Measurable, constraints: Constraints): MeasureResult {
        val placeable = measurable.measure(constraints)
        return layout(placeable.width, placeable.height) { placeable.place(0, 0) }
    }

    override fun IntrinsicMeasureScope.minIntrinsicWidth(measurable: IntrinsicMeasurable, height: Int): Int = 0

    override fun IntrinsicMeasureScope.maxIntrinsicWidth(measurable: IntrinsicMeasurable, height: Int): Int = 0
}

/** What the pill shows: a page's field, or its one word while short. Short pills of any page are one face, so "Ask" never fades into "Ask". */
private class PillFace(val slot: FootSlot, val short: Boolean) {
    val key: Any get() = if (short) "short:${slot.compact}" else slot.key
}

/** Words leave fading out and blurring and the next arrive sharpening, in place: nothing slides, the pill stays (Q2). */
private fun <S> AnimatedContentTransitionScope<S>.blurFade(moving: Boolean): ContentTransform {
    if (!moving) return EnterTransition.None togetherWith ExitTransition.None
    return (fadeIn(tween(FootMotion.FAST, easing = FootMotion.SmoothOut)) togetherWith fadeOut(tween(FootMotion.QUICK, easing = FootMotion.SmoothOut)))
        .using(SizeTransform(clip = false))
}

/** The blur that goes with [blurFade]: sharp while shown, [FootMotion.BlurMedium] on the way in or out. */
@Composable
private fun Modifier.blurredBy(transition: androidx.compose.animation.core.Transition<EnterExitState>): Modifier {
    val blur by transition.animateDp(transitionSpec = { tween(FootMotion.FAST, easing = FootMotion.SmoothOut) }, label = "foot-blur") {
        if (it == EnterExitState.Visible) 0.dp else FootMotion.BlurMedium
    }
    return footBlur(blur)
}

/** A bar arriving where there was none, and leaving with a page that has none: a quick fade. */
private fun barEnter(moving: Boolean): EnterTransition = if (!moving) EnterTransition.None else fadeIn(tween(FootMotion.FAST, easing = FootMotion.SmoothOut))
private fun barExit(moving: Boolean): ExitTransition = if (!moving) ExitTransition.None else fadeOut(tween(FootMotion.QUICK, easing = FootMotion.SmoothOut))

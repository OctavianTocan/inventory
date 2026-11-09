package com.octaviantocan.mymind.ui.library

import androidx.compose.runtime.CompositionLocalProvider
import com.octaviantocan.mymind.ui.card.LocalSheetMotionEnabled
import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.spring
import androidx.compose.foundation.MutatePriority
import androidx.compose.foundation.clickable
import androidx.compose.foundation.overscroll
import androidx.compose.foundation.rememberOverscrollEffect
import androidx.compose.foundation.gestures.FlingBehavior
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.ScrollScope
import androidx.compose.foundation.gestures.ScrollableDefaults
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.gestures.scrollable
import androidx.compose.foundation.gestures.stopScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.lerp
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.layer.GraphicsLayer
import androidx.compose.ui.graphics.layer.drawLayer
import androidx.compose.ui.graphics.rememberGraphicsLayer
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.lerp as lerpColor
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.Measurable
import androidx.compose.ui.layout.Placeable
import androidx.compose.ui.layout.layout
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.ScrollAxisRange
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.scrollToIndex
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.verticalScrollAxisRange
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import coil3.SingletonImageLoader
import coil3.compose.LocalPlatformContext
import coil3.memory.MemoryCache
import coil3.toBitmap
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.card.sheetEdgeFade
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.shell.LocalShellImmersion
import com.octaviantocan.mymind.ui.shell.LocalSidePagesAtRest
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.math.sqrt

/** Space on each side of the cards, the web showcase's on a phone; Pinned starts at the same inset. */
private val MOSAIC_MARGIN = 16.dp

/** Space between the page's header, such as its title or Pinned, and the first cards. */
private val MOSAIC_HEADER_GAP = 4.dp
private const val MOSAIC_HEADER = "gallery-header"
private const val MOSAIC_END = "gallery-end"
private const val MOSAIC_ROW = "row:"
private const val HOP_NANOS = 130 * 1_000_000L
private const val FRAME_NANOS = 16_666_667L
private val BACKDROP_BLUR = 14.dp
private const val WAVE_DELAY = 0.5f

/** A fling slower than this, in dp a second, settles back on the page it started from unless dragged past half of it. */
private val REEL_FLING = 400.dp

/** How stiffly a fling in reels comes to rest on its page: quick, with no bounce. */
private const val REEL_STIFFNESS = 900f

/** The colour behind a page in reels, so a picture shown whole at its own shape has no paper around it. */
private val REEL_PAPER = Color.Black
private data class MosaicAnchor(val cardId: String, val viewport: Offset, val fraction: Offset)
private data class MosaicBodyShape(val top: Float, val bottom: Float) : Shape {
    override fun createOutline(size: Size, layoutDirection: LayoutDirection, density: Density): Outline =
        Outline.Rectangle(Rect(0f, top.coerceIn(0f, size.height), size.width, bottom.coerceIn(top.coerceIn(0f, size.height), size.height)))
}
private data class MosaicLoadPosition(val count: Int, val last: Int, val total: Int, val ready: Boolean)

/**
 * Virtualized bricks that follow the fingers between cached density layouts. Stable card keys retain content while
 * only the bounded visible window is measured at its real, interpolated size. When [mosaic] has reels, its one column
 * is them: one fuller card per page, full screen, a swipe moving one page and Back or a pinch closing them.
 *
 * @param feed The cards and their loading state.
 * @param onLoadMore Called when the gallery nears its last card.
 * @param onOpenCard Called with a tapped card.
 * @param modifier Modifier for the gallery.
 * @param state Scroll position, kept by the caller to survive the gallery leaving the screen.
 * @param mosaic Density, kept by the caller for the same reason.
 * @param onLongPressCard Called with a held card, or null when holding does nothing.
 * @param header Scrolls above the cards, such as the page title.
 * @param tiles Draws and shapes the cards; library cards by default.
 * @param label What the gallery is called to accessibility services.
 * @param choice Where hold-and-slide gathers finds; null leaves holding to open the card menu at once. While it holds
 * finds, a tap adds or drops a find instead of opening it, and a held find opens its menu when let go without sliding.
 * @param onMoveCard Called when a held card is dragged to a new place and let go, with the card and the card it now
 *   follows (null when it is now first); null leaves the cards' order alone. While it is set, holding a card and
 *   letting go without moving opens [onLongPressCard] on release, and holding then dragging moves the card in the grid
 *   as it is, the other cards making way under the finger (Octavian, 6 November 2026: "Reordering should happen in the
 *   shape they're in").
 */
@Composable
internal fun LibraryMosaic(
    feed: FeedState,
    onLoadMore: () -> Unit,
    onOpenCard: (Card) -> Unit,
    modifier: Modifier = Modifier,
    state: LazyListState = rememberLazyListState(),
    mosaic: MosaicState = rememberMosaicState(),
    onLongPressCard: ((Card) -> Unit)? = null,
    header: (@Composable () -> Unit)? = null,
    tiles: MosaicTiles = GalleryMosaicTiles,
    label: String = "Card gallery",
    choice: CardChoice? = null,
    onMoveCard: ((Card, String?) -> Unit)? = null,
) {
    val colors = LibraryStyle.colors
    val backdropShown = MymindTheme.palette.backdrop != null
    val density = LocalDensity.current
    val moving = rememberSheetMotionEnabled()
    val pagesAtRest = LocalSidePagesAtRest.current
    val scope = rememberCoroutineScope()
    val overscroll = rememberOverscrollEffect()
    var anchor by remember { mutableStateOf<MosaicAnchor?>(null) }
    var settling by remember { mutableStateOf<Job?>(null) }
    var hopEasing by remember { mutableStateOf<Job?>(null) }
    var headerHeight by remember { mutableIntStateOf(0) }
    var footerHeight by remember { mutableIntStateOf(0) }
    var restingX by remember { mutableFloatStateOf(0f) }
    var anchorXWeight by remember { mutableFloatStateOf(1f) }
    var pivotId by remember { mutableStateOf<String?>(null) }
    val waves = remember { MosaicWaves() }
    val pinchSpeed = remember { MosaicSpeed() }
    val latestFeed by rememberUpdatedState(feed)
    val latestMore by rememberUpdatedState(onLoadMore)
    val headerItems = if (header == null) 0 else 1
    val haptics = LocalHapticFeedback.current
    // The cards in the order a drag has put them, until the feed brings that order back; null shows the feed's order.
    var arranged by remember { mutableStateOf<List<Card>?>(null) }
    // The card being dragged, where the finger is, and where in the card's tile the finger took hold of it.
    var liftedId by remember { mutableStateOf<String?>(null) }
    var finger by remember { mutableStateOf(Offset.Zero) }
    var grab by remember { mutableStateOf(Offset.Zero) }
    // The card the dragged one last swapped places with, so it does not swap back while the finger is still over it.
    val swapped = remember { arrayOfNulls<String>(1) }
    LaunchedEffect(feed.cards) { if (liftedId == null) arranged = null }
    val latestMove by rememberUpdatedState(onMoveCard)
    val latestHold by rememberUpdatedState(onLongPressCard)
    val haptic = LocalHapticFeedback.current
    val cards = arranged ?: feed.cards
    BoxWithConstraints(modifier.fillMaxSize()) {
        // The grid's own width: everything below works inside the margins, and touches are moved in by [margin].
        val margin = with(density) { MOSAIC_MARGIN.toPx() }
        val width = with(density) { maxWidth.toPx() } - 2 * margin
        val height = with(density) { maxHeight.toPx() }
        val gaps = remember(maxWidth, density) {
            (1..4).map { columns -> with(density) { mosaicGapDp(columns, (maxWidth - MOSAIC_MARGIN * 2).value).dp.toPx() } }
        }
        val bottomPadding = with(density) { 156.dp.toPx() }
        val page = if (mosaic.reels) height else 0f
        val layouts = remember(cards, width, gaps, tiles, page, margin) { MosaicLayouts(cards, width, gaps, tiles, page, margin) }

        fun clampOrigin(value: Float): Float {
            // Reels have no header above their first page and no room after their last.
            val open = 1f - mosaic.reel
            val first = if (header == null) 0f else -headerHeight * open
            val footer = if (latestFeed.loading || latestFeed.error != null) footerHeight else 0
            val last = (layouts.height(mosaic.position) + footer + bottomPadding * open - height).coerceAtLeast(first)
            return value.coerceIn(first, last)
        }
        fun scrollOrigin(): Float {
            if (layouts.level(1).cells.isEmpty()) return 0f
            val visible = state.layoutInfo.visibleItemsInfo
            for (item in visible) {
                val id = (item.key as? String)?.takeIf { it.startsWith(MOSAIC_ROW) }?.removePrefix(MOSAIC_ROW)
                val index = id?.let(layouts.byId::get) ?: continue
                return layouts.cell(index, mosaic.position).top - item.offset
            }
            visible.firstOrNull { it.key == MOSAIC_HEADER }?.let { return -(it.offset + it.size).toFloat() }
            visible.firstOrNull { it.key == "gallery-footer" }?.let { return layouts.height(mosaic.position) - it.offset }
            return 0f
        }
        fun scrollProgress(): Float {
            val index = state.firstVisibleItemIndex
            val size = state.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index }?.size ?: 0
            return index + state.firstVisibleItemScrollOffset.toFloat() / (size + 1)
        }
        fun origin(): Float {
            if (layouts.level(1).cells.isEmpty()) return 0f
            val focal = anchor ?: return scrollOrigin()
            val index = layouts.byId[focal.cardId] ?: return scrollOrigin()
            val cell = layouts.cell(index, mosaic.position)
            val held = clampOrigin(cell.top + focal.fraction.y * cell.height - focal.viewport.y)
            // On the way into reels the card under the fingers comes to fill the screen, its page's top at the top.
            val reel = mosaic.reel
            return if (reel <= 0f) held else held + (clampOrigin(cell.top) - held) * reel
        }
        fun alignAnchor() {
            val focal = anchor ?: return
            val index = layouts.byId[focal.cardId] ?: return
            val reference = layouts.level(mosaic.referenceColumns)
            if (reference.rows.isEmpty()) return
            val row = reference.rowOf[index]
            val top = layouts.cell(reference.starts[row], mosaic.position).top
            state.requestScrollToItem(row + headerItems, (origin() - top).roundToInt())
        }
        fun horizontalShift(y: Float): Float {
            if (layouts.level(1).cells.isEmpty()) return 0f
            val focal = anchor ?: return restingX
            val index = layouts.byId[focal.cardId] ?: return restingX
            val cell = layouts.cell(index, mosaic.position)
            return (focal.viewport.x - cell.left - focal.fraction.x * cell.width) * anchorXWeight * (1f - mosaic.reel)
        }
        fun pivot(): Int = (anchor?.cardId ?: pivotId)?.let(layouts.byId::get)
            ?: state.layoutInfo.visibleItemsInfo.firstNotNullOfOrNull { item ->
                (item.key as? String)?.removePrefix(MOSAIC_ROW)?.let(layouts.byId::get)
            } ?: 0
        fun projection(columns: Int): MosaicProjection {
            val projected = layouts.projection(columns, mosaic.position, pivot())
            if (projected.scale >= 1f) return projected
            // A grid drawn below its own size would leave empty margins; keep it whole around the focal corner.
            val focal = layouts.cell(pivot(), mosaic.position)
            val base = layouts.level(columns).cells[pivot()]
            return MosaicProjection(layouts, columns, 1f, Offset(focal.left - base.left, focal.top - base.top))
        }
        /** Moves a projection sideways only as far as its edges still cover the viewport. */
        fun coverShift(projected: MosaicProjection, shift: Float): Float {
            val low = width - width * projected.scale - projected.offset.x
            val high = -projected.offset.x
            return if (low <= high) shift.coerceIn(low, high) else (low + high) / 2f
        }
        fun frame(): MosaicFrame {
            val y = origin()
            val shift = horizontalShift(y)
            val between = mosaic.between()
            val focal = pivot()
            val from = layouts.projection(between.from, mosaic.position, focal)
            val to = layouts.projection(between.to, mosaic.position, focal)
            val t = between.progress
            // Both grids are drawn at the fingers' size; together their edges still cover the width.
            val left = from.offset.x + (to.offset.x - from.offset.x) * t
            val right = left + width * (from.scale + (to.scale - from.scale) * t)
            val x = if (width - right <= -left) shift.coerceIn(width - right, -left) else (width - right - left) / 2f
            val fillFrom = projection(between.from)
            val fillTo = projection(between.to)
            val wrap = width + maxOf(layouts.level(between.from).gap, layouts.level(between.to).gap)
            // The grids a stop past either one, whose cards a pinch reaches next, so they are composed before it does.
            val beyond = listOf(minOf(between.from, between.to) - 1, maxOf(between.from, between.to) + 1)
                .filter { it in 2..4 }.map { layouts.projection(it, mosaic.position, focal) }
            return MosaicFrame(from, to, x, y, t, waves.of(layouts, between, focal, anchor, height), wrap,
                fillFrom, coverShift(fillFrom, shift), fillTo, coverShift(fillTo, shift), held = mosaic.isPinching, beyond = beyond)
        }
        fun capture(focal: Offset?): MosaicAnchor? {
            if (layouts.level(1).cells.isEmpty()) return null
            val drawn = frame()
            val point = focal?.takeIf { it.x.isFinite() && it.y.isFinite() }?.minus(Offset(margin, 0f)) ?: Offset.Zero
            var nearest = -1
            var distance = Float.MAX_VALUE
            for (index in drawn.window(height)) {
                val cell = drawn.cell(index)
                if (cell.bottom <= 0f || cell.top >= height) continue
                val dx = point.x - point.x.coerceIn(cell.left, cell.right)
                val dy = point.y - point.y.coerceIn(cell.top, cell.bottom)
                if (dx * dx + dy * dy < distance) {
                    distance = dx * dx + dy * dy
                    nearest = index
                }
            }
            if (nearest < 0) return null
            val cell = drawn.cell(nearest)
            val at = Offset(point.x.coerceIn(cell.left, cell.right), point.y.coerceIn(cell.top, cell.bottom))
            return MosaicAnchor(layouts.cards[nearest].id, at,
                Offset((at.x - cell.left) / cell.width, (at.y - cell.top) / cell.height))
        }
        /** The card whose tile is under [point], in the gallery's own coordinates, and that tile's place there. */
        fun tileAt(point: Offset): Pair<Int, Rect>? {
            if (layouts.level(1).cells.isEmpty()) return null
            val drawn = frame()
            val local = point - Offset(margin, 0f)
            return drawn.window(height).firstOrNull { drawn.cell(it).contains(local) }?.let { it to drawn.cell(it).translate(margin, 0f) }
        }
        /** Lands on the current columns from where the gallery is, starting at [speed] stops per millisecond. */
        fun settle(speed: Float = 0f) {
            settling?.cancel()
            settling = scope.launch {
                val start = mosaic.position
                val xWeight = anchorXWeight
                val way = mosaic.columns - start
                val millis = settleMillis(way)
                if (moving && millis >= 1f) {
                    val lead = settleLead(speed, way, millis)
                    // The frame that shows the landing counts as its first, as a hop's does, so the cards never stand
                    // still for a frame between the fingers and the landing.
                    var began = -1L
                    var done = 0f
                    while (done < 1f) {
                        val now = withFrameNanos { it }
                        // A frame already sent can arrive after the gallery is gone; it must not undo the stop it saved.
                        ensureActive()
                        if (began < 0) began = now - FRAME_NANOS
                        done = settleProgress((now - began) / (millis * 1_000_000f), lead)
                        mosaic.position = start + way * done
                        anchorXWeight = xWeight * (1f - done)
                        alignAnchor()
                    }
                } else {
                    mosaic.position = mosaic.columns.toFloat()
                    anchorXWeight = 0f
                    alignAnchor()
                }
                restingX = 0f
                mosaic.referenceColumns = mosaic.columns
                alignAnchor()
                anchor = null
                // Pictures take their new decode size a frame after the cards come to rest, so no moving frame waits on it.
                if (moving) withFrameNanos { }
                mosaic.pictureColumns = mosaic.columns
            }
        }
        fun resize(change: () -> Boolean): Boolean {
            if (mosaic.isPinching) return false
            val focal = capture(null)
            val previous = mosaic.position
            if (!change()) return false
            settling?.cancel()
            anchor = focal
            pivotId = focal?.cardId
            anchorXWeight = 0f
            if (moving) mosaic.position = previous
            alignAnchor()
            settle()
            return true
        }
        /** Eases the hop a pinch has just started; the frame that shows it counts as its first. */
        fun hop() {
            if (!mosaic.hopping || hopEasing?.isActive == true) return
            if (!moving) {
                mosaic.advanceHop(1f)
                alignAnchor()
                return
            }
            hopEasing = scope.launch {
                var current = -1
                var start = 0L
                while (mosaic.hopping) {
                    val now = withFrameNanos { it }
                    if (mosaic.hops != current) {
                        current = mosaic.hops
                        start = now - FRAME_NANOS
                    }
                    mosaic.advanceHop((now - start).toFloat() / HOP_NANOS)
                    alignAnchor()
                }
            }
        }
        // Lambdas, not references such as ::capture: the compiler keeps a local function's first reference for good, so
        // a gallery first shown before its cards arrived found no card under the fingers and never pinched.
        val captureLatest by rememberUpdatedState<(Offset?) -> MosaicAnchor?> { focal -> capture(focal) }
        val hopLatest by rememberUpdatedState<() -> Unit> { hop() }
        val alignLatest by rememberUpdatedState<() -> Unit> { alignAnchor() }
        val settleLatest by rememberUpdatedState<(Float) -> Unit> { speed -> settle(speed) }
        /** The index of the card whose tile is under [point] of the gallery, or null over a gap or the header. */
        fun cardAt(point: Offset): Int? {
            if (layouts.level(1).cells.isEmpty()) return null
            val drawn = frame()
            val at = point - Offset(margin, 0f)
            return drawn.window(height).firstOrNull { drawn.cell(it).contains(at) }
        }
        val sweepLatest by rememberUpdatedState(SweepTarget(
            cards = { layouts.cards },
            cardAt = { point -> cardAt(point) },
            enabled = { !mosaic.isPinching && mosaic.reel == 0f && mosaic.position == mosaic.columns.toFloat() },
            scrollBy = { delta -> state.scrollBy(delta) },
            onHold = onLongPressCard,
        ))
        /** In reels at rest, how far a fling at [velocity] must scroll to land on a page's top; null elsewhere. */
        fun reelFling(velocity: Float): Float? {
            if (!mosaic.inReels || mosaic.position != 1f || mosaic.isPinching || layouts.page <= 0f || layouts.cards.isEmpty()) return null
            val origin = scrollOrigin()
            val at = origin / layouts.page
            val threshold = with(density) { REEL_FLING.toPx() }
            val target = when {
                velocity > threshold -> floor(at) + 1f
                velocity < -threshold -> ceil(at) - 1f
                else -> at.roundToInt().toFloat()
            }.coerceIn(0f, layouts.cards.lastIndex.toFloat())
            return target * layouts.page - origin
        }
        val reelFlingLatest by rememberUpdatedState<(Float) -> Float?> { velocity -> reelFling(velocity) }
        val freeFling = ScrollableDefaults.flingBehavior()
        val fling = remember(freeFling) { MosaicFling(freeFling) { velocity -> reelFlingLatest(velocity) } }
        BackHandler(enabled = mosaic.inReels) { resize(mosaic::smaller) }
        val immersion = LocalShellImmersion.current
        if (mosaic.reels && immersion != null) DisposableEffect(immersion, mosaic) {
            val remove = immersion.addPage { mosaic.reel }
            onDispose { remove() }
        }
        val latestLayouts by rememberUpdatedState(layouts)
        val latestHeaderItems by rememberUpdatedState(headerItems)
        val tileAtLatest by rememberUpdatedState<(Offset) -> Pair<Int, Rect>?> { point -> tileAt(point) }
        // Moves the dragged card to the place of the card under the finger, the others making way.
        fun follow() {
            val id = liftedId ?: return
            val over = tileAtLatest(finger)?.let { latestLayouts.cards[it.first].id }
            if (over == null || over == id) {
                swapped[0] = null
                return
            }
            if (over == swapped[0]) return
            val list = (arranged ?: latestFeed.cards).toMutableList()
            val from = list.indexOfFirst { it.id == id }
            val to = list.indexOfFirst { it.id == over }
            if (from < 0 || to < 0) return
            list.add(to, list.removeAt(from))
            arranged = list
            swapped[0] = over
            haptic.performHapticFeedback(HapticFeedbackType.SegmentFrequentTick)
        }
        // Lets go of the dragged card: a card held and let go without moving opens its menu; one moved keeps its place.
        fun drop(card: Card, from: Int, moved: Boolean) {
            liftedId = null
            swapped[0] = null
            val list = arranged
            val to = list?.indexOfFirst { it.id == card.id } ?: -1
            if (!moved || list == null || to < 0 || to == from) {
                arranged = null
                if (!moved) latestHold?.invoke(card)
                return
            }
            latestMove?.invoke(card, list.getOrNull(to - 1)?.id)
        }
        // While a card is dragged near the top or bottom edge, the gallery scrolls under it.
        if (onMoveCard != null) LaunchedEffect(liftedId) {
            if (liftedId == null) return@LaunchedEffect
            val edge = with(density) { MOVE_EDGE.toPx() }
            // Waits without asking for frames until the finger reaches an edge, then scrolls a step a frame.
            while (true) {
                val step = snapshotFlow {
                    when {
                        finger.y < edge -> -MOVE_SCROLL_STEP
                        finger.y > height - edge -> MOVE_SCROLL_STEP
                        else -> 0f
                    }
                }.first { it != 0f }
                withFrameNanos { }
                // At the end the gallery cannot scroll, so it waits for the finger to move before trying again.
                if (state.scrollBy(step) != 0f) follow() else snapshotFlow { finger }.drop(1).first()
            }
        }
        DisposableEffect(mosaic, state) {
            onDispose {
                settling?.cancel()
                if (anchor == null) anchor = captureLatest(null)
                mosaic.cancelPinch()
                mosaic.position = mosaic.columns.toFloat()
                mosaic.referenceColumns = mosaic.columns
                mosaic.pictureColumns = mosaic.columns
                anchorXWeight = 0f
                restingX = 0f
                alignLatest()
                anchor = null
            }
        }
        LaunchedEffect(layouts) {
            if (anchor?.cardId !in layouts.byId) anchor = null
            alignAnchor()
        }
        LaunchedEffect(state) {
            snapshotFlow {
                val current = latestFeed
                MosaicLoadPosition(current.cards.size, state.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1,
                    latestLayouts.level(mosaic.referenceColumns).rows.size + latestHeaderItems, !current.loading && !current.refreshing && !current.endReached && current.error == null)
            }.distinctUntilChanged().filter { it.ready && it.last >= it.total - 3 }.collect { latestMore() }
        }
        Box(Modifier.fillMaxSize().clipToBounds()
            .semantics {
                contentDescription = label
                stateDescription = if (mosaic.inReels) "Full screen" else "${mosaic.columns} columns"
                verticalScrollAxisRange = ScrollAxisRange(
                    value = { scrollProgress() },
                    maxValue = { if (state.canScrollForward) state.layoutInfo.totalItemsCount.toFloat() else scrollProgress() },
                    reverseScrolling = false,
                )
                scrollToIndex { index ->
                    if (index < 0 || index >= layouts.level(mosaic.referenceColumns).rows.size + headerItems) false else {
                        scope.launch { state.scrollToItem(index) }
                        true
                    }
                }
                customActions = listOf(
                    CustomAccessibilityAction("Larger cards") { resize(mosaic::larger) },
                    CustomAccessibilityAction("Smaller cards") { resize(mosaic::smaller) },
                )
            }
            .overscroll(overscroll)
            .scrollable(state, Orientation.Vertical, reverseDirection = true, enabled = !mosaic.isPinching, overscrollEffect = overscroll,
                flingBehavior = fling)
            .pointerInput(mosaic, state) {
                try {
                    awaitEachGesture {
                        val firstDown = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                        // A press stops a landing where it is, so the card under the finger stays there until up.
                        settling?.cancel()
                        var pinched = false
                        var dragged = false
                        var released = firstDown.uptimeMillis
                        var pointers = emptySet<androidx.compose.ui.input.pointer.PointerId>()
                        do {
                            val event = awaitPointerEvent(PointerEventPass.Initial)
                            released = event.changes.maxOf { it.uptimeMillis }
                            val pressed = event.changes.filter { it.pressed }.map { it.id }.toSet()
                            if (!pinched && !dragged && pressed.size == 1) {
                                val point = event.changes.first { it.pressed }.position
                                if ((point - firstDown.position).getDistance() > viewConfiguration.touchSlop) {
                                    dragged = true
                                    restingX = horizontalShift(origin())
                                    anchor = null
                                }
                            }
                            if (pressed.size >= 2) {
                                val point = event.changes.filter { it.pressed }.fold(Offset.Zero) { sum, change -> sum + change.position } / pressed.size.toFloat()
                                if (!pinched || pressed != pointers || anchor?.cardId !in latestLayouts.byId) {
                                    pinched = true
                                    anchor = captureLatest(point)
                                    pivotId = anchor?.cardId
                                    anchorXWeight = 1f
                                    mosaic.beginPinch()
                                    hopLatest()
                                    pinchSpeed.clear()
                                    pinchSpeed.add(released, mosaic.position)
                                    scope.launch(start = CoroutineStart.UNDISPATCHED) { state.stopScroll(MutatePriority.PreventUserInput) }
                                } else {
                                    // The focal point stays where the fingers met: a pinch resizes but never pans.
                                    mosaic.zoomPinch(event.calculateZoom())
                                    alignLatest()
                                    hopLatest()
                                    pinchSpeed.add(released, mosaic.position)
                                }
                            }
                            if (pinched) event.changes.forEach { it.consume() }
                            pointers = pressed
                        } while (event.changes.any { it.pressed })
                        if (pinched) {
                            mosaic.finishPinch()
                            settleLatest(pinchSpeed.at(released, mosaic.position))
                        } else {
                            val needsSettle = mosaic.position != mosaic.columns.toFloat() || anchor != null
                            if (needsSettle && anchor == null) anchor = captureLatest(null)
                            if (needsSettle) settleLatest(0f)
                        }
                    }
                } finally {
                    mosaic.cancelPinch()
                }
            }
            .then(if (choice == null) Modifier else Modifier.choiceSweep(choice, { sweepLatest }, scope, haptics))
            .then(if (onMoveCard == null) Modifier else Modifier.pointerInput(mosaic, state) {
                awaitEachGesture {
                    val down = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                    if (mosaic.reel > 0f || mosaic.isPinching) return@awaitEachGesture
                    // A hold is a finger kept still for the long-press time; moving first scrolls, a second finger pinches.
                    val broke = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                        while (true) {
                            val event = awaitPointerEvent(PointerEventPass.Initial)
                            val change = event.changes.firstOrNull { it.id == down.id }
                            if (change == null || !change.pressed || event.changes.count { it.pressed } != 1 ||
                                (change.position - down.position).getDistance() > viewConfiguration.touchSlop) break
                        }
                        true
                    }
                    if (broke != null) return@awaitEachGesture
                    val (index, cell) = tileAtLatest(down.position) ?: return@awaitEachGesture
                    val card = latestLayouts.cards[index]
                    val from = (arranged ?: latestFeed.cards).indexOfFirst { it.id == card.id }
                    grab = down.position - cell.topLeft
                    finger = down.position
                    liftedId = card.id
                    haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                    var moved = false
                    try {
                        while (true) {
                            val event = awaitPointerEvent(PointerEventPass.Initial)
                            val change = event.changes.firstOrNull { it.id == down.id } ?: break
                            event.changes.forEach { it.consume() }
                            if (!change.pressed) break
                            finger = change.position
                            if (!moved && (finger - down.position).getDistance() > viewConfiguration.touchSlop) moved = true
                            if (moved) follow()
                        }
                    } finally {
                        drop(card, from, moved)
                    }
                }
            })
            .sheetEdgeFade(top = 0.dp, bottom = 130.dp, color = colors.background, bottomStrength = { 1f - mosaic.reel },
                // While the cards scroll or move under the fingers the fade is not blurred, so a frame draws the gallery
                // once rather than once more for each blur layer.
                blurred = { !state.isScrollInProgress && !mosaic.isPinching && mosaic.position == mosaic.columns.toFloat() && pagesAtRest() })) {
            LazyColumn(modifier = Modifier.fillMaxSize(), state = state, userScrollEnabled = false) {
                if (header != null) item(key = MOSAIC_HEADER, contentType = "header") {
                    Column(Modifier.collapsing { 1f - mosaic.reel }.onSizeChanged { headerHeight = it.height }.padding(bottom = MOSAIC_HEADER_GAP)) { header() }
                }
                // These semantic-free slots retain LazyListState and fling/viewport bookkeeping while cards regroup. The
                // list reads its rows as it measures, so a landing's new rows and its scroll request reach it in the same
                // pass: read while composing, the request met the old rows and the gallery jumped once it had landed.
                val reference = mosaic.referenceColumns
                itemsIndexed(layouts.level(reference).rows, key = { _, row -> "$MOSAIC_ROW${row.key}" }, contentType = { _, _ -> "bricks" }) { index, _ ->
                    Layout(content = {}, modifier = Modifier.fillMaxWidth()) { _, constraints ->
                        layout(constraints.maxWidth, layouts.slotHeight(index, reference, mosaic.position).roundToInt()) {}
                    }
                }
                if (feed.cards.isNotEmpty() && (feed.loading || feed.error != null)) item(key = "gallery-footer", contentType = "footer") {
                    Column(Modifier.fillMaxWidth().onSizeChanged { footerHeight = it.height }.padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        if (feed.loading) LoadingSpinner(color = colors.accent)
                        feed.error?.let { message ->
                            Text(message, color = colors.muted, style = LibraryStyle.label)
                            Text("Retry", Modifier.clickable(role = Role.Button, onClick = onLoadMore).padding(14.dp), color = colors.text, style = LibraryStyle.body)
                        }
                    }
                }
                // Room to scroll the last cards up from under the bottom bar; reels end on their last page.
                item(key = MOSAIC_END, contentType = "end") {
                    Layout(content = {}, modifier = Modifier.fillMaxWidth()) { _, constraints ->
                        layout(constraints.maxWidth, (bottomPadding * (1f - mosaic.reel)).roundToInt()) {}
                    }
                }
            }
            if (feed.cards.isNotEmpty() && layouts.level(1).cells.isNotEmpty()) {
                // Header/footer remain real LazyColumn children; temporary projected edges clip only inside the body, whose
                // sides stop at the margins, so a card sliding off one end of its line leaves at the grid's edge. Reels
                // reach the screen's edges, so the sides open as they come.
                Box(Modifier.fillMaxSize().graphicsLayer {
                    val visible = state.layoutInfo.visibleItemsInfo
                    val top = visible.firstOrNull { it.key == MOSAIC_HEADER }?.let { (it.offset + it.size).toFloat() } ?: 0f
                    val bottom = visible.firstOrNull { it.key == "gallery-footer" }?.offset?.toFloat() ?: size.height
                    shape = MosaicBodyShape(top, bottom)
                    clip = true
                }.drawBehind {
                    // A theme's backdrop shows through between the cards; reels still darken to their own paper.
                    drawRect(if (backdropShown) REEL_PAPER.copy(alpha = mosaic.reel) else lerpColor(colors.background, REEL_PAPER, mosaic.reel))
                }.drawWithContent {
                    val side = margin * (1f - mosaic.reel)
                    clipRect(left = side, right = size.width - side) { this@drawWithContent.drawContent() }
                }) {
                    // Its own composable, so the cards coming into view recompose only the cards, not the whole gallery.
                    MosaicBody(layouts, mosaic, height, margin, { frame() }, { anchor?.cardId?.let(layouts.byId::get) },
                        { state.isScrollInProgress }, onOpenCard, onLongPressCard, choice,
                        lifted = { liftedId?.let { MosaicLift(it, finger - grab) } }, onMoveCard = onMoveCard)
                }
            }
        }
    }
}

/**
 * The cards of a [LibraryMosaic] at their places in the current [frame]: the tiles around the viewport and those composed
 * ahead ([MosaicPrefetch]), the blurred fill while a step moves, and the pages in reels. [focal] is the index of the card
 * under the fingers, if any, and [scrolling] is true while the list scrolls.
 */
@Composable
private fun MosaicBody(
    layouts: MosaicLayouts, mosaic: MosaicState, height: Float, margin: Float, frame: () -> MosaicFrame, focal: () -> Int?,
    scrolling: () -> Boolean, onOpenCard: (Card) -> Unit, onLongPressCard: ((Card) -> Unit)?, choice: CardChoice?,
    lifted: () -> MosaicLift? = { null }, onMoveCard: ((Card, String?) -> Unit)? = null,
) {
    val latestFrame by rememberUpdatedState(frame)
    val currentFrame = remember { { latestFrame() } }
    val window by remember(layouts, height) { derivedStateOf { latestFrame().window(height) } }
    // Which cards change line or shape decides what is composed; it changes once per step, not per frame.
    val shape by remember(layouts, height) { derivedStateOf { latestFrame().shape(window, layouts) } }
    // The card under the fingers is drawn over the others, from the same keyed child, so it is never rebuilt.
    val focalIndex = focal()
    // Pages are composed around the viewport while reels show, and tiles only while the gallery is not
    // resting on its pages; each changes as a page comes into view or a step begins, not per frame.
    val pages by remember(layouts, height) { derivedStateOf { latestFrame().pages(height) } }
    val tilesShown by remember(mosaic) { derivedStateOf { mosaic.reel < 1f } }
    // The cards a screen above and below, which are composed ahead, a frame at a time, before they show.
    val ahead by remember(layouts, height) { derivedStateOf { latestFrame().ahead(height, height) } }
    val prefetch = remember(layouts) { MosaicPrefetch() }
    LaunchedEffect(prefetch, height) {
        // At rest in reels the tiles of the grids around and a stop past are readied, unplaced, so pinching out of reels
        // composes none.
        snapshotFlow { MosaicPrefetchWindow(window, ahead,
            moving = mosaic.isPinching || mosaic.position != mosaic.columns.toFloat(), scrolling(), reels = !tilesShown,
            // Pages stay readied through reels, so pinching out of them builds none again.
            pages = pages) }
            .collectLatest { prefetch.fill(it, layouts, focalIndex = focal) }
    }
    if (shape?.fill == true) MosaicFill(layouts, height, margin, currentFrame, blur = with(LocalDensity.current) { BACKDROP_BLUR.toPx() })
    val composed = union(window, prefetch.tiles.filter { it in ahead })
    // Read once here, not by every card as it is composed.
    val motion = rememberSheetMotionEnabled()
    CompositionLocalProvider(LocalSheetMotionEnabled provides motion) {
        ProjectedMosaicCards(layouts, mosaic, composed, focalIndex, shape, prefetch.unders.filterTo(HashSet()) { it in window },
            // At rest in reels every page shows at once; on the way there each waits until it is readied.
            if (tilesShown) prefetch.pages.filterTo(HashSet()) { it in pages } else pages.toSet(),
            tilesShown, if (tilesShown) emptySet() else prefetch.tiles, currentFrame, margin, Modifier.fillMaxSize().testTag("mosaic-projection"), onOpenCard, onLongPressCard, choice, lifted, onMoveCard)
    }
}

/**
 * Slides fixed controls up past the top of the screen, out of sight and out of reach, as far as [mosaic] has gone into
 * reels, so a page fills the screen.
 */
internal fun Modifier.awayInReels(mosaic: MosaicState): Modifier = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    layout(placeable.width, placeable.height) {
        val top = coordinates?.positionInRoot()?.y ?: 0f
        placeable.placeWithLayer(0, 0) {
            val away = mosaic.reel
            alpha = 1f - away
            translationY = -away * (top + size.height)
        }
    }
}

/** One drawing of a card: where it is and how opaque. */
private class MosaicDraw(val rect: Rect, val alpha: Float)

/**
 * How a card is drawn now. The [card] drawing carries its touches and semantics; [copy] is a second, inert drawing
 * while the card changes line. [progress] is how far the card has gone from the old grid to the new one.
 */
private class MosaicDrawings(val card: MosaicDraw, val copy: MosaicDraw?, val progress: Float)

/**
 * What a step composes beyond one tile per card: copies of the cards changing line, old layouts of words changing shape,
 * and whether the blurred [fill] shows behind them.
 */
private data class MosaicShape(val wrapping: Set<Int>, val reshaping: Set<Int>, val fill: Boolean)

/**
 * One child of [ProjectedMosaicCards]: card [index]'s tile or its [copy], the layout drawn [under] it while it changes
 * shape, or its [page] in reels.
 */
private data class MosaicSlot(val index: Int, val copy: Boolean, val under: Boolean, val page: Boolean = false)

/**
 * Where each card is drawn now, in viewport pixels, while the fingers move it between its places in the grids of two
 * stops (Octavian's pick, version O3b of the "Calmer steps for O" canvas, 28 October 2026). A card that stays on its line
 * slides to its new place; one that changes line slides out past one edge of its old line and in from the other edge
 * of its new one, shrinking a little and fading between the two drawings. Lines further down start later, like a
 * wave. Behind the cards both grids are drawn whole and blurred ([fill]), so no blank paper shows between them.
 * [held] is true while the fingers are down.
 */
private class MosaicFrame(
    private val from: MosaicProjection, private val to: MosaicProjection, private val x: Float, val y: Float,
    private val progress: Float, private val wave: MosaicWave, private val wrap: Float,
    private val fillFrom: MosaicProjection, private val fillFromX: Float,
    private val fillTo: MosaicProjection, private val fillToX: Float,
    private val held: Boolean = false,
    private val beyond: List<MosaicProjection> = emptyList(),
) {
    val fromColumns: Int get() = from.columns
    val toColumns: Int get() = to.columns

    /** True between two stops, while cards are on their way and the blurred fill shows behind them. */
    val moving: Boolean get() = from.columns != to.columns && progress > 0f && progress < 1f

    /** True when one of the two grids is the reels, where every card is on its own line, as at one column. */
    private val reeling: Boolean get() = from.layouts.isReels(from.columns) || to.layouts.isReels(to.columns)

    private fun place(projection: MosaicProjection, index: Int): Rect = projection.cell(index).translate(Offset(x, -y))

    /**
     * Whether card [index] changes line. Its top decides, not its centre: a card two squares high in one grid and one in
     * the other keeps its line, as the focal card always does. Both grids scale together, so the answer holds for the
     * whole step.
     */
    fun wraps(index: Int): Boolean {
        if (!moving || reeling) return false
        val a = place(from, index)
        val b = place(to, index)
        return abs(a.top - b.top) > 0.5f * minOf(a.height, b.height)
    }

    fun draws(index: Int): MosaicDrawings {
        val a = place(from, index)
        val b = place(to, index)
        val m = wave.progress(index, progress)
        // Into reels a card grows from its cell to its page and is drawn at its own shape inside it.
        if (reeling) return MosaicDrawings(MosaicDraw(from.layouts.reelContent(index, lerp(a, b, m), if (to.layouts.isReels(to.columns)) m else 1f - m), 1f), null, m)
        if (!wraps(index)) return MosaicDrawings(MosaicDraw(lerp(a, b, m), 1f), null, m)
        val size = lerp(a.size, b.size, m)
        val past = if (b.top > a.top) wrap else -wrap
        val left = a.left + (b.left + past - a.left) * m
        val shrink = 1f - 0.1f * sin(PI.toFloat() * m)
        val shown = m * m * (3f - 2f * m)
        val leaving = MosaicDraw(Rect(Offset(left, a.top), size).scaled(shrink), 1f - shown)
        val entering = MosaicDraw(Rect(Offset(left - past, b.top), size).scaled(shrink), shown)
        return if (m < 0.5f) MosaicDrawings(leaving, entering, m) else MosaicDrawings(entering, leaving, m)
    }

    /** Where card [index]'s touchable drawing is. */
    fun cell(index: Int): Rect = draws(index).card.rect

    /** The cards either grid shows around the viewport, so a card sliding in or out is drawn the whole way. */
    fun window(viewport: Float): IntRange = union(from.window(y, viewport), to.window(y, viewport))

    /** The cards either grid shows within [margin] above or below the viewport. */
    fun window(viewport: Float, margin: Float): IntRange =
        union(from.window(y - margin, viewport + 2 * margin), to.window(y - margin, viewport + 2 * margin))

    /**
     * The cards to have ready: those either grid shows within [margin] above or below the viewport, and those the grids a
     * stop further show in it, which a pinch past the next stop brings in all at once.
     */
    fun ahead(viewport: Float, margin: Float): IntRange =
        beyond.fold(window(viewport, margin)) { cards, grid -> union(cards, grid.window(y, viewport)) }

    /** The cards whose pages show around the viewport while one of the grids is the reels; none otherwise. */
    fun pages(viewport: Float): IntRange = when {
        to.layouts.isReels(to.columns) -> to.window(y, viewport)
        from.layouts.isReels(from.columns) -> from.window(y, viewport)
        else -> IntRange.EMPTY
    }

    /**
     * Card [index]'s page: where it is on its way between its cell and the whole screen, and how strongly it shows, fading
     * in over the last part of the way into reels ([reelPageShown]).
     */
    fun page(index: Int): MosaicDraw {
        val m = wave.progress(index, progress)
        return MosaicDraw(lerp(place(from, index), place(to, index), m), reelPageShown(if (to.layouts.isReels(to.columns)) m else 1f - m))
    }

    /**
     * What this step composes beyond the tiles. While the fingers are down, a tile with words keeps its layout in the
     * other grid even as the cards rest on a stop, so crossing a stop composes nothing new; it goes once they lift and
     * the cards land.
     */
    fun shape(window: IntRange, layouts: MosaicLayouts): MosaicShape? {
        if (!moving && !held) return null
        // Resting on the last stop, the other grid is the one the fingers can only head for.
        val other = if (to.columns != from.columns) to.columns else (from.columns - 1).coerceAtLeast(1)
        return MosaicShape(
            if (moving) window.filter(::wraps).toSet() else emptySet(),
            window.filter { !layouts.isPicture(it) && layouts.wordsFrame(it, from.columns) != layouts.wordsFrame(it, other) }.toSet(),
            fill = moving && !reeling,
        )
    }

    /** The cards each blurred grid shows, by its columns: the new grid first, drawn whole, then the old one fading out over it. */
    fun fill(viewport: Float): List<Pair<Int, IntRange>> =
        listOf(fillTo.columns to fillTo.window(y, viewport), fillFrom.columns to fillFrom.window(y, viewport))

    fun fillCell(columns: Int, index: Int): Rect =
        if (columns == fillTo.columns) fillTo.cell(index).translate(Offset(fillToX, -y)) else fillFrom.cell(index).translate(Offset(fillFromX, -y))

    fun fillAlpha(columns: Int): Float = if (columns == fillTo.columns) 1f else 1f - progress

    /** How strongly the whole blurred fill shows now ([mosaicFillShown]). */
    val fillShown: Float get() = mosaicFillShown(progress)
}

/** How strongly a card's page shows [reel] of the way into reels: over the last two fifths, as its tile comes to rest. */
private fun reelPageShown(reel: Float): Float {
    val t = ((reel - 0.6f) / 0.4f).coerceIn(0f, 1f)
    return t * t * (3f - 2f * t)
}

/** The cards of [window] and [extra], in reading order. */
private fun union(window: IntRange, extra: List<Int>): List<Int> =
    if (extra.isEmpty()) window.toList() else (window.toSortedSet() + extra).sorted()

/** Both index ranges and everything between them. */
private fun union(a: IntRange, b: IntRange): IntRange = when {
    a.isEmpty() -> b
    b.isEmpty() -> a
    else -> minOf(a.first, b.first)..maxOf(a.last, b.last)
}

/** This rectangle scaled by [factor] about its centre. */
private fun Rect.scaled(factor: Float): Rect =
    Rect(center.x - width * factor / 2f, center.y - height * factor / 2f, center.x + width * factor / 2f, center.y + height * factor / 2f)

/**
 * Lines further down the screen start later: a card waits up to [WAVE_DELAY] of the step, by its reading order among
 * the cards either grid shows, and then makes up the rest by the end of the step. The [focal] card never waits, so it
 * changes shape with the fingers.
 */
private class MosaicWave(private val first: Int, private val last: Int, private val focal: Int) {
    fun progress(index: Int, step: Float): Float {
        if (index == focal) return step
        val read = if (last <= first) 0f else ((index - first).toFloat() / (last - first)).coerceIn(0f, 1f)
        val wait = WAVE_DELAY * read
        return ((step - wait) / (1f - wait)).coerceIn(0f, 1f)
    }
}

/** The [MosaicWave] of the current step, worked out once per step rather than on every pointer frame. */
private class MosaicWaves {
    private var key: Any? = null
    private var wave = MosaicWave(0, 0, -1)

    fun of(layouts: MosaicLayouts, between: MosaicBetween, focal: Int, anchor: MosaicAnchor?, viewport: Float): MosaicWave {
        val next = listOf(layouts, between.from, between.to, focal, anchor, viewport)
        if (next == key) return wave
        var first = Int.MAX_VALUE
        var last = -1
        // Into reels every card moves at once, so none slides under a card growing to fill the screen.
        if (!layouts.isReels(between.from) && !layouts.isReels(between.to)) for (columns in setOf(between.from, between.to)) {
            val level = layouts.level(columns)
            val cell = level.cells[focal]
            val top = cell.top + (anchor?.fraction?.y ?: 0.5f) * cell.height - (anchor?.viewport?.y ?: (viewport / 2f))
            for (index in layouts.window(columns.toFloat(), top, viewport)) {
                val shown = level.cells[index]
                if (shown.bottom > top && shown.top < top + viewport) {
                    first = minOf(first, index)
                    last = maxOf(last, index)
                }
            }
        }
        key = next
        wave = if (last < 0) MosaicWave(0, 0, focal) else MosaicWave(first, last, focal)
        return wave
    }
}

/**
 * The children [ProjectedMosaicCards] composes for card [index], in drawing order: its [tile]'s, then its [page]. Its
 * layout in the other grid is composed once it is among [unders], ready before a pinch needs it.
 */
private fun mosaicSlots(index: Int, shape: MosaicShape?, unders: Set<Int>, tile: Boolean, page: Boolean): List<MosaicSlot> {
    val copy = tile && shape?.wrapping?.contains(index) == true
    val under = tile && index in unders
    return listOfNotNull(
        MosaicSlot(index, copy = false, under = true).takeIf { under },
        MosaicSlot(index, copy = false, under = false).takeIf { tile },
        MosaicSlot(index, copy = true, under = true).takeIf { copy && under },
        MosaicSlot(index, copy = true, under = false).takeIf { copy },
        MosaicSlot(index, copy = false, under = false, page = true).takeIf { page },
    )
}

/**
 * Draws [indices] at their places in the current [MosaicFrame], [left] in from the body's edge. Each card has one tile
 * that takes touches and semantics; a card changing line also has an inert copy, and a tile with words changing shape
 * has its layout in the other grid drawn under it, the two cross-fading with the card's progress. The [focal] card is
 * drawn last, over the others. While reels show, the cards in [pages] also have their page, laid out at the whole
 * screen's size and drawn over the tile as it comes to rest; with [tiles] false, at rest in reels, the page is all there
 * is, and takes the touches and semantics.
 */
@Composable
private fun ProjectedMosaicCards(
    layouts: MosaicLayouts, mosaic: MosaicState, indices: List<Int>, focal: Int?, shape: MosaicShape?, unders: Set<Int>, pages: Set<Int>, tiles: Boolean,
    readied: Set<Int>, frame: () -> MosaicFrame, left: Float, modifier: Modifier, onOpenCard: (Card) -> Unit, onLongPressCard: ((Card) -> Unit)?,
    choice: CardChoice?,
    lifted: () -> MosaicLift? = { null }, onMoveCard: ((Card, String?) -> Unit)? = null,
) {
    val latestMove by rememberUpdatedState(onMoveCard)
    // With a choice the sweep takes holds, and a tap while finds are chosen adds or drops one. The gallery's own hold
    // takes a card that can move, and opens its menu on release.
    val tap = { card: Card -> if (choice?.active == true) choice.toggle(card) else onOpenCard(card) }
    val hold = onLongPressCard.takeIf { choice == null && onMoveCard == null }
    val slots = indices.flatMap { mosaicSlots(it, shape, unders, tiles || it in readied, it in pages) }
    val density = LocalDensity.current
    val corner = with(density) { layouts.tiles.corner.toPx() }
    val shadow = LibraryStyle.colors.shadow
    Layout(modifier = modifier, content = {
        indices.forEach { index ->
            key(layouts.cards[index].id) {
                val card = layouts.cards[index]
                // A picture keeps the largest size it has been decoded at, so landing on more columns reloads nothing.
                val largest = remember { IntArray(1) }
                val imageSize = maxOf(largest[0], imageRequestSize(layouts, mosaic, index)).also { largest[0] = it }
                // A copy draws again what its card or its other layout drew, so a pinch composes nothing for it.
                val drawing = rememberGraphicsLayer()
                // Remembered, so the window moving on during a pinch recomposes the list without its tiles.
                val tile = remember(drawing) { Modifier.fillMaxSize().recording(drawing) }
                val underDrawing = if (index in unders) rememberGraphicsLayer() else null
                // Keyed, so the tile keeps its place when a layout is added under it or taken away.
                mosaicSlots(index, shape, unders, tiles || index in readied, index in pages).forEach { slot ->
                    key(slot.copy, slot.under, slot.page) {
                        when {
                            slot.page -> layouts.tiles.ReelPage(card,
                                onClick = { if (!mosaic.isPinching) onOpenCard(card) },
                                onLongClick = onLongPressCard?.let { press -> { if (!mosaic.isPinching) press(card) } },
                                imageRequestSizePx = reelImageSize(layouts, index), modifier = Modifier.fillMaxSize())
                            slot.copy -> MosaicCopy(if (slot.under) underDrawing ?: drawing else drawing)
                            slot.under -> Box(remember(underDrawing) { Modifier.fillMaxSize().clearAndSetSemantics {}.recording(underDrawing ?: drawing) }) {
                                layouts.tiles.Copy(card, imageSize)
                            }
                            // Tiles stay enabled through a pinch: the gesture takes every touch it moves, and toggling them
                            // would rebuild every tile on screen as the fingers land and again as they let go.
                            // A tile readied in reels is hidden from touch and from accessibility until it shows; the box
                            // around it changes, not the tile, so showing it rebuilds nothing.
                            else -> Box(if (!tiles) Modifier.clearAndSetSemantics {} else if (onMoveCard != null) Modifier.moveActions(layouts.cards, index) { moved, after ->
                                latestMove?.invoke(moved, after)
                            } else Modifier, propagateMinConstraints = true) {
                                layouts.tiles.Tile(card,
                                    onClick = { if (!mosaic.isPinching) tap(card) },
                                    onLongClick = hold?.let { press -> { if (!mosaic.isPinching) press(card) } },
                                    enabled = true, imageRequestSizePx = imageSize, modifier = tile)
                                if (choice != null) ChoiceMark(choice, card.id, layouts.tiles.corner)
                            }
                        }
                    }
                }
            }
        }
    }) { children, constraints ->
        val current = frame()
        val held = lifted()
        val heldIndex = held?.let { layouts.byId[it.cardId] }
        val heldLift = with(density) { MOVE_LIFT.toPx() }
        // Reels reach the screen's edges, so their pages have square corners and no shadow.
        val round = corner * (1f - mosaic.reel)
        val lifted = 1f - mosaic.reel
        val drawings = HashMap<Int, MosaicDrawings>()
        val reach = constraints.maxHeight * 0.05f
        val placed = children.zip(slots) { child, slot ->
            if (slot.page) {
                val page = current.page(slot.index)
                val whole = layouts.level(1).cells[slot.index]
                return@zip PlacedTile(child.measure(whole.size.fixed()), page.rect.translate(left, 0f), true, page.alpha, slot.index, page = true)
            }
            val drawn = drawings.getOrPut(slot.index) { current.draws(slot.index) }
            val draw = if (slot.copy) drawn.copy ?: drawn.card else drawn.card
            val m = drawn.progress
            val words = !layouts.isPicture(slot.index)
            // A tile with words is laid out in the grid it is nearer; its layout in the other one is drawn under it.
            val near = if (m < 0.5f) current.fromColumns else current.toColumns
            val far = if (m < 0.5f) current.toColumns else current.fromColumns
            val reshapes = shape?.reshaping?.contains(slot.index) == true && slot.index in unders
            // A layout readied for a pinch to come is laid out at the stop the pinch heads for, and not drawn until then.
            if (slot.under && !reshapes) {
                val other = if (current.toColumns != current.fromColumns) current.toColumns else if (current.fromColumns > 1) current.fromColumns - 1 else 2
                child.measure(layouts.wordsFrame(slot.index, other).fixed())
                return@zip null
            }
            // At rest in reels a readied tile is laid out, not drawn, until a pinch out of reels shows it.
            if (!tiles) {
                child.measureTile(draw.rect, layouts.wordsFrame(slot.index, near).takeIf { words })
                return@zip null
            }
            // The tile gives way to its page as the page fades in over it.
            val paged = if (slot.index in pages) 1f - current.page(slot.index).alpha else 1f
            val alpha = draw.alpha * paged * if (reshapes && !slot.under) maxOf(m, 1f - m) else 1f
            // A picture off screen keeps its size at rest, so a pinch lays out again only the pictures it shows, not
            // those composed ahead; it is placed all the same, so its place stays true for touch and accessibility.
            val off = draw.rect.bottom < -reach || draw.rect.top > constraints.maxHeight + reach
            val cell = if (off && !words) layouts.level(current.fromColumns).cells[slot.index] else draw.rect
            val tile = child.measureTile(cell, layouts.wordsFrame(slot.index, if (slot.under) far else near).takeIf { words })
            // A tile with words changing shape casts one shadow, from its layout underneath, which is always opaque.
            val lift = if (reshapes && !slot.under) 0f else with(density) { layouts.tiles.elevation(layouts.cards[slot.index]).toPx() } * lifted
            // The dragged card follows the finger at its own size, raised over the others.
            if (slot.index == heldIndex && !slot.copy && !slot.under && held != null) {
                return@zip PlacedTile(tile, Rect(held.topLeft, draw.rect.size), words, 1f, slot.index, lift = heldLift)
            }
            PlacedTile(tile, draw.rect.translate(left, 0f), words, alpha, slot.index, lift = lift)
        }.filterNotNull()
        layout(constraints.maxWidth, constraints.maxHeight) {
            for (pagesNow in listOf(false, true)) {
                val top = heldIndex ?: focal
                placed.forEach { if (it.page == pagesNow && it.index != top) placeTile(it.tile, it.cell, it.words, it.alpha, if (it.page) 0f else round, it.lift, shadow) }
                placed.forEach { if (it.page == pagesNow && it.index == top) placeTile(it.tile, it.cell, it.words, it.alpha, if (it.page) 0f else round, it.lift, shadow) }
            }
        }
    }
}

/**
 * A tile measured for its cell, as [placeTile] places it; [index] is its card's, [page] is true for its page in reels,
 * and [lift] is the elevation, in pixels, its shadow is cast from.
 */
private class PlacedTile(val tile: Placeable, val cell: Rect, val words: Boolean, val alpha: Float, val index: Int = -1, val page: Boolean = false,
    val lift: Float = 0f)

/** The card being dragged ([cardId]) and where its tile's top left corner is under the finger, in the gallery's pixels. */
private class MosaicLift(val cardId: String, val topLeft: Offset)

/** Height, above the others, of the card being dragged. */
private val MOVE_LIFT = 12.dp

/** How near the top or bottom edge a dragged card scrolls the gallery. */
private val MOVE_EDGE = 96.dp

/** Pixels the gallery scrolls each frame while a dragged card is held at an edge. */
private const val MOVE_SCROLL_STEP = 14f

/**
 * Move earlier and Move later for accessibility services, on the tile of card [index] of [cards]: each calls [move] with
 * the card and the card it would then follow, as dropping it there would.
 */
private fun Modifier.moveActions(cards: List<Card>, index: Int, move: (Card, String?) -> Unit): Modifier = semantics {
    customActions = listOfNotNull(
        if (index > 0) CustomAccessibilityAction("Move earlier") { move(cards[index], cards.getOrNull(index - 2)?.id); true } else null,
        if (index < cards.lastIndex) CustomAccessibilityAction("Move later") { move(cards[index], cards[index + 1].id); true } else null,
    )
}

/** An inert second drawing of what [drawing] recorded: no semantics or touches. */
@Composable
private fun MosaicCopy(drawing: GraphicsLayer) {
    Spacer(remember(drawing) { Modifier.fillMaxSize().drawBehind { drawLayer(drawing) } })
}

/** Draws the content through [layer], so a [MosaicCopy] can draw the same picture somewhere else. */
private fun Modifier.recording(layer: GraphicsLayer): Modifier = drawWithContent {
    layer.record { this@drawWithContent.drawContent() }
    drawLayer(layer)
}

/**
 * Both grids drawn whole and blurred behind the moving cards ([MosaicFrame.fill]), so no blank paper shows, [left] in
 * from the body's edge. It is one drawing: each card is its backdrop's colour and, once the picture is in memory, the
 * picture itself, so a pinch composes nothing for it as it starts or crosses a stop.
 */
@Composable
private fun MosaicFill(layouts: MosaicLayouts, height: Float, left: Float, frame: () -> MosaicFrame, blur: Float) {
    val colors = LibraryStyle.colors
    val loader = SingletonImageLoader.get(LocalPlatformContext.current)
    val backdrops = remember(layouts, colors) { arrayOfNulls<MosaicBackdrop>(layouts.cards.size) }
    val pictures = remember(layouts) { HashMap<String, ImageBitmap>() }
    fun picture(key: String): ImageBitmap? = pictures[key]
        ?: loader.memoryCache?.get(MemoryCache.Key(key))?.image?.toBitmap()?.asImageBitmap()?.also { pictures[key] = it }
    Spacer(Modifier.fillMaxSize().clearAndSetSemantics {}.graphicsLayer {
        renderEffect = BlurEffect(blur, blur, TileMode.Clamp)
        alpha = frame().fillShown
    }.drawBehind {
        val current = frame()
        for ((columns, cards) in current.fill(height)) {
            val alpha = current.fillAlpha(columns)
            for (index in cards) {
                val cell = current.fillCell(columns, index).translate(left, 0f)
                if (cell.bottom <= 0f || cell.top >= size.height || cell.width < 1f || cell.height < 1f) continue
                val backdrop = backdrops[index] ?: layouts.tiles.backdrop(layouts.cards[index], colors)?.also { backdrops[index] = it } ?: continue
                drawRect(backdrop.color, cell.topLeft, cell.size, alpha)
                val image = backdrop.picture?.let(::picture) ?: continue
                // The picture is cut to its cell as the tile cuts it.
                val scale = maxOf(cell.width / image.width, cell.height / image.height)
                val cut = IntSize((cell.width / scale).roundToInt().coerceIn(1, image.width), (cell.height / scale).roundToInt().coerceIn(1, image.height))
                drawImage(image, IntOffset((image.width - cut.width) / 2, (image.height - cut.height) / 2), cut,
                    IntOffset(cell.left.roundToInt(), cell.top.roundToInt()), IntSize(cell.width.roundToInt(), cell.height.roundToInt()), alpha)
            }
        }
    })
}

/** Flings freely, except in reels, where a fling moves at most one page and lands on a page's top ([reelDistance]). */
private class MosaicFling(private val free: FlingBehavior, private val reelDistance: (Float) -> Float?) : FlingBehavior {
    override suspend fun ScrollScope.performFling(initialVelocity: Float): Float {
        val distance = reelDistance(initialVelocity) ?: return with(free) { performFling(initialVelocity) }
        var done = 0f
        animate(0f, distance, initialVelocity, spring(stiffness = REEL_STIFFNESS)) { value, _ -> done += scrollBy(value - done) }
        return 0f
    }
}

/** Shows [shown] of the content's height, its lower part, fading the rest out: a page's header as reels open. */
private fun Modifier.collapsing(shown: () -> Float): Modifier = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    val fraction = shown().coerceIn(0f, 1f)
    val height = (placeable.height * fraction).roundToInt()
    layout(placeable.width, height) {
        if (fraction >= 1f) placeable.place(0, 0) else placeable.placeWithLayer(0, height - placeable.height) { alpha = fraction }
    }
}

/** The size a card's page in reels decodes its picture at: the picture's own shape fitted to the whole screen. */
private fun reelImageSize(layouts: MosaicLayouts, index: Int): Int =
    layouts.reelContent(index, layouts.level(1).cells[index], 1f).let { maxOf(it.width, it.height) }.roundToInt().coerceAtLeast(1)

/**
 * The size gallery pictures are decoded at: the card's cell at the density the gallery last came to rest on
 * ([MosaicState.pictureColumns]), so neither a pinch nor its landing changes it; in reels, the size its page asks.
 */
private fun imageRequestSize(layouts: MosaicLayouts, mosaic: MosaicState, index: Int): Int =
    if (layouts.isReels(mosaic.pictureColumns)) reelImageSize(layouts, index)
    else layouts.level(mosaic.pictureColumns).cells[index].let { maxOf(it.width, it.height) }.roundToInt().coerceAtLeast(1)

/**
 * Measures a tile for [cell]. A picture is cut to the cell itself; a tile with words is laid out at its [frame]
 * ([MosaicLayouts.wordsFrame]), so its words keep their lines however far the fingers move.
 */
private fun Measurable.measureTile(cell: Rect, frame: Size?): Placeable = measure((frame ?: cell.size).fixed())

/** Exact constraints for a size in whole pixels. */
private fun Size.fixed(): Constraints = Constraints.fixed(width.roundToInt().coerceAtLeast(1), height.roundToInt().coerceAtLeast(1))

/**
 * Places a tile measured by [measureTile] in [cell] at [alpha], with its corners rounded to [corner] pixels on screen
 * and, when [lift] is above 0, a soft [shadow] cast from that height; a tile with words is scaled from its frame, as a
 * picture zooms. The shadow fades with the tile.
 */
private fun Placeable.PlacementScope.placeTile(tile: Placeable, cell: Rect, words: Boolean, alpha: Float, corner: Float = 0f,
    lift: Float = 0f, shadow: Color = Color.Black) {
    val x = cell.left.roundToInt()
    val y = cell.top.roundToInt()
    if (!words && alpha >= 1f && corner <= 0f && lift <= 0f) {
        tile.place(x, y)
        return
    }
    tile.placeWithLayer(x, y) {
        this.alpha = alpha
        if (words) {
            transformOrigin = TransformOrigin(0f, 0f)
            scaleX = cell.width / tile.width
            scaleY = cell.height / tile.height
        }
        if (corner > 0f || lift > 0f) {
            // The layer scales its own outline, so a tile drawn scaled gets corners that scale cancels.
            shape = RoundedCornerShape(corner / sqrt(scaleX * scaleY))
            clip = true
        }
        if (lift > 0f) {
            shadowElevation = lift
            ambientShadowColor = shadow
            spotShadowColor = shadow
        }
    }
}

/**
 * What [MosaicPrefetch] readies: the cards [shown], those [ahead] of them, and whether the cards move under the fingers or
 * land ([moving]) or the list scrolls.
 */
private data class MosaicPrefetchWindow(val shown: IntRange, val ahead: IntRange, val moving: Boolean, val scrolling: Boolean,
    val reels: Boolean = false, val pages: IntRange = IntRange.EMPTY)

/**
 * Cards composed before they are needed, one a frame, so neither scrolling nor a pinch composes a burst of them in one
 * frame: the [tiles] a screen above and below what shows and those the grids a stop further show, nearest first, and,
 * for the cards with words that show, their layouts in the other grid ([unders]), which a pinch cross-fades with their
 * own, once the list is not scrolling. Each frame readies one of each. While the cards move under the fingers or land,
 * nothing but [pages] is readied, so a pinch's frames only move cards; on the way into reels the [pages] around the
 * fingers come first, one a frame, long before they fade in.
 */
@Stable
private class MosaicPrefetch {
    var tiles by mutableStateOf<Set<Int>>(emptySet())
        private set
    var unders by mutableStateOf<Set<Int>>(emptySet())
        private set
    var pages by mutableStateOf<Set<Int>>(emptySet())
        private set

    /** Readies [window] a frame at a time, nearest [focalIndex] or the middle of what shows first, until all of it is. */
    suspend fun fill(window: MosaicPrefetchWindow, layouts: MosaicLayouts, focalIndex: () -> Int?) {
        val shown = window.shown
        tiles = tiles.filterTo(HashSet()) { it in window.ahead && (window.reels || it !in shown) }
        unders = unders.filterTo(HashSet()) { it in shown }
        pages = pages.filterTo(HashSet()) { it in window.pages }
        if (shown.isEmpty()) return
        while (true) {
            val middle = focalIndex() ?: ((shown.first + shown.last) / 2)
            val page = window.pages.filter { it !in pages }.minByOrNull { abs(it - middle) }
            if (page != null) {
                withFrameNanos { }
                pages = pages + page
                continue
            }
            // While the cards move under the fingers or land, nothing is built but pages: what a pinch needs is
            // readied at rest, so its frames only move cards.
            if (window.moving) return
            // Layouts for a pinch wait while the list scrolls, so scrolling composes only the cards ahead.
            val under = if (window.reels || window.scrolling) null
                else shown.filter { !layouts.isPicture(it) && it !in unders }.minByOrNull { abs(it - middle) }
            val tile = window.ahead.filter { (window.reels || it !in shown) && it !in tiles }.minByOrNull {
                if (window.reels) abs(it - middle) else if (it < shown.first) shown.first - it else it - shown.last
            }
            if (under == null && tile == null) return
            withFrameNanos { }
            if (tile != null) tiles = tiles + tile
            if (under != null) unders = unders + under
        }
    }
}

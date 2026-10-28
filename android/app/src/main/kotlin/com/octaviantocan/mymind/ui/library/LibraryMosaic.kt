package com.octaviantocan.mymind.ui.library

import androidx.compose.animation.core.Easing
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.foundation.MutatePriority
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.overscroll
import androidx.compose.foundation.rememberOverscrollEffect
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.scrollable
import androidx.compose.foundation.gestures.stopScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.lerp
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.Measurable
import androidx.compose.ui.layout.Placeable
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
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
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.card.sheetEdgeFade
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.roundToInt
import kotlin.math.sin

private val MOSAIC_GAP = 3.dp
private const val MOSAIC_HEADER = "gallery-header"
private const val MOSAIC_ROW = "row:"
private const val HOP_NANOS = 130 * 1_000_000L
private const val FRAME_NANOS = 16_666_667L
private const val SETTLE_MILLIS = 180
private val SettleEasing = Easing { 1f - (1f - it) * (1f - it) * (1f - it) }
private val BACKDROP_BLUR = 14.dp
private const val WAVE_DELAY = 0.5f
private data class MosaicAnchor(val cardId: String, val viewport: Offset, val fraction: Offset)
private data class MosaicBodyShape(val top: Float, val bottom: Float) : Shape {
    override fun createOutline(size: Size, layoutDirection: LayoutDirection, density: Density): Outline =
        Outline.Rectangle(Rect(0f, top.coerceIn(0f, size.height), size.width, bottom.coerceIn(top.coerceIn(0f, size.height), size.height)))
}
private data class MosaicLoadPosition(val count: Int, val last: Int, val total: Int, val ready: Boolean)

/**
 * Virtualized bricks that follow the fingers between cached density layouts. Stable card keys retain content while
 * only the bounded visible window is measured at its real, interpolated size.
 */
@Composable
fun LibraryMosaic(
    feed: FeedState,
    onLoadMore: () -> Unit,
    onOpenCard: (Card) -> Unit,
    modifier: Modifier = Modifier,
    state: LazyListState = rememberLazyListState(),
    mosaic: MosaicState = rememberMosaicState(),
    onLongPressCard: ((Card) -> Unit)? = null,
    header: (@Composable () -> Unit)? = null,
) {
    val colors = LibraryStyle.colors
    val density = LocalDensity.current
    val moving = rememberSheetMotionEnabled()
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
    val latestFeed by rememberUpdatedState(feed)
    val latestMore by rememberUpdatedState(onLoadMore)
    val headerItems = if (header == null) 0 else 1
    BoxWithConstraints(modifier.fillMaxSize()) {
        val width = with(density) { maxWidth.toPx() }
        val height = with(density) { maxHeight.toPx() }
        val gap = with(density) { MOSAIC_GAP.toPx() }
        val bottomPadding = with(density) { 156.dp.toPx() }
        val layouts = remember(feed.cards, width, gap) { MosaicLayouts(feed.cards, width, gap) }
        // The slots measure against the rows composed here, never a newer reference whose rows are not composed yet.
        val referenceColumns = mosaic.referenceColumns
        val rows = layouts.level(referenceColumns).rows

        fun clampOrigin(value: Float): Float {
            val first = if (header == null) 0f else -headerHeight.toFloat()
            val footer = if (latestFeed.loading || latestFeed.error != null) footerHeight else 0
            val last = (layouts.height(mosaic.position) + footer + bottomPadding - height).coerceAtLeast(first)
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
            return clampOrigin(cell.top + focal.fraction.y * cell.height - focal.viewport.y)
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
            return (focal.viewport.x - cell.left - focal.fraction.x * cell.width) * anchorXWeight
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
            return MosaicFrame(from, to, x, y, t, waves.of(layouts, between, focal, anchor, height), width + gap,
                fillFrom, coverShift(fillFrom, shift), fillTo, coverShift(fillTo, shift))
        }
        fun capture(focal: Offset?): MosaicAnchor? {
            if (layouts.level(1).cells.isEmpty()) return null
            val drawn = frame()
            val point = focal?.takeIf { it.x.isFinite() && it.y.isFinite() } ?: Offset.Zero
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
        fun settle() {
            settling?.cancel()
            settling = scope.launch {
                val start = mosaic.position
                val xWeight = anchorXWeight
                if (moving) {
                    animate(0f, 1f, animationSpec = tween(SETTLE_MILLIS, easing = SettleEasing)) { value, _ ->
                        mosaic.position = start + (mosaic.columns - start) * value
                        anchorXWeight = xWeight * (1f - value)
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
        val captureLatest by rememberUpdatedState<(Offset?) -> MosaicAnchor?>(::capture)
        val hopLatest by rememberUpdatedState<() -> Unit>(::hop)
        val alignLatest by rememberUpdatedState<() -> Unit>(::alignAnchor)
        val settleLatest by rememberUpdatedState<() -> Unit>(::settle)
        val latestLayouts by rememberUpdatedState(layouts)
        val latestRows by rememberUpdatedState(rows)
        val latestHeaderItems by rememberUpdatedState(headerItems)
        DisposableEffect(mosaic, state) {
            onDispose {
                settling?.cancel()
                if (anchor == null) anchor = captureLatest(null)
                mosaic.cancelPinch()
                mosaic.position = mosaic.columns.toFloat()
                mosaic.referenceColumns = mosaic.columns
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
                    latestRows.size + latestHeaderItems, !current.loading && !current.refreshing && !current.endReached && current.error == null)
            }.distinctUntilChanged().filter { it.ready && it.last >= it.total - 3 }.collect { latestMore() }
        }
        Box(Modifier.fillMaxSize().clipToBounds()
            .semantics {
                contentDescription = "Card gallery"
                stateDescription = "${mosaic.columns} columns"
                verticalScrollAxisRange = ScrollAxisRange(
                    value = { scrollProgress() },
                    maxValue = { if (state.canScrollForward) state.layoutInfo.totalItemsCount.toFloat() else scrollProgress() },
                    reverseScrolling = false,
                )
                scrollToIndex { index ->
                    if (index < 0 || index >= rows.size + headerItems) false else {
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
            .scrollable(state, Orientation.Vertical, reverseDirection = true, enabled = !mosaic.isPinching, overscrollEffect = overscroll)
            .pointerInput(mosaic, state) {
                try {
                    awaitEachGesture {
                        val firstDown = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                        // A press stops a landing where it is, so the card under the finger stays there until up.
                        settling?.cancel()
                        var pinched = false
                        var dragged = false
                        var pointers = emptySet<androidx.compose.ui.input.pointer.PointerId>()
                        do {
                            val event = awaitPointerEvent(PointerEventPass.Initial)
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
                                    scope.launch(start = CoroutineStart.UNDISPATCHED) { state.stopScroll(MutatePriority.PreventUserInput) }
                                } else {
                                    // The focal point stays where the fingers met: a pinch resizes but never pans.
                                    mosaic.zoomPinch(event.calculateZoom())
                                    alignLatest()
                                    hopLatest()
                                }
                            }
                            if (pinched) event.changes.forEach { it.consume() }
                            pointers = pressed
                        } while (event.changes.any { it.pressed })
                        if (pinched) {
                            mosaic.finishPinch()
                            settleLatest()
                        } else {
                            val needsSettle = mosaic.position != mosaic.columns.toFloat() || anchor != null
                            if (needsSettle && anchor == null) anchor = captureLatest(null)
                            if (needsSettle) settleLatest()
                        }
                    }
                } finally {
                    mosaic.cancelPinch()
                }
            }
            .sheetEdgeFade(top = 0.dp, bottom = 130.dp, color = colors.background)) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(), state = state, userScrollEnabled = false,
                contentPadding = PaddingValues(bottom = 156.dp),
            ) {
                if (header != null) item(key = MOSAIC_HEADER, contentType = "header") {
                    Column(Modifier.onSizeChanged { headerHeight = it.height }.padding(bottom = MOSAIC_GAP)) { header() }
                }
                // These semantic-free slots retain LazyListState and fling/viewport bookkeeping while cards regroup.
                itemsIndexed(rows, key = { _, row -> "$MOSAIC_ROW${row.key}" }, contentType = { _, _ -> "bricks" }) { index, _ ->
                    Layout(content = {}, modifier = Modifier.fillMaxWidth()) { _, constraints ->
                        layout(constraints.maxWidth, layouts.slotHeight(index, referenceColumns, mosaic.position).roundToInt()) {}
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
            }
            if (feed.cards.isNotEmpty() && layouts.level(1).cells.isNotEmpty()) {
                // Header/footer remain real LazyColumn children; temporary projected edges clip only inside the body.
                Box(Modifier.fillMaxSize().graphicsLayer {
                    val visible = state.layoutInfo.visibleItemsInfo
                    val top = visible.firstOrNull { it.key == MOSAIC_HEADER }?.let { (it.offset + it.size).toFloat() } ?: 0f
                    val bottom = visible.firstOrNull { it.key == "gallery-footer" }?.offset?.toFloat() ?: size.height
                    shape = MosaicBodyShape(top, bottom)
                    clip = true
                }.background(colors.background)) {
                    val latestFrame by rememberUpdatedState<() -> MosaicFrame>(::frame)
                    val window by remember(layouts, height) { derivedStateOf { latestFrame().window(height) } }
                    // Which cards change line or shape decides what is composed; it changes once per step, not per frame.
                    val shape by remember(layouts, height) { derivedStateOf { latestFrame().shape(window, layouts) } }
                    val focalIndex = anchor?.cardId?.let(layouts.byId::get)
                    if (shape != null) MosaicFill(layouts, mosaic, height, ::frame, blur = with(density) { BACKDROP_BLUR.toPx() })
                    ProjectedMosaicCards(layouts, mosaic, window.filter { it != focalIndex }, shape, ::frame,
                        Modifier.fillMaxSize().testTag("mosaic-projection"), onOpenCard, onLongPressCard)
                    if (focalIndex != null) key("focal", layouts.cards[focalIndex].id) {
                        ProjectedMosaicCards(layouts, mosaic, listOf(focalIndex), shape, ::frame, Modifier.fillMaxSize(), onOpenCard, onLongPressCard)
                    }
                }
            }
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

/** What a step composes beyond one tile per card: copies of the cards changing line, old layouts of words changing shape. */
private data class MosaicShape(val wrapping: Set<Int>, val reshaping: Set<Int>)

/** One child of [ProjectedMosaicCards]: card [index]'s tile or its [copy], or the layout drawn [under] it while it changes shape. */
private data class MosaicSlot(val index: Int, val copy: Boolean, val under: Boolean)

/**
 * Where each card is drawn now, in viewport pixels, while the fingers move it between its places in the grids of two
 * stops (Tavi's pick, version O3b of the "Calmer steps for O" canvas, 28 October 2026). A card that stays on its line
 * slides to its new place; one that changes line slides out past one edge of its old line and in from the other edge
 * of its new one, shrinking a little and fading between the two drawings. Lines further down start later, like a
 * wave. Behind the cards both grids are drawn whole and blurred ([fill]), so no blank paper shows between them.
 */
private class MosaicFrame(
    private val from: MosaicProjection, private val to: MosaicProjection, private val x: Float, val y: Float,
    private val progress: Float, private val wave: MosaicWave, private val wrap: Float,
    private val fillFrom: MosaicProjection, private val fillFromX: Float,
    private val fillTo: MosaicProjection, private val fillToX: Float,
) {
    val fromColumns: Int get() = from.columns
    val toColumns: Int get() = to.columns

    /** True between two stops, while cards are on their way and the blurred fill shows behind them. */
    val moving: Boolean get() = from.columns != to.columns && progress > 0f && progress < 1f

    private fun place(projection: MosaicProjection, index: Int): Rect = projection.cell(index).translate(Offset(x, -y))

    /**
     * Whether card [index] changes line. Its top decides, not its centre: a card two squares high in one grid and one in
     * the other keeps its line, as the focal card always does. Both grids scale together, so the answer holds for the
     * whole step.
     */
    fun wraps(index: Int): Boolean {
        if (!moving) return false
        val a = place(from, index)
        val b = place(to, index)
        return abs(a.top - b.top) > 0.5f * minOf(a.height, b.height)
    }

    fun draws(index: Int): MosaicDrawings {
        val a = place(from, index)
        val b = place(to, index)
        val m = wave.progress(index, progress)
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

    fun shape(window: IntRange, layouts: MosaicLayouts): MosaicShape? = if (!moving) null else MosaicShape(
        window.filter(::wraps).toSet(),
        window.filter { !galleryTileIsPicture(layouts.cards[it]) && layouts.wordsFrame(it, from.columns) != layouts.wordsFrame(it, to.columns) }.toSet(),
    )

    /** The cards each blurred grid shows, by its columns: the new grid first, drawn whole, then the old one fading out over it. */
    fun fill(viewport: Float): List<Pair<Int, IntRange>> =
        listOf(fillTo.columns to fillTo.window(y, viewport), fillFrom.columns to fillFrom.window(y, viewport))

    fun fillCell(columns: Int, index: Int): Rect =
        if (columns == fillTo.columns) fillTo.cell(index).translate(Offset(fillToX, -y)) else fillFrom.cell(index).translate(Offset(fillFromX, -y))

    fun fillAlpha(columns: Int): Float = if (columns == fillTo.columns) 1f else 1f - progress

    /** How strongly the whole blurred fill shows now ([mosaicFillShown]). */
    val fillShown: Float get() = mosaicFillShown(progress)
}

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
        for (columns in setOf(between.from, between.to)) {
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
        wave = MosaicWave(first, last, focal)
        return wave
    }
}

/** The children [ProjectedMosaicCards] composes for card [index], in drawing order. */
private fun mosaicSlots(index: Int, shape: MosaicShape?): List<MosaicSlot> {
    val copy = shape?.wrapping?.contains(index) == true
    val under = shape?.reshaping?.contains(index) == true
    return listOfNotNull(
        MosaicSlot(index, copy = false, under = true).takeIf { under },
        MosaicSlot(index, copy = false, under = false),
        MosaicSlot(index, copy = true, under = true).takeIf { copy && under },
        MosaicSlot(index, copy = true, under = false).takeIf { copy },
    )
}

/**
 * Draws [indices] at their places in the current [MosaicFrame]. Each card has one tile that takes touches and
 * semantics; a card changing line also has an inert copy, and a tile with words changing shape has its layout in the
 * other grid drawn under it, the two cross-fading with the card's progress.
 */
@Composable
private fun ProjectedMosaicCards(
    layouts: MosaicLayouts, mosaic: MosaicState, indices: List<Int>, shape: MosaicShape?, frame: () -> MosaicFrame,
    modifier: Modifier, onOpenCard: (Card) -> Unit, onLongPressCard: ((Card) -> Unit)?,
) {
    val slots = indices.flatMap { mosaicSlots(it, shape) }
    Layout(modifier = modifier, content = {
        indices.forEach { index ->
            key(layouts.cards[index].id) {
                val card = layouts.cards[index]
                val imageSize = imageRequestSize(layouts, mosaic, index)
                mosaicSlots(index, shape).forEach { slot ->
                    if (slot.copy || slot.under) MosaicCopy(card, imageSize) else GalleryTile(card, { if (!mosaic.isPinching) onOpenCard(card) },
                        modifier = Modifier.fillMaxSize(), enabled = !mosaic.isPinching,
                        onLongClick = onLongPressCard?.let { press -> { if (!mosaic.isPinching) press(card) } },
                        imageRequestSizePx = imageSize)
                }
            }
        }
    }) { children, constraints ->
        val current = frame()
        val drawings = HashMap<Int, MosaicDrawings>()
        val placed = children.zip(slots) { child, slot ->
            val drawn = drawings.getOrPut(slot.index) { current.draws(slot.index) }
            val draw = if (slot.copy) drawn.copy ?: drawn.card else drawn.card
            val m = drawn.progress
            val words = !galleryTileIsPicture(layouts.cards[slot.index])
            // A tile with words is laid out in the grid it is nearer; its layout in the other one is drawn under it.
            val near = if (m < 0.5f) current.fromColumns else current.toColumns
            val far = if (m < 0.5f) current.toColumns else current.fromColumns
            val reshapes = shape?.reshaping?.contains(slot.index) == true
            val alpha = draw.alpha * if (reshapes && !slot.under) maxOf(m, 1f - m) else 1f
            val tile = child.measureTile(draw.rect, layouts.wordsFrame(slot.index, if (slot.under) far else near).takeIf { words })
            PlacedTile(tile, draw.rect, words, alpha)
        }
        layout(constraints.maxWidth, constraints.maxHeight) {
            placed.forEach { placeTile(it.tile, it.cell, it.words, it.alpha) }
        }
    }
}

/** A tile measured for its cell, as [placeTile] places it. */
private class PlacedTile(val tile: Placeable, val cell: Rect, val words: Boolean, val alpha: Float)

/** An inert drawing of [card]: no semantics, touches or playing video. */
@Composable
private fun MosaicCopy(card: Card, imageRequestSizePx: Int) {
    Box(Modifier.fillMaxSize().clearAndSetSemantics {}) {
        GalleryTile(card, {}, enabled = false, imageRequestSizePx = imageRequestSizePx, playVideo = false)
    }
}

/** Both grids drawn whole and blurred behind the moving cards ([MosaicFrame.fill]), so no blank paper shows. */
@Composable
private fun MosaicFill(layouts: MosaicLayouts, mosaic: MosaicState, height: Float, frame: () -> MosaicFrame, blur: Float) {
    val latestFrame by rememberUpdatedState(frame)
    val grids by remember(layouts, height) { derivedStateOf { latestFrame().fill(height) } }
    val slots = grids.flatMap { (columns, cards) -> cards.map { columns to it } }
    val blurred = Modifier.fillMaxSize().clearAndSetSemantics {}.graphicsLayer {
        renderEffect = BlurEffect(blur, blur, TileMode.Clamp)
        alpha = latestFrame().fillShown
    }
    Layout(modifier = blurred,
        content = {
            slots.forEach { (columns, index) ->
                key(columns, layouts.cards[index].id) { GalleryTileBackdrop(layouts.cards[index], imageRequestSize(layouts, mosaic, index)) }
            }
        }) { children, constraints ->
        val current = frame()
        val placed = children.zip(slots) { child, (columns, index) ->
            val cell = current.fillCell(columns, index)
            PlacedTile(child.measure(cell.size.fixed()), cell, false, current.fillAlpha(columns))
        }
        layout(constraints.maxWidth, constraints.maxHeight) {
            placed.forEach { placeTile(it.tile, it.cell, it.words, it.alpha) }
        }
    }
}

/** The size gallery pictures are decoded at: the card's cell at the resting density, so a pinch never changes it. */
private fun imageRequestSize(layouts: MosaicLayouts, mosaic: MosaicState, index: Int): Int =
    layouts.level(mosaic.columns).cells[index].let { maxOf(it.width, it.height) }.roundToInt().coerceAtLeast(1)

/**
 * Measures a tile for [cell]. A picture is cut to the cell itself; a tile with words is laid out at its [frame]
 * ([MosaicLayouts.wordsFrame]), so its words keep their lines however far the fingers move.
 */
private fun Measurable.measureTile(cell: Rect, frame: Size?): Placeable = measure((frame ?: cell.size).fixed())

/** Exact constraints for a size in whole pixels. */
private fun Size.fixed(): Constraints = Constraints.fixed(width.roundToInt().coerceAtLeast(1), height.roundToInt().coerceAtLeast(1))

/**
 * Places a tile measured by [measureTile] in [cell] at [alpha]; a tile with words is scaled from its frame, as a
 * picture zooms.
 */
private fun Placeable.PlacementScope.placeTile(tile: Placeable, cell: Rect, words: Boolean, alpha: Float) {
    val x = cell.left.roundToInt()
    val y = cell.top.roundToInt()
    if (!words && alpha >= 1f) {
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
    }
}

package com.octaviantocan.mymind.ui.library

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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.Layout
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
import androidx.compose.ui.semantics.testTag as projectionTestTag
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.verticalScrollAxisRange
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.CardSheetMotion
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.card.sheetEdgeFade
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private val MOSAIC_GAP = 3.dp
private const val MOSAIC_HEADER = "gallery-header"
private const val MOSAIC_ROW = "row:"
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
    var headerHeight by remember { mutableIntStateOf(0) }
    var footerHeight by remember { mutableIntStateOf(0) }
    var restingX by remember { mutableFloatStateOf(0f) }
    var anchorXWeight by remember { mutableFloatStateOf(1f) }
    var pivotId by remember { mutableStateOf<String?>(null) }
    var sceneColumns by remember { mutableIntStateOf(mosaic.columns) }
    var outgoingColumns by remember { mutableStateOf<Int?>(null) }
    var sceneAlpha by remember { mutableFloatStateOf(1f) }
    var pressedColumns by remember { mutableStateOf<Int?>(null) }
    val latestFeed by rememberUpdatedState(feed)
    val latestMore by rememberUpdatedState(onLoadMore)
    val headerItems = if (header == null) 0 else 1
    BoxWithConstraints(modifier.fillMaxSize()) {
        val width = with(density) { maxWidth.toPx() }
        val height = with(density) { maxHeight.toPx() }
        val gap = with(density) { MOSAIC_GAP.toPx() }
        val bottomPadding = with(density) { 156.dp.toPx() }
        val layouts = remember(feed.cards, width, gap) { MosaicLayouts(feed.cards, width, gap) }
        val rows = layouts.level(mosaic.referenceColumns).rows

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
        fun projection(columns: Int): MosaicProjection = layouts.projection(columns, mosaic.position, pivot())
        fun activeColumns(): Int = pressedColumns ?: outgoingColumns?.takeIf { sceneAlpha < 0.5f } ?: sceneColumns
        fun capture(focal: Offset?): MosaicAnchor? {
            if (layouts.level(1).cells.isEmpty()) return null
            val y = origin()
            val x = horizontalShift(y)
            val point = focal?.takeIf { it.x.isFinite() && it.y.isFinite() } ?: Offset.Zero
            var nearest = -1
            var distance = Float.MAX_VALUE
            val projected = projection(activeColumns())
            for (index in projected.window(y, height)) {
                val cell = projected.cell(index).translate(Offset(x, -y))
                if (cell.bottom <= 0f || cell.top >= height) continue
                val dx = point.x - point.x.coerceIn(cell.left, cell.right)
                val dy = point.y - point.y.coerceIn(cell.top, cell.bottom)
                if (dx * dx + dy * dy < distance) {
                    distance = dx * dx + dy * dy
                    nearest = index
                }
            }
            if (nearest < 0) return null
            val cell = projected.cell(nearest).translate(Offset(x, -y))
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
                    animate(0f, 1f, animationSpec = tween(CardSheetMotion.FAST)) { value, _ ->
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
        val captureLatest by rememberUpdatedState<(Offset?) -> MosaicAnchor?>(::capture)
        val alignLatest by rememberUpdatedState<() -> Unit>(::alignAnchor)
        val settleLatest by rememberUpdatedState<() -> Unit>(::settle)
        val latestLayouts by rememberUpdatedState(layouts)
        val latestRows by rememberUpdatedState(rows)
        val latestHeaderItems by rememberUpdatedState(headerItems)
        val nearestScene = mosaic.position.roundToInt().coerceIn(1, 4)
        val targetScene = pressedColumns ?: if (mosaic.isPinching && mosaic.position in (sceneColumns - 0.55f)..(sceneColumns + 0.55f)) sceneColumns else nearestScene
        LaunchedEffect(targetScene, moving) {
            val dominant = activeColumns()
            sceneColumns = targetScene
            outgoingColumns = dominant.takeIf { it != targetScene }
            sceneAlpha = if (outgoingColumns == null || !moving) 1f else 0f
            if (outgoingColumns != null && moving) {
                animate(0f, 1f, animationSpec = tween(90)) { value, _ -> sceneAlpha = value }
            }
            outgoingColumns = null
        }
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
                        // Compose fixes the pointer hit path on down. Keep that scene and its focal node alive until up.
                        pressedColumns = activeColumns()
                        settling?.cancel()
                        var pinched = false
                        var dragged = false
                        var pointers = emptySet<androidx.compose.ui.input.pointer.PointerId>()
                        var centroidOffset = Offset.Zero
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
                                    centroidOffset = anchor?.viewport?.minus(point) ?: Offset.Zero
                                    pressedColumns = null
                                    mosaic.beginPinch()
                                    scope.launch(start = CoroutineStart.UNDISPATCHED) { state.stopScroll(MutatePriority.PreventUserInput) }
                                } else {
                                    anchor = anchor?.copy(viewport = point + centroidOffset)
                                    mosaic.zoomPinch(event.calculateZoom())
                                    alignLatest()
                                }
                            }
                            if (pinched) event.changes.forEach { it.consume() }
                            pointers = pressed
                        } while (event.changes.any { it.pressed })
                        if (pinched) {
                            pressedColumns = null
                            mosaic.finishPinch()
                            settleLatest()
                        } else {
                            val needsSettle = mosaic.position != mosaic.columns.toFloat() || anchor != null
                            if (needsSettle && anchor == null) anchor = captureLatest(null)
                            pressedColumns = null
                            if (needsSettle) settleLatest()
                        }
                    }
                } finally {
                    pressedColumns = null
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
                        layout(constraints.maxWidth, layouts.slotHeight(index, mosaic.referenceColumns, mosaic.position).roundToInt()) {}
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
                }) {
                    val active = activeColumns()
                    val focalIndex = anchor?.cardId?.let(layouts.byId::get)
                    for (columns in listOfNotNull(outgoingColumns, sceneColumns).distinct()) key(columns) {
                        val alpha = if (columns == sceneColumns) sceneAlpha else 1f
                        ProjectedMosaicCards(layouts, mosaic, state, height, focalIndex,
                            projection = { projection(columns) }, origin = ::origin, shiftX = { horizontalShift(origin()) },
                            activeInteraction = columns == active,
                            modifier = Modifier.fillMaxSize().graphicsLayer { this.alpha = alpha }.background(colors.background)
                                .then(if (columns == active) Modifier.testTag("mosaic-projection") else Modifier.clearAndSetSemantics { projectionTestTag = "mosaic-projection" }),
                            onOpenCard = onOpenCard, onLongPressCard = onLongPressCard)
                    }
                    if (focalIndex != null) key("focal", layouts.cards[focalIndex].id) {
                        val card = layouts.cards[focalIndex]
                        val imageCell = layouts.level(mosaic.columns).cells[focalIndex]
                        Layout(modifier = Modifier.fillMaxSize(), content = {
                            GalleryTile(card, { if (!mosaic.isPinching) onOpenCard(card) },
                                enabled = !mosaic.isPinching, onLongClick = onLongPressCard?.let { press -> { if (!mosaic.isPinching) press(card) } },
                                imageRequestSizePx = maxOf(imageCell.width, imageCell.height).roundToInt().coerceAtLeast(1))
                        }) { children, constraints ->
                            val cell = layouts.cell(focalIndex, mosaic.position)
                            val child = children.single().measure(Constraints.fixed(cell.width.roundToInt().coerceAtLeast(1), cell.height.roundToInt().coerceAtLeast(1)))
                            val y = origin()
                            val x = horizontalShift(y)
                            layout(constraints.maxWidth, constraints.maxHeight) { child.place((cell.left + x).roundToInt(), (cell.top - y).roundToInt()) }
                        }
                    }
                }
            }
        }
    }
}

/** Draws one intact grid; the other projection never supplies this grid's card positions. */
@Composable
private fun ProjectedMosaicCards(
    layouts: MosaicLayouts, mosaic: MosaicState, state: LazyListState, height: Float, focalIndex: Int?,
    projection: () -> MosaicProjection, origin: () -> Float, shiftX: () -> Float, activeInteraction: Boolean, modifier: Modifier,
    onOpenCard: (Card) -> Unit, onLongPressCard: ((Card) -> Unit)?,
) {
    val latestProjection by rememberUpdatedState(projection)
    val latestOrigin by rememberUpdatedState(origin)
    val window by remember(layouts, state, height) { derivedStateOf { latestProjection().window(latestOrigin(), height) } }
    val drawn = window.filter { it != focalIndex }
    Layout(modifier = modifier, content = {
        for (index in drawn) key(layouts.cards[index].id) {
            val card = layouts.cards[index]
            val imageCell = layouts.level(mosaic.columns).cells[index]
            GalleryTile(card, { if (activeInteraction && !mosaic.isPinching) onOpenCard(card) },
                modifier = Modifier.fillMaxSize(), enabled = activeInteraction && !mosaic.isPinching,
                onLongClick = onLongPressCard?.let { press -> { if (activeInteraction && !mosaic.isPinching) press(card) } },
                imageRequestSizePx = maxOf(imageCell.width, imageCell.height).roundToInt().coerceAtLeast(1))
        }
    }) { children, constraints ->
        val projected = projection()
        val y = origin()
        val x = shiftX()
        val measured = children.mapIndexed { at, child ->
            val cell = projected.cell(drawn[at])
            child.measure(Constraints.fixed(cell.width.roundToInt().coerceAtLeast(1), cell.height.roundToInt().coerceAtLeast(1)))
        }
        layout(constraints.maxWidth, constraints.maxHeight) {
            measured.forEachIndexed { at, child ->
                val cell = projected.cell(drawn[at])
                child.place((cell.left + x).roundToInt(), (cell.top - y).roundToInt())
            }
        }
    }
}

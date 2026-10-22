package com.octaviantocan.mymind.ui.library

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.MutatePriority
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.stopScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.CardSheetMotion
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.card.sheetEdgeFade
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private val MOSAIC_GAP = 3.dp
private const val MOSAIC_HEADER = "gallery-header"

private data class MosaicAnchor(val cardId: String, val top: Int)
private data class MosaicLoadPosition(val count: Int, val last: Int, val total: Int, val ready: Boolean)

/**
 * Lazy justified bricks with a saveable tile size. A two-finger pinch previews zoom, then settles at the next density
 * while keeping the card under the fingers in view. The profile/highlights header never participates in the zoom.
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
    var pendingAnchor by remember { mutableStateOf<MosaicAnchor?>(null) }
    val latestFeed by rememberUpdatedState(feed)
    val latestMore by rememberUpdatedState(onLoadMore)
    val headerItems = if (header == null) 0 else 1
    val latestHeaderItems by rememberUpdatedState(headerItems)
    BoxWithConstraints(modifier.fillMaxSize()) {
        val width = with(density) { maxWidth.toPx() }
        val gap = with(density) { MOSAIC_GAP.toPx() }
        val rows = remember(feed.cards, width, gap, mosaic.columns) { mosaicRows(feed.cards, width, mosaic.columns, gap) }
        val latestRows by rememberUpdatedState(rows)
        val latestWidth by rememberUpdatedState(width)
        val capture by rememberUpdatedState<(Offset?) -> MosaicAnchor?>({ focal ->
            val visible = state.layoutInfo.visibleItemsInfo.filter { it.index - latestHeaderItems in latestRows.indices }
            val item = focal?.let { point -> visible.firstOrNull { point.y >= it.offset && point.y < it.offset + it.size } }
                ?: visible.firstOrNull()
            val row = item?.let { latestRows.getOrNull(it.index - latestHeaderItems) }
            if (row == null) null else {
                var edge = 0f
                val at = focal?.x ?: 0f
                val cell = row.widths.indexOfFirst { cellWidth ->
                    val end = edge + cellWidth
                    val contains = at < end
                    edge = end + gap
                    contains
                }.takeIf { it >= 0 } ?: row.cards.lastIndex
                MosaicAnchor(row.cards[cell].id, item.offset)
            }
        })
        fun resize(change: () -> Boolean): Boolean {
            val anchor = capture(null)
            if (!change()) return false
            pendingAnchor = anchor
            return true
        }
        LaunchedEffect(rows, pendingAnchor) {
            val anchor = pendingAnchor ?: return@LaunchedEffect
            val row = rows.indexOfFirst { packed -> packed.cards.any { it.id == anchor.cardId } }
            if (row >= 0) state.requestScrollToItem(row + headerItems, -anchor.top)
            pendingAnchor = null
        }
        SideEffect {
            if (state.firstVisibleItemIndex == 0 && state.firstVisibleItemScrollOffset == 0 && !state.isScrollInProgress && !mosaic.isPinching) {
                state.requestScrollToItem(0)
            }
        }
        LaunchedEffect(state) {
            snapshotFlow {
                val current = latestFeed
                MosaicLoadPosition(
                    count = current.cards.size,
                    last = state.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1,
                    total = latestRows.size + latestHeaderItems,
                    ready = !current.loading && !current.refreshing && !current.endReached && current.error == null,
                )
            }.distinctUntilChanged().filter { it.ready && it.last >= it.total - 3 }.collect { latestMore() }
        }
        LazyColumn(
            modifier = Modifier.fillMaxSize().clipToBounds()
                .semantics {
                    contentDescription = "Card gallery"
                    stateDescription = "${mosaic.columns} columns"
                    customActions = listOf(
                        CustomAccessibilityAction("Larger cards") { resize(mosaic::larger) },
                        CustomAccessibilityAction("Smaller cards") { resize(mosaic::smaller) },
                    )
                }
                .pointerInput(mosaic, state) {
                    try {
                        awaitEachGesture {
                            awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                            var pinched = false
                            var anchor: MosaicAnchor? = null
                            do {
                                val event = awaitPointerEvent(PointerEventPass.Initial)
                                val pressed = event.changes.count { it.pressed }
                                if (!pinched && pressed >= 2) {
                                    pinched = true
                                    val point = event.calculateCentroid()
                                    anchor = capture(point)
                                    mosaic.beginPinch()
                                    scope.launch { state.stopScroll(MutatePriority.PreventUserInput) }
                                }
                                if (pinched) {
                                    if (pressed >= 2) {
                                        val point = event.calculateCentroid()
                                        if (point.x.isFinite() && latestWidth > 0f) mosaic.focalX = (point.x / latestWidth).coerceIn(0f, 1f)
                                        mosaic.zoomPinch(event.calculateZoom())
                                    }
                                    event.changes.forEach { it.consume() }
                                }
                            } while (event.changes.any { it.pressed })
                            if (pinched) {
                                val previous = mosaic.columns
                                mosaic.finishPinch()
                                if (mosaic.columns != previous) pendingAnchor = anchor
                            }
                        }
                    } finally {
                        mosaic.cancelPinch()
                    }
                }
                .sheetEdgeFade(top = 0.dp, bottom = 130.dp, color = colors.background),
            state = state, userScrollEnabled = !mosaic.isPinching,
            contentPadding = PaddingValues(bottom = 156.dp), verticalArrangement = Arrangement.spacedBy(MOSAIC_GAP),
        ) {
            if (header != null) item(key = MOSAIC_HEADER, contentType = "header") { header() }
            items(rows, key = { "row:${it.key}" }, contentType = { "bricks" }) { row ->
                val height by animateDpAsState(
                    with(density) { row.height.toDp() },
                    animationSpec = if (moving) tween(CardSheetMotion.FAST, easing = CardSheetMotion.SmoothOut) else snap(),
                    label = "mosaic-height",
                )
                val placement = if (moving && !mosaic.isPinching) spring<androidx.compose.ui.unit.IntOffset>() else null
                MosaicZoomRow(
                    height, mosaic.previewScale, mosaic.focalX,
                    Modifier.animateItem(fadeInSpec = if (moving) tween(250) else null, fadeOutSpec = if (moving) tween(150) else null, placementSpec = placement),
                ) {
                    Row(Modifier.fillMaxSize(), horizontalArrangement = Arrangement.spacedBy(MOSAIC_GAP)) {
                        row.cards.forEachIndexed { index, card ->
                            GalleryTile(
                                card, onClick = { if (!mosaic.isPinching) onOpenCard(card) },
                                modifier = Modifier.width(with(density) { row.widths[index].toDp() }).fillMaxHeight(),
                                onLongClick = onLongPressCard?.let { press -> { if (!mosaic.isPinching) press(card) } },
                                enabled = !mosaic.isPinching,
                            )
                        }
                    }
                }
            }
            if (feed.cards.isNotEmpty() && (feed.loading || feed.error != null)) item(key = "gallery-footer", contentType = "footer") {
                Column(Modifier.fillMaxWidth().padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (feed.loading) LoadingSpinner(color = colors.accent)
                    feed.error?.let { message ->
                        Text(message, color = colors.muted, style = LibraryStyle.label)
                        Text("Retry", Modifier.clickable(role = Role.Button, onClick = onLoadMore).padding(14.dp), color = colors.text, style = LibraryStyle.body)
                    }
                }
            }
        }
    }
}

/** Reserves the scaled height as fingers zoom, without scaling the library's profile or highlights header. */
@Composable
private fun MosaicZoomRow(height: Dp, scale: Float, focalX: Float, modifier: Modifier, content: @Composable () -> Unit) {
    Layout(content = content, modifier = modifier.fillMaxWidth()) { children, constraints ->
        val baseHeight = height.roundToPx().coerceAtLeast(1)
        val child = children.single().measure(Constraints.fixed(constraints.maxWidth, baseHeight))
        layout(constraints.maxWidth, (baseHeight * scale).roundToInt().coerceAtLeast(1)) {
            child.placeWithLayer(0, 0) {
                scaleX = scale
                scaleY = scale
                transformOrigin = TransformOrigin(focalX.coerceIn(0f, 1f), 0f)
            }
        }
    }
}

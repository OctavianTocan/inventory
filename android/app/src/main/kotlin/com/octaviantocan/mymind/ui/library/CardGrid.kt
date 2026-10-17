package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridScope
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.foundation.lazy.staggeredgrid.LazyVerticalStaggeredGrid
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridCells
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.items
import androidx.compose.foundation.lazy.staggeredgrid.rememberLazyStaggeredGridState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.shell.PAGE_BOTTOM_GAP
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter

/** Space the web page leaves under the grid (`body.android main`). */
val GRID_BOTTOM_PADDING = 30.dp

private const val LOAD_AHEAD = 6

/**
 * The two-column masonry grid of cards, loading the next page near the end.
 *
 * @param feed Cards and loading state.
 * @param onLoadMore Called when the end of the loaded cards comes near.
 * @param onOpenCard Called with the tapped card.
 * @param modifier Modifier for the grid.
 * @param state Scroll position.
 * @param top Space above the first row.
 * @param onLongPressCard Called with a card the user long-pressed, or null to ignore long presses.
 * @param header Full-width items placed before the cards.
 */
@Composable
fun CardGrid(
    feed: FeedState,
    onLoadMore: () -> Unit,
    onOpenCard: (Card) -> Unit,
    modifier: Modifier = Modifier,
    state: LazyStaggeredGridState = rememberLazyStaggeredGridState(),
    top: Dp = PAGE_GUTTER,
    onLongPressCard: ((Card) -> Unit)? = null,
    header: (LazyStaggeredGridScope.() -> Unit)? = null,
) {
    LaunchedEffect(state) {
        snapshotFlow {
            val last = state.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1
            last >= state.layoutInfo.totalItemsCount - LOAD_AHEAD
        }
            .distinctUntilChanged()
            .filter { it }
            .collect { onLoadMore() }
    }
    LazyVerticalStaggeredGrid(
        columns = StaggeredGridCells.Fixed(2),
        modifier = modifier.fillMaxSize(),
        state = state,
        contentPadding = PaddingValues(start = PAGE_GUTTER, end = PAGE_GUTTER, top = top, bottom = GRID_BOTTOM_PADDING),
        horizontalArrangement = Arrangement.spacedBy(PAGE_GUTTER),
        verticalItemSpacing = PAGE_GUTTER,
    ) {
        header?.invoke(this)
        items(feed.cards, key = { it.id }) { card ->
            CardTile(card, onClick = { onOpenCard(card) }, onLongClick = onLongPressCard?.let { { it(card) } })
        }
        if (feed.cards.isNotEmpty() && (feed.loading || feed.error != null)) {
            item(span = StaggeredGridItemSpan.FullLine) { GridFooter(feed, onLoadMore) }
        }
    }
}

/** Spinner while the next page loads, or the error with a retry button. */
@Composable
private fun GridFooter(feed: FeedState, onRetry: () -> Unit) {
    Column(Modifier.fillMaxWidth().padding(vertical = 16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        if (feed.loading) LoadingSpinner(color = MymindTheme.chrome.tabUnselected)
        feed.error?.let {
            Text(it, style = TextStyle(color = MymindTheme.page.muted, fontSize = 14.sp, fontFamily = MymindTheme.fonts.nunito))
            TextButton(onClick = onRetry) { Text("Retry") }
        }
    }
}

/**
 * The horizontal Top of Mind row above the Everything grid.
 *
 * @param cards Pinned cards in pin order.
 * @param onOpenCard Called with the tapped card.
 * @param modifier Modifier for the row.
 * @param onLoadMore Called when the end of the loaded pins comes near.
 */
@Composable
fun TopOfMindRow(cards: List<Card>, onOpenCard: (Card) -> Unit, modifier: Modifier = Modifier, onLoadMore: () -> Unit = {}) {
    val state = rememberLazyListState()
    SideEffect {
        // Without this, a new first pin lands left of a row resting at its start.
        if (state.firstVisibleItemIndex == 0 && state.firstVisibleItemScrollOffset == 0 && !state.isScrollInProgress) {
            state.requestScrollToItem(0)
        }
    }
    LaunchedEffect(state) {
        snapshotFlow {
            val last = state.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1
            last >= state.layoutInfo.totalItemsCount - LOAD_AHEAD
        }
            .distinctUntilChanged()
            .filter { it }
            .collect { onLoadMore() }
    }
    LazyRow(
        modifier.fillMaxWidth().height(100.dp),
        state = state,
        contentPadding = PaddingValues(horizontal = PAGE_GUTTER),
        horizontalArrangement = Arrangement.spacedBy(PAGE_GUTTER),
    ) {
        items(cards, key = { it.id }) { card -> TopOfMindTile(card, onClick = { onOpenCard(card) }) }
    }
}

/**
 * The spinner shown while the first page of cards loads, at the centre of the screen.
 *
 * @param modifier Modifier for the box, which fills the page between the status bar and the tab bar.
 */
@Composable
fun PageSpinner(modifier: Modifier = Modifier) {
    val statusBar = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
    Box(modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        // The real app's loading layer covers the whole screen, so its spinner sits lower than the page's centre.
        LoadingSpinner(Modifier.offset(y = (PAGE_BOTTOM_GAP - statusBar) / 2), color = MymindTheme.chrome.tabUnselected)
    }
}

/**
 * Header item holding the Top of Mind row, 11 dp above the grid as on the web page.
 *
 * @param cards Pinned cards in pin order.
 * @param onOpenCard Called with the tapped card.
 * @param onLoadMore Called when the end of the loaded pins comes near.
 */
fun LazyStaggeredGridScope.topOfMindHeader(cards: List<Card>, onOpenCard: (Card) -> Unit, onLoadMore: () -> Unit = {}) {
    if (cards.isEmpty()) return
    item(key = "top-of-mind", span = StaggeredGridItemSpan.FullLine) {
        Column {
            TopOfMindRow(cards, onOpenCard, Modifier.bleed(PAGE_GUTTER), onLoadMore)
            Spacer(Modifier.height(1.dp))
        }
    }
}

/** Widens the element by [side] on both sides, past the padding of the list holding it. */
private fun Modifier.bleed(side: Dp) = layout { measurable, constraints ->
    val extra = side.roundToPx() * 2
    val width = constraints.maxWidth + extra
    val placeable = measurable.measure(constraints.copy(minWidth = width, maxWidth = width))
    layout(constraints.maxWidth, placeable.height) { placeable.place(-extra / 2, 0) }
}

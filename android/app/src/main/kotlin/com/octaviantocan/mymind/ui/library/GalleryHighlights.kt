package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter

/** Large cover cards for the account's pins, in pin order, above the library mosaic. */
@Composable
fun GalleryHighlights(
    cards: List<Card>,
    onOpenCard: (Card) -> Unit,
    modifier: Modifier = Modifier,
    onLoadMore: () -> Unit = {},
    onLongPressCard: ((Card) -> Unit)? = null,
) {
    if (cards.isEmpty()) return
    val state = rememberLazyListState()
    val latestCards by rememberUpdatedState(cards)
    val latestMore by rememberUpdatedState(onLoadMore)
    SideEffect {
        if (state.firstVisibleItemIndex == 0 && state.firstVisibleItemScrollOffset == 0 && !state.isScrollInProgress) state.requestScrollToItem(0)
    }
    LaunchedEffect(state) {
        snapshotFlow { (state.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1) to latestCards.size }
            .distinctUntilChanged().filter { (last, size) -> size > 0 && last >= size - 3 }.collect { latestMore() }
    }
    Column(modifier.fillMaxWidth().padding(bottom = 22.dp)) {
        Text(
            "Top of Mind", Modifier.padding(start = 20.dp, end = 20.dp, top = 8.dp, bottom = 14.dp).semantics { heading() },
            style = LibraryStyle.title, color = LibraryStyle.colors.text,
        )
        LazyRow(
            Modifier.fillMaxWidth().height(232.dp), state = state,
            contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            items(cards, key = { it.id }) { card ->
                GalleryTile(card, onClick = { onOpenCard(card) }, modifier = Modifier.size(184.dp, 232.dp),
                    onLongClick = onLongPressCard?.let { { it(card) } }, highlight = true)
            }
        }
    }
}

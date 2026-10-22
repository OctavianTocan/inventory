package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.shell.FootPressTheme
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.theme.LibraryStyle

/**
 * A card type the search can be limited to, with the chip label the web front-end uses. The chips come from the
 * server's type suggestions; this list stands in until the server answers.
 *
 * @property type Card type as the API names it.
 * @property label Chip text.
 * @property slug The `type:` value the search takes for this type, as the server's type suggestions write it.
 */
enum class SearchType(val type: String, val label: String, val slug: String) {
    XPost("XPost", "X Posts", "x-post"),
    WebPage("WebPage", "Web Pages", "web-page"),
    Repository("Repository", "Repositories", "repository"),
    Screenshot("Screenshot", "Screenshots", "screenshot"),
    Article("Article", "Articles", "article"),
    Image("Image", "Images", "image"),
    Video("Video", "Videos", "video"),
    InstagramReel("InstagramReel", "Instagram Reels", "instagram-reel"),
    Note("Note", "Notes", "note"),
    YouTubeVideo("YouTubeVideo", "YouTube Videos", "youtube-video"),
    Product("Product", "Products", "product"),
    ThreadsPost("ThreadsPost", "Threads Posts", "threads-post"),
    App("SoftwareApplication", "Apps", "software-application"),
    Document("Document", "Documents", "document"),
    Snippet("Content", "Snippets", "content"),
    RedditPost("RedditPost", "Reddit Posts", "reddit-post"),
    SubstackNote("SubstackNote", "Substack Notes", "substack-note"),
    ;

    /** This type as a type suggestion, the shape the server's `/search/suggestions` sends. */
    val suggestion: Suggestion get() = Suggestion("type", label, "type:$slug")
}

/** The library gallery; its search field lives in the shell's single floating bar. */
@Composable
fun EverythingScreen(
    viewModel: LibraryViewModel,
    onOpenCard: (Card) -> Unit,
    onOpenSettings: () -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
    onLongPressCard: ((Card) -> Unit)? = null,
    searching: Boolean = false,
    onSerendipity: () -> Unit = {},
    photoUrl: String? = null,
) {
    val feed by viewModel.everything.state.collectAsStateWithLifecycle()
    val pinned by viewModel.topOfMind.state.collectAsStateWithLifecycle()
    val query by viewModel.search.query.collectAsStateWithLifecycle()
    val filter by viewModel.search.filter.collectAsStateWithLifecycle()
    val types by viewModel.search.types.collectAsStateWithLifecycle()
    val suggestions by viewModel.search.suggestions.collectAsStateWithLifecycle()
    val results by viewModel.search.results.collectAsStateWithLifecycle()
    val resultState by results.state.collectAsStateWithLifecycle()
    val focusManager = LocalFocusManager.current
    val resultsScroll = rememberLazyListState()
    LaunchedEffect(searching) { if (searching) viewModel.search.loadTypes() }
    LaunchedEffect(query, filter) { resultsScroll.scrollToItem(0) }
    EverythingContent(
        feed, searching, if (query.isNotBlank() || filter != null) resultState else null,
        types.ifEmpty { SearchType.entries.map { it.suggestion } }, filter, suggestions,
        onChoose = { viewModel.search.choose(it); focusManager.clearFocus() },
        onRemoveFilter = { viewModel.search.setFilter(null) },
        onLoadMore = viewModel.everything::loadMore, onLoadMoreResults = results::loadMore,
        onRetry = viewModel.everything::reload, onOpenCard = onOpenCard,
        onOpenSettings = onOpenSettings, onAdd = onAdd, modifier = modifier,
        topOfMind = pinned.cards, onLoadMoreTopOfMind = viewModel.topOfMind::loadMore,
        onRefresh = viewModel::refreshEverything, onRefreshResults = results::refresh,
        onLongPressCard = onLongPressCard, onSerendipity = onSerendipity,
        photoUrl = photoUrl, resultsState = resultsScroll,
    )
}

/** Profile controls stay fixed above the scrolling pin rail and cards; search and browsing retain independent positions. */
@Composable
fun EverythingContent(
    feed: FeedState,
    searching: Boolean,
    searchResults: FeedState?,
    typeChips: List<Suggestion>,
    filter: SearchFilter?,
    suggestions: List<Suggestion>,
    onChoose: (Suggestion) -> Unit,
    onRemoveFilter: () -> Unit,
    onLoadMore: () -> Unit,
    onLoadMoreResults: () -> Unit,
    onRetry: () -> Unit,
    onOpenCard: (Card) -> Unit,
    onOpenSettings: () -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
    gridState: LazyListState = rememberLazyListState(),
    topOfMind: List<Card> = emptyList(),
    onLoadMoreTopOfMind: () -> Unit = {},
    onRefresh: () -> Unit = {},
    onRefreshResults: () -> Unit = {},
    onLongPressCard: ((Card) -> Unit)? = null,
    onSerendipity: () -> Unit = {},
    photoUrl: String? = null,
    mosaic: MosaicState = rememberMosaicState(),
    resultsState: LazyListState = rememberLazyListState(),
) {
    val colors = LibraryStyle.colors
    val photoLoader = if (photoUrl != null) rememberPublicAccountPhotoLoader() else null
    Box(modifier.fillMaxSize().background(colors.background)) {
        when {
            searching && searchResults != null -> PullRefresh(searchResults.refreshing, onRefreshResults) {
                LibraryMosaic(searchResults, onLoadMoreResults, onOpenCard, state = resultsState, mosaic = mosaic,
                    onLongPressCard = onLongPressCard, header = {
                        Text("Search", Modifier.padding(20.dp).semantics { heading() }, style = LibraryStyle.title, color = colors.text)
                        ResultChips(filter, suggestions, onChoose, onRemoveFilter)
                        if (searchResults.cards.isEmpty()) GalleryFeedNotice(searchResults, onLoadMoreResults, search = true)
                    })
            }
            searching -> Column(Modifier.fillMaxSize().padding(top = 20.dp)) {
                Text("Find something", Modifier.padding(horizontal = 20.dp, vertical = 12.dp).semantics { heading() },
                    style = LibraryStyle.title, color = colors.text)
                SearchChips(typeChips, onChoose, Modifier.padding(bottom = 150.dp))
            }
            else -> Column(Modifier.fillMaxSize()) {
                GalleryHeader(onOpenSettings, onAdd, onSerendipity, photoUrl = photoUrl, photoLoader = photoLoader)
                PullRefresh(feed.refreshing, onRefresh, Modifier.weight(1f).fillMaxWidth()) {
                    LibraryMosaic(feed, onLoadMore, onOpenCard, state = gridState, mosaic = mosaic,
                        onLongPressCard = onLongPressCard, header = {
                            GalleryHighlights(topOfMind, onOpenCard, onLoadMore = onLoadMoreTopOfMind, onLongPressCard = onLongPressCard)
                            if (feed.cards.isEmpty()) GalleryFeedNotice(feed, onRetry, search = false)
                        })
                }
            }
        }
    }
}

/** Loading, empty and failed first pages keep the profile controls available. */
@Composable
private fun GalleryFeedNotice(feed: FeedState, onRetry: () -> Unit, search: Boolean) {
    val colors = LibraryStyle.colors
    Column(Modifier.fillMaxWidth().padding(horizontal = 24.dp, vertical = 64.dp),
        horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(16.dp)) {
        when {
            feed.error != null -> {
                Text(feed.error, style = LibraryStyle.body, color = colors.muted)
                Text("Retry", Modifier.clip(CircleShape).clickable(role = Role.Button, onClick = onRetry).padding(16.dp),
                    style = LibraryStyle.body, color = colors.text)
            }
            feed.loading || !feed.endReached -> LoadingSpinner(color = colors.accent)
            else -> Text(if (search) "No cards found" else "Your mind starts here", style = LibraryStyle.body, color = colors.muted)
        }
    }
}

/** Search suggestions use the same neutral capsule treatment as the gallery navigation. */
@Composable
fun SearchChips(chips: List<Suggestion>, onSelect: (Suggestion) -> Unit, modifier: Modifier = Modifier) {
    ChipFlow(modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 20.dp)) {
        chips.forEach { chip -> SearchChip(chip.text, false) { onSelect(chip) } }
    }
}

@Composable
private fun ResultChips(filter: SearchFilter?, suggestions: List<Suggestion>, onChoose: (Suggestion) -> Unit, onRemoveFilter: () -> Unit) {
    if (filter == null && suggestions.isEmpty()) return
    ChipFlow(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 20.dp)) {
        filter?.let { SearchChip(it.label, true, onRemoveFilter) }
        suggestions.filter { it.queryText != filter?.queryText }.forEach { chip ->
            SearchChip(if (chip.type == "type") chip.text else chip.text.lowercase(), false) { onChoose(chip) }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ChipFlow(modifier: Modifier, content: @Composable () -> Unit) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) { content() }
}

@Composable
private fun SearchChip(label: String, chosen: Boolean, onClick: () -> Unit) {
    val colors = LibraryStyle.colors
    FootPressTheme {
    Box(Modifier.heightIn(min = 44.dp).clip(CircleShape).background(if (chosen) colors.selected else colors.raised)
        .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onClick)
        .padding(horizontal = 16.dp, vertical = 10.dp), contentAlignment = Alignment.Center) {
        Text(label, style = LibraryStyle.label, color = colors.text)
    }
    }
}

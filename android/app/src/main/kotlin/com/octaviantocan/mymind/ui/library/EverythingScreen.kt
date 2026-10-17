package com.octaviantocan.mymind.ui.library

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridScope
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.rememberLazyStaggeredGridState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.ui.errors.WebErrorScreen
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Distance from the top of the page to the first row under the search bar (10 + 55 + 11 dp). */
val UNDER_SEARCH_BAR = PAGE_GUTTER + SEARCH_BAR_HEIGHT + 11.dp

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

/**
 * The Everything tab: the search bar over the Top of Mind row and the card grid; with the search field focused, the
 * type chips or the search results instead.
 *
 * @param viewModel Library behind the tab.
 * @param onOpenCard Called with a card the user tapped.
 * @param onOpenSettings Called when the hamburger is tapped.
 * @param onAdd Called when the + button is tapped.
 * @param modifier Modifier for the page.
 */
@Composable
fun EverythingScreen(
    viewModel: LibraryViewModel,
    onOpenCard: (Card) -> Unit,
    onOpenSettings: () -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val feed by viewModel.everything.state.collectAsStateWithLifecycle()
    val query by viewModel.search.query.collectAsStateWithLifecycle()
    val filter by viewModel.search.filter.collectAsStateWithLifecycle()
    val types by viewModel.search.types.collectAsStateWithLifecycle()
    val suggestions by viewModel.search.suggestions.collectAsStateWithLifecycle()
    val results by viewModel.search.results.collectAsStateWithLifecycle()
    val resultState by results.state.collectAsStateWithLifecycle()
    var focused by remember { mutableStateOf(false) }
    val focusRequester = remember { FocusRequester() }
    val focusManager = LocalFocusManager.current
    val searching = focused || query.isNotEmpty() || filter != null
    LaunchedEffect(searching) { if (searching) viewModel.search.loadTypes() }
    val close = {
        viewModel.search.clear()
        focusManager.clearFocus()
        focused = false
    }
    BackHandler(enabled = searching, onBack = close)

    EverythingContent(
        feed = feed,
        searching = searching,
        searchResults = if (query.isNotBlank() || filter != null) resultState else null,
        typeChips = types.ifEmpty { SearchType.entries.map { it.suggestion } },
        filter = filter,
        suggestions = suggestions,
        query = query,
        focused = focused,
        focusRequester = focusRequester,
        onQueryChange = viewModel.search::setQuery,
        onFocusChange = { if (it) focused = true },
        onClear = close,
        onChoose = {
            viewModel.search.choose(it)
            focusManager.clearFocus()
        },
        onRemoveFilter = { viewModel.search.setFilter(null) },
        onLoadMore = viewModel.everything::loadMore,
        onLoadMoreResults = results::loadMore,
        onRetry = viewModel.everything::reload,
        onOpenCard = onOpenCard,
        onOpenSettings = onOpenSettings,
        onAdd = onAdd,
        modifier = modifier,
    )
}

/**
 * The Everything tab without a view model.
 *
 * @param feed The library's cards.
 * @param searching True while the search overlay is open.
 * @param searchResults Results of the current search, or null while nothing is searched.
 * @param typeChips Card types offered while the field is empty, most used first.
 * @param filter Filter the search is limited to, or null.
 * @param suggestions Tags that match the typed text, shown above the results.
 * @param query Text in the search field.
 * @param focused True while the search field has focus.
 * @param focusRequester Moves focus into the search field.
 * @param onQueryChange Called with new search text.
 * @param onFocusChange Called when the search field gains or loses focus.
 * @param onClear Called when the search is closed.
 * @param onChoose Called with a tapped type or tag chip.
 * @param onRemoveFilter Called when the filter chip is tapped.
 * @param onLoadMore Called when the grid nears its end.
 * @param onLoadMoreResults Called when the results near their end.
 * @param onRetry Called to read the library again after a failure.
 * @param onOpenCard Called with a tapped card.
 * @param onOpenSettings Called when the hamburger is tapped.
 * @param onAdd Called when the + button is tapped.
 * @param modifier Modifier for the page.
 * @param gridState Scroll position of the library grid.
 */
@Composable
fun EverythingContent(
    feed: FeedState,
    searching: Boolean,
    searchResults: FeedState?,
    typeChips: List<Suggestion>,
    filter: SearchFilter?,
    suggestions: List<Suggestion>,
    query: String,
    focused: Boolean,
    focusRequester: FocusRequester,
    onQueryChange: (String) -> Unit,
    onFocusChange: (Boolean) -> Unit,
    onClear: () -> Unit,
    onChoose: (Suggestion) -> Unit,
    onRemoveFilter: () -> Unit,
    onLoadMore: () -> Unit,
    onLoadMoreResults: () -> Unit,
    onRetry: () -> Unit,
    onOpenCard: (Card) -> Unit,
    onOpenSettings: () -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
    gridState: LazyStaggeredGridState = rememberLazyStaggeredGridState(),
) {
    val page = MymindTheme.page
    Box(modifier.fillMaxSize().background(page.background)) {
        when {
            searching && searchResults != null -> CardGrid(searchResults, onLoadMoreResults, onOpenCard, top = UNDER_SEARCH_BAR) {
                resultChips(filter, suggestions, onChoose, onRemoveFilter)
            }
            searching -> SearchChips(typeChips, onChoose, Modifier.padding(top = UNDER_SEARCH_BAR))
            feed.cards.isEmpty() && feed.error != null -> WebErrorScreen(onRetry = onRetry)
            feed.empty -> PageSpinner()
            else -> {
                val pinned = feed.cards.filter { it.pinned }.sortedBy { it.pinPosition ?: Long.MAX_VALUE }
                CardGrid(feed, onLoadMore, onOpenCard, state = gridState, top = UNDER_SEARCH_BAR) { topOfMindHeader(pinned, onOpenCard) }
            }
        }
        Box(Modifier.fillMaxWidth().height(PAGE_GUTTER).background(page.background))
        SearchBar(
            query = query,
            focused = searching,
            onQueryChange = onQueryChange,
            onFocusChange = onFocusChange,
            onClear = onClear,
            onOpenSettings = onOpenSettings,
            onAdd = onAdd,
            focusRequester = focusRequester,
        )
    }
}

/**
 * The type chips under the focused search bar.
 *
 * @param chips Type suggestions, most used first.
 * @param onSelect Called with the tapped chip.
 * @param modifier Modifier for the chip area.
 */
@Composable
fun SearchChips(chips: List<Suggestion>, onSelect: (Suggestion) -> Unit, modifier: Modifier = Modifier) {
    ChipFlow(
        modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = PAGE_GUTTER),
    ) {
        chips.forEach { chip -> SearchChip(chip.text, chosen = false) { onSelect(chip) } }
    }
}

/** The filter chip, in the accent colour, and the tag suggestions over the search results. */
private fun LazyStaggeredGridScope.resultChips(
    filter: SearchFilter?,
    suggestions: List<Suggestion>,
    onChoose: (Suggestion) -> Unit,
    onRemoveFilter: () -> Unit,
) {
    if (filter == null && suggestions.isEmpty()) return
    item(key = "search-chips", span = StaggeredGridItemSpan.FullLine) {
        ChipFlow(Modifier.fillMaxWidth()) {
            filter?.let { SearchChip(it.label, chosen = true, onClick = onRemoveFilter) }
            suggestions.filter { it.queryText != filter?.queryText }.forEach { chip ->
                SearchChip(if (chip.type == "type") chip.text else chip.text.lowercase(), chosen = false) { onChoose(chip) }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ChipFlow(modifier: Modifier, content: @Composable () -> Unit) {
    FlowRow(
        modifier,
        horizontalArrangement = Arrangement.spacedBy(PAGE_GUTTER),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) { content() }
}

/** One 32.7 dp search chip; [chosen] draws it in the accent colour. */
@Composable
private fun SearchChip(label: String, chosen: Boolean, onClick: () -> Unit) {
    val page = MymindTheme.page
    val ink = if (MymindTheme.colors.dark) Color(0xFF8B94A1) else page.muted
    Box(
        Modifier
            .height(32.7.dp)
            .clip(RoundedCornerShape(50))
            .background(if (chosen) MymindTheme.colors.accent else page.surface)
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            label,
            style = TextStyle(color = if (chosen) Color.White else ink, fontSize = 16.sp, fontFamily = MymindTheme.fonts.nunito),
            maxLines = 1,
        )
    }
}

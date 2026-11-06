package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.ui.theme.themeBackground
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.ImageLoader
import com.octaviantocan.mymind.data.AiSpace
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.shell.FootPressTheme
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.settings.glassSource
import com.octaviantocan.mymind.ui.settings.rememberGlassBackdrop
import com.octaviantocan.mymind.ui.sky.BackTodayBand
import com.octaviantocan.mymind.ui.sky.BackTodayState
import com.octaviantocan.mymind.ui.sky.SkyLayer
import com.octaviantocan.mymind.ui.sky.rememberSkyOpening
import com.octaviantocan.mymind.ui.sky.rememberStarFlight
import com.octaviantocan.mymind.ui.sky.starFlights
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

/**
 * The library gallery; its search field lives in the shell's single floating bar.
 *
 * @param openedLately Cards opened lately, latest first, which the opened search page offers again.
 * @param cardOpen True while a card is open over the page, which then covers the sky.
 * @param choice Where holding a find and sliding gathers finds, in the gallery and in search results; null leaves holding
 * to open the card menu.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun EverythingScreen(
    viewModel: LibraryViewModel,
    onOpenCard: (Card) -> Unit,
    onOpenSettings: () -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
    onLongPressCard: ((Card) -> Unit)? = null,
    searching: Boolean = false,
    photoUrl: String? = null,
    onOpenSpace: (AiSpace) -> Unit = {},
    photoLoader: ImageLoader? = null,
    openedLately: List<Card> = emptyList(),
    cardOpen: Boolean = false,
    choice: CardChoice? = null,
) {
    val feed by viewModel.everything.state.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.state.collectAsStateWithLifecycle()
    val backToday by viewModel.backToday.state.collectAsStateWithLifecycle()
    val query by viewModel.search.query.collectAsStateWithLifecycle()
    val filter by viewModel.search.filter.collectAsStateWithLifecycle()
    val types by viewModel.search.types.collectAsStateWithLifecycle()
    val suggestions by viewModel.search.suggestions.collectAsStateWithLifecycle()
    val results by viewModel.search.results.collectAsStateWithLifecycle()
    val resultState by results.state.collectAsStateWithLifecycle()
    val rank by viewModel.search.rank.collectAsStateWithLifecycle()
    val engine by viewModel.search.engine.collectAsStateWithLifecycle()
    val onlyRanked by viewModel.search.onlyRanked.collectAsStateWithLifecycle()
    val recentSearches by viewModel.search.recentSearches.collectAsStateWithLifecycle()
    val tagChoices by viewModel.tagChoices.collectAsStateWithLifecycle()
    val focusManager = LocalFocusManager.current
    val resultsScroll = rememberLazyListState()
    LaunchedEffect(searching) {
        if (!searching) return@LaunchedEffect
        viewModel.search.loadTypes()
        viewModel.search.loadEngine()
        viewModel.loadTagChoices()
    }
    LaunchedEffect(query, filter) { resultsScroll.scrollToItem(0) }
    EverythingContent(
        feed, searching, if (query.isNotBlank() || filter != null) resultState else null,
        types.ifEmpty { SearchType.entries.map { it.suggestion } }, filter, suggestions,
        onChoose = { viewModel.search.choose(it); focusManager.clearFocus() },
        onRemoveFilter = { viewModel.search.setFilter(null) },
        onLoadMore = viewModel.everything::loadMore, onLoadMoreResults = results::loadMore,
        onRetry = viewModel.everything::reload, onOpenCard = onOpenCard,
        onOpenSettings = onOpenSettings, onAdd = onAdd, modifier = modifier,
        pinned = pinned.cards, onLoadMorePinned = viewModel.pinned::loadMore,
        backToday = backToday, onUncover = viewModel.backToday::uncover,
        onSkyOpened = { viewModel.backToday.load(); viewModel.backToday.loadFinds() }, cardOpen = cardOpen,
        onOpenStar = { id ->
            viewModel.backToday.state.value.finds[id]?.let { card ->
                viewModel.backToday.uncover(card)
                onOpenCard(card)
            }
        },
        onRefresh = viewModel::refreshEverything, onRefreshResults = results::refresh,
        onLongPressCard = onLongPressCard, choice = choice,
        photoUrl = photoUrl, photoLoader = photoLoader, resultsState = resultsScroll, ai = rank,
        onAiTag = { tag -> viewModel.search.choose(Suggestion("tag", tag, "tag:$tag")); focusManager.clearFocus() },
        onOpenSpace = { focusManager.clearFocus(); onOpenSpace(it) },
        dock = SearchDockState(query, WindowInsets.isImeVisible, engine, recentSearches,
            tagChoices.recent.ifEmpty { tagChoices.all }.take(DOCKED_TAGS), openedLately, onlyRanked),
        dockActions = remember(viewModel, focusManager) {
            SearchDockActions(
                onAsk = { focusManager.clearFocus(); viewModel.search.submit() },
                onSearchFor = { focusManager.clearFocus(); viewModel.search.searchFor(it) },
                onForgetSearches = viewModel.search::forgetSearches,
                onOnlyRanked = viewModel.search::showOnlyRanked,
            )
        },
    )
}

/** How many of the person's tags the opened search page offers. */
private const val DOCKED_TAGS = 12

/** Height of the fixed controls' row over the top of a library page. */
val GALLERY_CONTROLS_HEIGHT = 72.dp

/** Pixels the list has scrolled past its top, saturating once the first item has left. */
internal fun LazyListState.scrolledBy(): Float =
    if (firstVisibleItemIndex > 0) Float.MAX_VALUE else firstVisibleItemScrollOffset.toFloat()

/** The title scrolls with the pin rail and cards under fixed controls; search and browsing retain independent positions. */
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
    pinned: List<Card> = emptyList(),
    onLoadMorePinned: () -> Unit = {},
    backToday: BackTodayState = BackTodayState(),
    onUncover: (Card) -> Unit = {},
    onSkyOpened: () -> Unit = {},
    onOpenStar: (String) -> Unit = {},
    cardOpen: Boolean = false,
    onRefresh: () -> Unit = {},
    onRefreshResults: () -> Unit = {},
    onLongPressCard: ((Card) -> Unit)? = null,
    choice: CardChoice? = null,
    photoUrl: String? = null,
    mosaic: MosaicState = rememberMosaicState(reels = true),
    resultsState: LazyListState = rememberLazyListState(),
    ai: AiRank? = null,
    onAiTag: (String) -> Unit = {},
    onOpenSpace: (AiSpace) -> Unit = {},
    photoLoader: ImageLoader? = null,
    dock: SearchDockState = SearchDockState(),
    dockActions: SearchDockActions = SearchDockActions(),
) {
    val colors = LibraryStyle.colors
    val accountPhotoLoader = photoLoader ?: if (photoUrl != null) rememberPublicAccountPhotoLoader() else null
    val backdrop = rememberGlassBackdrop(rememberUpdatedState(1f))
    val resultsBackdrop = rememberGlassBackdrop(rememberUpdatedState(1f))
    val flight = rememberStarFlight()
    val sky = rememberSkyOpening()
    Box(modifier.fillMaxSize().themeBackground(colors.background, backdrop = false)) {
        when {
            searching && searchResults != null -> {
                PullRefresh(searchResults.refreshing, onRefreshResults, Modifier.fillMaxSize().glassSource(resultsBackdrop)) {
                    LibraryMosaic(searchResults, onLoadMoreResults, onOpenCard, state = resultsState, mosaic = mosaic,
                        onLongPressCard = onLongPressCard, choice = choice, header = {
                            ResultsTitle(filter.takeIf { dock.query.isBlank() }, searchResults)
                            // With words typed and nothing found, the dock above the bar says what to try instead.
                            val docked = dock.query.isNotBlank() && searchResults.endReached && searchResults.error == null
                            if (searchResults.cards.isEmpty() && !docked) GalleryFeedNotice(searchResults, onLoadMoreResults, search = true)
                        })
                }
                GalleryScrollEdge(resultsBackdrop, colors.background, Modifier.align(Alignment.TopCenter).graphicsLayer { alpha = 1f - mosaic.reel },
                    visible = { resultsState.scrolledBy() })
                SearchResultsDock(searchResults, filter, suggestions, ai, dock, dockActions, resultsState,
                    onChoose, onRemoveFilter, onAiTag, onOpenSpace, onOpenCard)
            }
            searching -> SearchStart(typeChips, dock, dockActions, onChoose, onOpenCard, onLongPressCard)
            else -> Box(Modifier.fillMaxSize()) {
                PullRefresh(feed.refreshing, onRefresh, Modifier.fillMaxSize().glassSource(backdrop), top = GALLERY_CONTROLS_HEIGHT) {
                    LibraryMosaic(feed, onLoadMore, onOpenCard, state = gridState, mosaic = mosaic,
                        onLongPressCard = onLongPressCard, choice = choice, header = {
                            GalleryTitle("Your inventory")
                            GalleryHighlights(pinned, onOpenCard, onLoadMore = onLoadMorePinned, onLongPressCard = onLongPressCard)
                            BackTodayBand(backToday, { card, at -> onUncover(card); flight.launch(at) }, onOpenCard, onLongPressCard = onLongPressCard)
                            if (feed.cards.isEmpty()) GalleryFeedNotice(feed, onRetry, search = false)
                        })
                }
                GalleryScrollEdge(backdrop, colors.background, Modifier.align(Alignment.TopCenter).graphicsLayer { alpha = 1f - mosaic.reel },
                    visible = { gridState.scrolledBy() })
                GalleryControls(onOpenSettings, onAdd, Modifier.align(Alignment.TopEnd).awayInReels(mosaic), photoUrl = photoUrl, photoLoader = accountPhotoLoader,
                    onSky = if (backToday.hasSky) ({ sky.open(); onSkyOpened() }) else null, flight = flight)
                Box(Modifier.fillMaxSize().starFlights(flight))
                backToday.sky?.let { SkyLayer(sky, it, backToday.finds, onOpenStar, covered = cardOpen) }
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
            else -> Text(if (search) "No cards found" else "Your inventory starts here", style = LibraryStyle.body, color = colors.muted)
        }
    }
}

/** The results' title: the filter's name and how many cards it holds once all are read, or Search for typed words. */
@Composable
private fun ResultsTitle(filter: SearchFilter?, results: FeedState) {
    val colors = LibraryStyle.colors
    Column(Modifier.fillMaxWidth().padding(20.dp)) {
        Text(filter?.label ?: "Search", Modifier.semantics { heading() }, style = LibraryStyle.title, color = colors.text)
        if (filter != null && results.endReached && results.error == null && results.cards.isNotEmpty()) {
            Text(cardCount(results.cards.size), style = LibraryStyle.body, color = colors.muted)
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun ChipFlow(modifier: Modifier, content: @Composable () -> Unit) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) { content() }
}

@Composable
internal fun SearchChip(label: String, chosen: Boolean, onRaised: Boolean = false, onClick: () -> Unit) {
    val colors = LibraryStyle.colors
    FootPressTheme {
    // On a raised panel the chip takes the page's colour so it still stands out.
    Box(Modifier.heightIn(min = 44.dp).clip(CircleShape).background(if (chosen) colors.selected else if (onRaised) colors.background else colors.raised)
        .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onClick)
        .padding(horizontal = 16.dp, vertical = 10.dp), contentAlignment = Alignment.Center) {
        Text(label, style = LibraryStyle.label, color = colors.text)
    }
    }
}

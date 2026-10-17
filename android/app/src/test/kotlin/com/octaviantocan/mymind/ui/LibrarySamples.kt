package com.octaviantocan.mymind.ui

import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.ui.card.CardHeaderIcons
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.SearchFilter
import com.octaviantocan.mymind.ui.library.SearchType
import com.octaviantocan.mymind.ui.library.SpacesState
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.menu.CardMenuIcons

/** Made-up cards of every tile kind; pictures are blobs so nothing is fetched. */
object LibrarySamples {
    private fun picture(id: String, type: String, w: Int, h: Int, color: String, title: String?, flags: Int = 0, pin: Long? = null) =
        testCard(id, type = type, flags = flags, pinPosition = pin, title = title)
            .copy(blob = Blob(type = "image/png", width = w, height = h, dominantColor = color))

    private const val PINNED = 512

    val topOfMind = listOf(
        picture("t1", "WebPage", 1200, 900, "#e9edf2", "Pinned page", PINNED, 1),
        testCard("t2", type = "Article", flags = PINNED, pinPosition = 2, title = "A pinned article about layouts"),
        picture("t3", "WebPage", 1200, 630, "#f5f5f5", "Another pinned page", PINNED, 3),
    )

    val grid = listOf(
        picture("g1", "WebPage", 1280, 960, "#f2f2f2", "A saved web page with a long title"),
        testCard("g2", type = "XPost", title = "A post").copy(
            post = Post(bodyHtml = "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p><p>Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>", author = "Someone"),
        ),
        testCard("g3", type = "Article", flags = 1 shl 1, title = "An article with a two line headline")
            .copy(cover = Cover(width = 1200, height = 630, dominantColor = "#e8e2d6")),
        testCard("g4", type = "YouTubeVideo", title = "A video").copy(cover = Cover(width = 1280, height = 720, dominantColor = "#c9a27e")),
        testCard("g5", type = "Note", title = null).copy(note = "A note the user wrote.\n\nIt has two paragraphs of plain text that wrap over a few lines."),
        testCard("g6", type = "Repository", title = "owner/project").copy(siteName = "GitHub"),
        picture("g7", "Image", 1000, 1300, "#8a6d5a", "A picture"),
        picture("g8", "WebPage", 1200, 800, "#dfe6ee", "One more page"),
        testCard("g9", type = "XPost", title = "Another post").copy(post = Post(bodyHtml = "<p>Short post text.</p>", author = "Someone else")),
        picture("g10", "Image", 900, 900, "#55606e", "A square picture"),
    )

    val feed = FeedState(cards = topOfMind + grid)

    val spaces = SpacesState(
        spaces = listOf(
            Space("s1", "Reading", "#ff5924"),
            Space("s2", "Design", "#4080df"),
            Space("s3", "Recipes", "#2dba4e"),
            Space("s4", "Travel", "#b04fd8"),
        ),
        covers = mapOf(
            "s1" to grid[0],
            "s2" to grid[6],
            "s3" to grid[9],
            "s4" to grid[3],
        ),
    )

    val article: Card = testCard("a1", type = "Article", url = "https://example.com/a1", title = "An article opened in the card view")
        .copy(description = (1..12).joinToString(" ") { "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor." })

    val note: Card = testCard("n1", type = "Note", title = null).copy(note = (1..10).joinToString("\n\n") { "A paragraph of a note, line $it." })

    val video: Card = testCard("v1", type = "YouTubeVideo", url = "https://example.com/v1", title = "A video card")
        .copy(cover = Cover(width = 1280, height = 720, dominantColor = "#3d4a5c"))
}

/**
 * The Everything page filled with [LibrarySamples], with every callback doing nothing.
 *
 * @param feed The library's cards.
 * @param searching True while the search overlay is open.
 * @param results Search results, or null.
 * @param type Type chip the search is limited to.
 * @param filter Filter chip of the results; by default the type's.
 * @param suggestions Tag suggestions above the results.
 * @param query Text in the search field.
 * @param grid Scroll position of the grid; null keeps one of its own.
 */
@Composable
fun SampleEverything(
    feed: FeedState = LibrarySamples.feed,
    searching: Boolean = false,
    results: FeedState? = null,
    type: SearchType? = null,
    filter: SearchFilter? = type?.let { SearchFilter(it.label, "type:${it.slug}") },
    suggestions: List<Suggestion> = emptyList(),
    query: String = "",
    grid: LazyStaggeredGridState? = null,
) {
    val focus = remember { FocusRequester() }
    EverythingContent(
        feed = feed,
        searching = searching,
        searchResults = results,
        typeChips = SearchType.entries.map { it.suggestion },
        filter = filter,
        suggestions = suggestions,
        query = query,
        focused = searching,
        focusRequester = focus,
        onQueryChange = {},
        onFocusChange = {},
        onClear = {},
        onChoose = {},
        onRemoveFilter = {},
        onLoadMore = {},
        onLoadMoreResults = {},
        onRetry = {},
        onOpenCard = {},
        onOpenSettings = {},
        onAdd = {},
        gridState = grid ?: remember { LazyStaggeredGridState() },
    )
}

private val Grey = Color(0xFF8B99AF)

/** Menu icons as flat blocks with the shipped drawables' sizes in dp. */
fun flatCardMenuIcons() = CardMenuIcons(
    icon = { BlockPainter(if (it.danger) Color(0xFFFF002E) else Grey, Size(21f * 2.8125f, 21f * 2.8125f)) },
    lock = BlockPainter(Grey, Size(24f * 2.8125f, 24f * 2.8125f)),
)

/** Card header icons as flat blocks with the shipped drawables' sizes in dp (caret 16 x 11, dots 16 x 4). */
fun flatSizedCardHeaderIcons() = CardHeaderIcons(
    close = BlockPainter(Grey, Size(16f * 2.8125f, 11f * 2.8125f)),
    menu = BlockPainter(Grey, Size(16f * 2.8125f, 4f * 2.8125f)),
)

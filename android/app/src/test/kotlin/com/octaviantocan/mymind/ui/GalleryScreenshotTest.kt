package com.octaviantocan.mymind.ui

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.remember
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipe
import androidx.compose.ui.text.TextLayoutResult
import androidx.test.core.app.ApplicationProvider
import coil3.decode.DataSource
import coil3.EventListener
import coil3.ImageLoader
import coil3.SingletonImageLoader
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.AiSpace
import com.octaviantocan.mymind.data.AiUnderstood
import com.octaviantocan.mymind.data.AppInfo
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.GalleryImage
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.data.Product
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.library.AiRank
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.MosaicState
import com.octaviantocan.mymind.ui.library.steppedPosition
import com.octaviantocan.mymind.ui.library.SearchDockActions
import com.octaviantocan.mymind.ui.library.SearchDockState
import com.octaviantocan.mymind.ui.library.SearchFilter
import com.octaviantocan.mymind.ui.library.SearchType
import com.octaviantocan.mymind.ui.library.sourceLabel
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.shell.LibraryFootBar
import com.octaviantocan.mymind.ui.shell.SEARCH_HINT
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.util.Collections
import java.util.concurrent.ConcurrentHashMap
import kotlin.math.pow

/** Actual Compose gallery and Coil photo decoding, with public fixture bytes and no network requests. */
@OptIn(coil3.annotation.DelicateCoilApi::class)
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class GalleryScreenshotTest(private val dark: Boolean) {
    @get:Rule val compose = createComposeRule()
    private val loaded = Collections.synchronizedSet(mutableSetOf<String>())
    private val requests = ConcurrentHashMap<String, Int>()
    private val fetches = ConcurrentHashMap<String, Int>()
    private val decodes = ConcurrentHashMap<String, Int>()
    private lateinit var loader: ImageLoader
    private lateinit var mosaicState: MosaicState
    private lateinit var client: OkHttpClient

    @Before fun photos() {
        val pictures = listOf("mountains", "forest", "coast", "hills").associateWith { name ->
            checkNotNull(javaClass.getResourceAsStream("/gallery/$name.jpg")).use { it.readBytes() }
        }
        client = OkHttpClient.Builder().addInterceptor { chain ->
            val request = chain.request()
            check(request.url.host == "gallery.example.test")
            fetches.merge(request.url.toString(), 1, Int::plus)
            check(request.header("Authorization") == null)
            val bytes = pictures.getValue(request.url.encodedPath.removePrefix("/").removeSuffix(".jpg"))
            Response.Builder().request(request).protocol(Protocol.HTTP_1_1).code(200).message("Fixture")
                .body(bytes.toResponseBody("image/jpeg".toMediaType())).build()
        }.build()
        loader = ImageLoader.Builder(ApplicationProvider.getApplicationContext())
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { client })) }
            .eventListener(object : EventListener() {
                override fun onStart(request: ImageRequest) { requests.merge(request.data.toString(), 1, Int::plus) }
                override fun onSuccess(request: ImageRequest, result: SuccessResult) {
                    if (result.dataSource != DataSource.MEMORY_CACHE) decodes.merge(request.data.toString(), 1, Int::plus)
                    loaded += request.data.toString()
                }
            }).build()
        SingletonImageLoader.setUnsafe(loader)
    }

    @After fun close() {
        SingletonImageLoader.reset()
        loader.shutdown()
        client.connectionPool.evictAll()
        client.dispatcher.executorService.shutdown()
    }

    private fun photo(id: String, picture: String, width: Int, height: Int, title: String) =
        testCard(id, type = "Image", title = title).copy(cover = Cover("https://gallery.example.test/$picture.jpg", width, height, dominantColor = "#4c6254"))

    private val pins: List<Card> get() = listOf(
        photo("pin-landscape", "mountains", 1200, 900, "Places to return to").copy(flags = 512, pinPosition = 1),
        photo("pin-coast", "coast", 900, 1200, "Along the coast").copy(flags = 512, pinPosition = 2),
    )

    private val cards: List<Card> get() = listOf(
        photo("forest", "forest", 800, 1200, "A quiet trail"),
        photo("mountains", "mountains", 1200, 800, "Mountain light"),
        testCard("note", type = "Note", title = "A thought to keep", note = "Leave a little room for things you haven't found yet."),
        photo("coast", "coast", 1200, 900, "On the water"),
        photo("article", "hills", 1200, 800, "Small spaces, better ideas").copy(type = "Article"),
        testCard("post", type = "XPost", title = "Designing for attention").copy(post = Post(bodyHtml = "The best tools give you space to think. Save what moves you, and find it again when it matters.")),
        photo("forest-again", "forest", 1000, 900, "Somewhere green"),
        testCard("reading", type = "Article", title = "Good tools should get out of your way"),
        photo("hills", "hills", 1200, 800, "Evening hills"),
        photo("coast-again", "coast", 900, 1200, "Ocean blue"),
        photo("mountains-again", "mountains", 1100, 900, "A place worth saving"),
    )

    private val mixedNotes: List<Card> get() = listOf(
        testCard("thought", type = "Note", title = "A thought to keep", note = "Leave a little room for things you haven't found yet."),
        photo("walk", "forest", 800, 1200, "Weekend walk").copy(type = "Note", note = "Take the long way home. There is always something worth noticing."),
        photo("reading", "hills", 1200, 800, "Small spaces, better ideas").copy(type = "Article"),
        testCard("post-picture", type = "XPost", title = "Designing for attention").copy(post = Post(
            bodyHtml = "<p>The best tools leave you room to think. Keep the words and the picture together.</p>",
            gallery = listOf(GalleryImage("https://gallery.example.test/coast.jpg", 1200, 900)),
        )),
        photo("saved-link", "hills", 1200, 800, "Collect the things that move you").copy(type = "WebPage", domain = "reading.example"),
        testCard("quote", type = "Quote", title = "An idea to keep", note = "Make room for things that make you curious."),
        photo("mountain-note", "mountains", 1200, 800, "Back to the mountains").copy(type = "Content", note = "A slower morning. A clearer mind. Save this for the next trip."),
        testCard("repository", type = "Repository", title = "tools-for-thought").copy(domain = "github.com"),
        photo("image-only", "forest", 800, 1200, "Somewhere green"),
        photo("annotated-photo", "mountains", 1200, 800, "Morning light").copy(note = "A small reminder: take a breath before rushing into the day."),
        photo("saved-video", "coast", 1200, 900, "A saved film").copy(type = "YouTubeVideo", note = "A short film worth watching again."),
        testCard("snippet", type = "Content", title = "A useful reminder", note = "Good tools should get out of your way."),
        testCard("long-note", type = "Note", title = "Landing pages", note = "Send your landing page to five people who have never heard of you " +
            "and ask them what you sell. If they cannot say it in one sentence, the page is not done. Then cut every line that " +
            "does not help them say it. Keep the picture that shows the thing working, drop the ones that only look nice."),
        testCard("post-media-link", type = "XPost", title = "A saved post").copy(post = Post(
            bodyHtml = "<p>\uD83C\uDF2D pic.twitter.com/mHuTM2eiQ1</p>", author = "Ana Lima (@ana)",
            gallery = listOf(GalleryImage("https://gallery.example.test/forest.jpg", 1000, 900)),
        )),
    )

    /** One of each kind of print (look 07 Prints, Octavian, 1 November 2026): sources under words, pins, apps, videos and a PDF. */
    private val kinds: List<Card> get() = listOf(
        testCard("quote", type = "Quote", note = "In every walk with nature one receives far more than he seeks.").copy(author = "John Muir"),
        photo("pin", "hills", 900, 1200, "Speckled stoneware mug").copy(type = "Product", siteName = "Hearth & Clay", product = Product(price = "GBP 24.00")),
        testCard("link", type = "WebPage", title = "Ten walks from the end of a tube line").copy(siteName = "Outwalks"),
        photo("film", "coast", 1280, 720, "Reading a river: where trout hold in fast water").copy(type = "YouTubeVideo", author = "by Upstream Notes", videoLength = 768),
        photo("described", "mountains", 900, 1300, "Small spaces, better ideas").copy(type = "Article", siteName = "Workroom Weekly",
            description = "Why the best workshops are a little too small, and what that does to the work made in them."),
        photo("app", "forest", 512, 512, "Trailhead: offline maps").copy(type = "SoftwareApplication", app = AppInfo(distributor = "play-store")),
        photo("pdf", "hills", 642, 840, "The Ridgeway, a walker's guide").copy(type = "Document"),
        testCard("soup", type = "Note", note = "Call Mum about the 14th, and ask how she makes the barley soup."),
        photo("street", "forest", 800, 1200, "A street I never turned into"),
    )

    private fun show(columns: Int, searching: Boolean = false,
        sample: List<Card> = cards, highlights: List<Card> = pins, reels: Boolean = false, pictures: Int = 3) {
        mosaicState = MosaicState(columns, reels)
        compose.setPhoneContent(dark) {
            ShellFrame(Tab.Everything, {}, PageScheme.System, footer = {
                LibraryFootBar(Tab.Everything, {}, searching, "", {}, {}, {})
            }) {
                EverythingContent(FeedState(sample, endReached = true), searching, null,
                    SearchType.entries.map { it.suggestion }, null, emptyList(), {}, {}, {}, {}, {}, {}, {}, {},
                    pinned = highlights, gridState = remember { LazyListState() }, mosaic = remember { mosaicState })
            }
        }
        compose.mainClock.autoAdvance = true
        if (!searching) compose.waitUntil(10_000) { loaded.size >= pictures }
        compose.mainClock.advanceTimeBy(1200)
        compose.waitForIdle()
    }

    private fun capture(columns: Int, name: String, searching: Boolean = false,
        sample: List<Card> = cards, highlights: List<Card> = pins) {
        show(columns, searching, sample, highlights)
        compose.onRoot().captureRoboImage("screenshots/${name}_${if (dark) "dark" else "light"}.png")
    }

    @Test fun gallery() {
        show(2)
        val account = compose.onNodeWithContentDescription("Account settings").fetchSemanticsNode().boundsInRoot
        val rail = compose.onNodeWithContentDescription("Pinned cards").fetchSemanticsNode().boundsInRoot
        val highlight = compose.onNodeWithContentDescription("Places to return to").fetchSemanticsNode().boundsInRoot
        assertTrue("Pinned cards must sit below the fixed account controls", rail.top >= account.bottom)
        assertTrue("Highlights must remain inside the rail", highlight.top >= rail.top)
        compose.onRoot().captureRoboImage("screenshots/gallery_${if (dark) "dark" else "light"}.png")
    }
    @Test fun compactGallery() = capture(4, "gallery_compact")
    @Test fun scrolledGalleryTopBlur() {
        show(2)
        val profile = compose.onNodeWithContentDescription("Account settings").fetchSemanticsNode().boundsInRoot
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            swipe(Offset(centerX, height * 0.82f), Offset(centerX, height * 0.18f), durationMillis = 700)
        }
        compose.waitForIdle()
        assertEquals(profile, compose.onNodeWithContentDescription("Account settings").fetchSemanticsNode().boundsInRoot)
        compose.onRoot().captureRoboImage("screenshots/gallery_scrolled_top_blur_${if (dark) "dark" else "light"}.png")
    }
    @Test fun search() = capture(2, "gallery_search", searching = true)

    @Test fun reels() {
        show(2, reels = true)
        val screen = compose.onRoot().fetchSemanticsNode().boundsInRoot
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            val y = height * 0.45f
            down(0, Offset(centerX - width * 0.2f, y))
            down(1, Offset(centerX + width * 0.2f, y))
            moveTo(0, Offset(centerX - width * 0.26f, y), delayMillis = 120)
            moveTo(1, Offset(centerX + width * 0.26f, y), delayMillis = 120)
            up(0)
            up(1)
        }
        compose.waitForIdle()
        assertEquals(1, mosaicState.columns)
        val bar = compose.onNodeWithContentDescription("Search").fetchSemanticsNode().boundsInRoot
        assertTrue("The bar slides down out of sight: $bar", bar.top >= screen.bottom - 1f)
        val account = compose.onNodeWithContentDescription("Account settings").fetchSemanticsNode().boundsInRoot
        assertTrue("The top controls slide up out of sight: $account", account.bottom <= screen.top + 1f)
        compose.onRoot().captureRoboImage("screenshots/gallery_reels_${if (dark) "dark" else "light"}.png")
    }

    @Test fun reelPages() {
        val tags = listOf(Tag("t1", "ideas"), Tag("t2", "design"), Tag("t3", "slow living"))
        show(1, sample = mixedNotes.map { it.copy(tags = tags, siteName = it.siteName ?: "reading.example".takeIf { _ -> it.type == "Article" }) },
            reels = true, pictures = 1)
        repeat(8) { page ->
            compose.onRoot().captureRoboImage("screenshots/gallery_reel_page_${page}_${if (dark) "dark" else "light"}.png")
            compose.onNodeWithContentDescription("Card gallery").performTouchInput {
                swipe(Offset(centerX, height * 0.7f), Offset(centerX, height * 0.35f), durationMillis = 200)
            }
            compose.waitForIdle()
        }
    }

    private fun showSearch(
        query: String,
        results: FeedState?,
        filter: SearchFilter? = null,
        suggestions: List<Suggestion> = emptyList(),
        ai: AiRank? = null,
        dock: SearchDockState = SearchDockState(query = query),
        actions: SearchDockActions = SearchDockActions(),
        onRemoveFilter: () -> Unit = {},
    ) {
        mosaicState = MosaicState(2)
        compose.setPhoneContent(dark) {
            ShellFrame(Tab.Everything, {}, PageScheme.System, footer = {
                LibraryFootBar(Tab.Everything, {}, true, query, {}, {}, {}, hint = filter?.let { "Search in ${it.label}" } ?: SEARCH_HINT)
            }) {
                EverythingContent(FeedState(cards, endReached = true), true, results,
                    SearchType.entries.map { it.suggestion }, filter, suggestions, {}, onRemoveFilter, {}, {}, {}, {}, {}, {},
                    mosaic = remember { mosaicState }, ai = ai, dock = dock, dockActions = actions)
            }
        }
        compose.mainClock.autoAdvance = true
        compose.mainClock.advanceTimeBy(1200)
        compose.waitForIdle()
    }

    private fun snap(name: String) = compose.onRoot().captureRoboImage("screenshots/${name}_${if (dark) "dark" else "light"}.png")

    // Version 10 of the search pages, "Near your thumb" (Octavian, 5 November 2026: "Let's build 10").
    @Test fun searchOpensWithRecentSearchesAndTagsNearTheThumb() {
        val searched = mutableListOf<String>()
        var forgot = false
        showSearch("", null, dock = SearchDockState(recentSearches = listOf("foggy paintings", "reading list", "thoreau"),
            tags = listOf("walking", "design", "slow living", "recipes"), openedLately = cards.take(4)),
            actions = SearchDockActions(onSearchFor = { searched += it }, onForgetSearches = { forgot = true }))
        compose.onNodeWithText("Find something").assertExists()
        compose.onNodeWithContentDescription("Opened lately").assertExists()
        compose.onNodeWithText("Your tags").assertExists()
        snap("gallery_search_start")
        compose.onNodeWithText("thoreau").performClick()
        assertEquals(listOf("thoreau"), searched)
        compose.onNodeWithContentDescription("Clear recent searches").performClick()
        assertTrue(forgot)
    }

    @Test fun typingOffersATagACardAndTheEngine() {
        var asked = 0
        showSearch("walk", FeedState(cards, endReached = true), suggestions = listOf(Suggestion("tag", "walking", "#walking")),
            dock = SearchDockState(query = "walk", typing = true, engine = "Claude Code"), actions = SearchDockActions(onAsk = { asked++ }))
        compose.onNodeWithText("walking").assertExists()
        compose.onNodeWithText("Tag").assertExists()
        snap("gallery_search_typing")
        compose.onNodeWithText("Ask Claude Code about “walk”").performClick()
        assertEquals(1, asked)
    }

    @Test fun aTagFilterRidesAboveTheBar() {
        var removed = false
        showSearch("", FeedState(cards, endReached = true), filter = SearchFilter("walking", "tag:walking"), onRemoveFilter = { removed = true })
        compose.onNodeWithText("11 cards").assertExists()
        compose.onNodeWithText("Search in walking").assertExists()
        snap("gallery_search_tag")
        compose.onNodeWithContentDescription("Remove the walking filter").performClick()
        assertTrue(removed)
    }

    @Test fun theEngineAtWorkSaysItsStepAboveTheBar() {
        showSearch("walks by the sea", FeedState(cards, endReached = true), ai = AiRank.Understanding("Claude Code"),
            dock = SearchDockState(query = "walks by the sea", engine = "Claude Code"))
        compose.onNodeWithText("Claude Code is reading what you wrote").assertExists()
        snap("gallery_search_working")
    }

    @Test fun nothingFoundOffersOneWordOrTheEngine() {
        val searched = mutableListOf<String>()
        showSearch("kayaking trips", FeedState(emptyList(), endReached = true),
            dock = SearchDockState(query = "kayaking trips", engine = "Claude Code"), actions = SearchDockActions(onSearchFor = { searched += it }))
        compose.onNodeWithText("Nothing matches “kayaking trips” yet").assertExists()
        compose.onNodeWithText("Ask Claude Code").assertExists()
        compose.onNodeWithText("No cards found").assertDoesNotExist()
        snap("gallery_search_nothing")
        compose.onNodeWithText("Search “kayaking”").performClick()
        assertEquals(listOf("kayaking"), searched)
    }

    @Test fun searchAnsweredByTheEngine() {
        val reading = AiUnderstood("Walking essays saved in spring", tags = listOf("walks"), spaces = listOf(AiSpace("s1", "Reading")),
            question = true)
        val rank = AiRank.Ranked("Claude Code", 2, reading, "You saved the walking essay in April, with two photos of the same trail.",
            cards.take(1))
        var only: Boolean? = null
        val query = "when did I save the walking essay"
        showSearch(query, FeedState(cards, endReached = true), ai = rank, dock = SearchDockState(query = query, engine = "Claude Code"),
            actions = SearchDockActions(onOnlyRanked = { only = it }))
        compose.onNodeWithText("Claude Code ranked 2 first").assertExists()
        compose.onNodeWithText("You saved the walking essay in April, with two photos of the same trail.").assertExists()
        snap("gallery_search_answered")
        compose.onNodeWithText("Read more").performClick()
        compose.onNodeWithText("#walks").assertExists()
        compose.onNodeWithText("Space: Reading").assertExists()
        compose.onNodeWithText(sourceLabel(cards.first())).assertExists()
        snap("gallery_search_answered_more")
        compose.onNodeWithText("Only these 2").performClick()
        assertEquals(true, only)
    }
    @Test fun differentNoteTypes() = capture(2, "gallery_notes", sample = mixedNotes, highlights = emptyList())
    @Test fun compactNoteTypes() = capture(4, "gallery_notes_compact", sample = mixedNotes, highlights = emptyList())
    @Test fun threeAcrossNoteTypes() = capture(3, "gallery_notes_three", sample = mixedNotes, highlights = emptyList())
    @Test fun threeAcrossGallery() = capture(3, "gallery_three")
    @Test fun printsOfEveryKind() = capture(3, "gallery_kinds", sample = kinds, highlights = emptyList())
    @Test fun printsOfEveryKindTwoAcross() = capture(2, "gallery_kinds_two", sample = kinds, highlights = emptyList())

    @Test fun mixedNotePreviewsFollowThePinchBeforeRelease() {
        val watchedUrl = "https://gallery.example.test/forest.jpg?pinch-fixture=walk"
        val sample = mixedNotes.map { card -> if (card.id == "walk") card.copy(cover = card.cover!!.copy(src = watchedUrl)) else card }
        show(2, sample = sample, highlights = emptyList())
        compose.waitUntil(10_000) { watchedUrl in loaded }
        val initialFetches = fetches[watchedUrl]
        val initialDecodes = decodes[watchedUrl]
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        fun noteWidth() = compose.onNodeWithContentDescription("Weekend walk").fetchSemanticsNode().boundsInRoot.width
        fun textLayout(): TextLayoutResult {
            val results = mutableListOf<TextLayoutResult>()
            // The card's own tile, not its inert layout at the other stop while a pinch cross-fades the two.
            compose.onNode(hasText(sample.first { it.id == "walk" }.note!!) and hasAnyAncestor(hasClickAction()), useUnmergedTree = true)
                .performSemanticsAction(SemanticsActions.GetTextLayoutResult) { assertTrue(it(results)) }
            return results.single()
        }
        fun frame(name: String) {
            compose.waitForIdle()
            compose.onRoot().captureRoboImage("screenshots/gallery_pinch_${name}_${if (dark) "dark" else "light"}.png")
        }
        fun spread(fraction: Float) {
            gallery.performTouchInput {
                updatePointerTo(0, Offset(width * fraction, height * 0.22f))
                updatePointerTo(1, Offset(width * (1f - fraction), height * 0.22f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
        }
        val initial = noteWidth()
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.22f))
            down(1, Offset(width * 0.9f, height * 0.22f))
        }
        compose.mainClock.autoAdvance = false
        // The finger position [finger] is reached by pinching 30% per stop from the two columns the pinch began at.
        fun spreadTo(finger: Float) = spread((1f - 0.8f * 1.3f.pow(2f - finger)) / 2f)
        spreadTo(2.4f)
        compose.mainClock.advanceTimeByFrame()
        assertEquals(steppedPosition(2.4f), mosaicState.position, 0.02f)
        // Cards on their way between the two grids, with the blurred fill behind them, never a second touchable grid.
        compose.onAllNodesWithTag("mosaic-projection", useUnmergedTree = true).assertCountEquals(1)
        frame("moving")
        spreadTo(2.7f)
        compose.mainClock.advanceTimeBy(48)
        frame("hop")
        repeat(4) {
            // Jitter just past a stop.
            for (finger in listOf(2.09f, 2.11f)) {
                spreadTo(finger)
                compose.mainClock.advanceTimeBy(16)
            }
        }
        compose.mainClock.advanceTimeBy(160)
        compose.onAllNodesWithTag("mosaic-projection", useUnmergedTree = true).assertCountEquals(1)
        assertEquals(steppedPosition(2.11f), mosaicState.position, 0.02f)
        val heldWidth = noteWidth()
        frame("held")
        compose.mainClock.advanceTimeBy(120)
        compose.onAllNodesWithTag("mosaic-projection", useUnmergedTree = true).assertCountEquals(1)
        assertEquals("Holding the fingers must not change the live focal size", heldWidth, noteWidth(), 2f)
        compose.mainClock.autoAdvance = true
        spread(0.2f)
        val requestsAfterHandoff = requests[watchedUrl]
        spread(0.205f)
        assertEquals("Measuring another pointer frame must keep the same projection request", requestsAfterHandoff, requests[watchedUrl])
        val middle = noteWidth()
        assertTrue("The preview must shrink while both fingers remain down", middle < initial - 8f)
        frame("middle")
        // Exactly to the densest stop: past it, the pinch discards the overshoot so a reversal answers at once.
        spreadTo(4f)
        assertTrue("One pinch must continue through more than one size", noteWidth() < middle - 8f)
        assertEquals("Additional projections must reuse cached image bytes", initialFetches, fetches[watchedUrl])
        assertEquals("Additional projections must reuse the decoded memory image", initialDecodes, decodes[watchedUrl])
        frame("dense")
        spreadTo(2f)
        assertEquals("Reversing the fingers must restore the original preview size", initial, noteWidth(), 2f)
        frame("reversed")
        gallery.performTouchInput { up(0); up(1) }
        compose.waitForIdle()
        assertEquals("Releasing at the original size must not jump", initial, noteWidth(), 2f)

        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.22f))
            down(1, Offset(width * 0.9f, height * 0.22f))
        }
        spread(0.3f)
        val denseWidth = noteWidth()
        val beforeRelease = textLayout()
        gallery.performTouchInput { up(0); up(1) }
        compose.waitForIdle()
        val afterRelease = textLayout()
        assertEquals("The dense preview must keep its geometry on release", denseWidth, noteWidth(), 2f)
        assertEquals("The font must not reset on release", beforeRelease.layoutInput.style.fontSize, afterRelease.layoutInput.style.fontSize)
        assertEquals("The excerpt must not rewrap on release", beforeRelease.lineCount, afterRelease.lineCount)
        frame("dense_released")
    }

    @Test fun landingOnFewerRowsKeepsDrawingEveryFrame() {
        // The notes make more rows at two columns than at three, so as a landing on three ends, the list still holds
        // rows that size does not have, for a frame.
        show(2, sample = mixedNotes, highlights = emptyList())
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.22f))
            down(1, Offset(width * 0.9f, height * 0.22f))
        }
        compose.mainClock.autoAdvance = false
        compose.mainClock.advanceTimeByFrame()
        for (finger in listOf(2.1f, 2.3f, 2.55f)) {
            val fraction = (1f - 0.8f * 1.3f.pow(2f - finger)) / 2f
            gallery.performTouchInput {
                updatePointerTo(0, Offset(width * fraction, height * 0.22f))
                updatePointerTo(1, Offset(width * (1f - fraction), height * 0.22f))
                move(delayMillis = 16)
            }
            compose.mainClock.advanceTimeByFrame()
        }
        repeat(8) { compose.mainClock.advanceTimeByFrame() }
        gallery.performTouchInput { up(0); up(1) }
        var frames = 0
        while (mosaicState.position != mosaicState.columns.toFloat() && frames++ < 60) {
            compose.mainClock.advanceTimeByFrame()
            compose.onRoot().captureToImage()
        }
        repeat(3) { compose.mainClock.advanceTimeByFrame(); compose.onRoot().captureToImage() }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
        assertEquals(3, mosaicState.columns)
    }

    companion object {
        @JvmStatic @ParameterizedRobolectricTestRunner.Parameters(name = "dark={0}") fun modes() = listOf(arrayOf(false), arrayOf(true))
    }
}

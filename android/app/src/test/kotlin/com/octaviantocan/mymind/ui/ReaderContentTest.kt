package com.octaviantocan.mymind.ui

import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.onRoot
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.GalleryImage
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.data.ThreadPost
import com.octaviantocan.mymind.data.PostArticle
import com.octaviantocan.mymind.data.RedditComment
import com.octaviantocan.mymind.data.RedditInfo
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.card.ReaderActions
import com.octaviantocan.mymind.ui.card.ReaderPart
import com.octaviantocan.mymind.ui.card.ReaderText
import com.octaviantocan.mymind.ui.card.bylineOf
import com.octaviantocan.mymind.ui.card.readerSections
import com.octaviantocan.mymind.ui.card.readingMinutes
import com.octaviantocan.mymind.ui.card.readingTime
import com.octaviantocan.mymind.ui.card.savedLabel
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import java.time.LocalDate
import java.time.ZoneOffset
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** What the reader shows of a card, and how its preview, source link and tags respond. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ReaderContentTest {
    @get:Rule val compose = createComposeRule()

    private val page = testCard("w1", type = "WebPage", url = "https://ultracite.ai/", title = "Ultracite",
        tags = listOf(Tag("t1", "Linting"), Tag("t2", "Formatting")))
        .copy(description = "A zero-configuration preset.", domain = "ultracite.ai",
            cover = Cover(src = "/media/ultracite.png", width = 1200, height = 900))

    @Test
    fun theReaderKeepsEveryPartOfTheCardOnceAndInOrder() {
        val prose = Json.parseToJsonElement("""{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"The README."}]}]}""")
        val repository = testCard("r1", type = "Repository", note = "Try it for demos").copy(description = "Screen recording for agents.", prose = prose)
        assertEquals(listOf(ReaderText(ReaderPart.Summary, "Screen recording for agents."), ReaderText(ReaderPart.YourNote, "Try it for demos"),
            ReaderText(ReaderPart.Page, "The README.")), readerSections(repository))
        assertEquals("1 min read", readingTime(repository))
        val post = testCard("x1", type = "XPost").copy(description = "Hello", post = Post(bodyHtml = "<p>Hello</p>"))
        assertEquals(listOf(ReaderText(ReaderPart.Post, "Hello")), readerSections(post))
        assertNull(readingTime(post))
        val note = testCard("n1", type = "Note", title = null, note = "Just a thought")
        assertEquals(listOf(ReaderText(ReaderPart.Note, "Just a thought")), readerSections(note))
    }

    @Test
    fun theBylineSaysWhoWhereWhenAndHowLong() {
        assertEquals(0, readingMinutes(" \n "))
        assertEquals(1, readingMinutes(List(200) { "word" }.joinToString(" ")))
        assertEquals(2, readingMinutes(List(201) { "word" }.joinToString("\n")))
        val today = LocalDate.of(2026, 10, 28)
        assertEquals("Saved 24 Oct", savedLabel("2026-10-24T09:00:00Z", today, ZoneOffset.UTC))
        assertEquals("Saved 2 Mar 2025", savedLabel("2025-03-02T23:30:00Z", today, ZoneOffset.UTC))
        assertNull(savedLabel("not a date", today, ZoneOffset.UTC))
        assertEquals("Henry David Thoreau · The Atlantic",
            bylineOf(testCard("a1", url = "https://www.theatlantic.com/x").copy(author = " Henry David Thoreau ", siteName = "The Atlantic")))
        assertEquals("example.com", bylineOf(testCard("a2", url = "https://www.example.com/a")))
        assertNull(bylineOf(testCard("n2", type = "Note")))
    }

    @Test
    fun thePreviewAndTheSourceLinkOpenTheOriginalAndHoldingThePreviewOpensActions() {
        val opened = mutableListOf<String>()
        var menus = 0
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                CardDetail(page, onClose = {}, onMenu = { menus++ }, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = { opened += it })
            }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithContentDescription("Preview").performClick()
        compose.onNodeWithContentDescription("Open source, ultracite.ai").performClick()
        compose.onNodeWithContentDescription("Preview").performTouchInput { longClick() }
        compose.runOnIdle {
            assertEquals(listOf("https://ultracite.ai/", "https://ultracite.ai/"), opened)
            assertEquals(1, menus)
        }
    }

    @Test
    fun theFootOpensTheCardsActionsAndShareAndAddShowOnlyWithTheirActions() {
        var menus = 0
        var shares = 0
        var adds = 0
        val actions = mutableStateOf(ReaderActions())
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                CardDetail(page, onClose = {}, onMenu = { menus++ }, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = {}, actions = actions.value)
            }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithContentDescription("Share").assertDoesNotExist()
        compose.onNodeWithText("+ Add").assertDoesNotExist()
        compose.onNodeWithContentDescription("Card actions").performClick()
        compose.change { actions.value = ReaderActions(onShare = { shares++ }, onAddTags = { adds++ }) }
        compose.onNodeWithContentDescription("Share").performClick()
        compose.onNodeWithText("+ Add").performScrollTo().performClick()
        compose.runOnIdle { assertEquals(listOf(1, 1, 1), listOf(menus, shares, adds)) }
    }

    @Test
    fun tagsStayQuietUntilEditTagsShowsTheirCrosses() {
        val removed = mutableListOf<String>()
        val editing = mutableStateOf(false)
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                CardDetail(page, onClose = {}, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onRemoveTag = { removed += it.name.orEmpty() },
                    onOpenSource = {}, editingTags = editing.value, actions = ReaderActions(onEditingTags = { editing.value = it }))
            }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithText("Linting").performScrollTo()
        compose.onNodeWithText("Edit tags").assertDoesNotExist()
        compose.onNodeWithContentDescription("Remove tag Linting").assertDoesNotExist()
        compose.runOnIdle { editing.value = true }
        compose.onNodeWithContentDescription("Remove tag Formatting").performClick()
        compose.onNodeWithText("Done").performClick()
        compose.onNodeWithContentDescription("Remove tag Linting").assertDoesNotExist()
        compose.runOnIdle { assertEquals(listOf("Formatting"), removed) }
    }

    @Test
    fun aRedditPostKeepsItsTopCommentsUnderThePost() {
        val thread = testCard("r1", type = "RedditPost", url = "https://www.reddit.com/r/hiking/comments/1abc/", title = "The longest walk?")
            .copy(author = "fernhollow", post = Post(bodyHtml = "<p>What is the longest day you have walked?</p>"),
                reddit = RedditInfo("The longest walk?", "hiking", listOf(RedditComment("moss_and_map", "Forty-one km along the coast."),
                    RedditComment("ridgeline", "Thirty, in the rain."))))
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) { CardDetail(thread, onClose = {}, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = {}) }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNodeWithText("Top comments").assertExists()
        compose.onNodeWithText("u/moss_and_map").assertExists()
        compose.onNodeWithText("Thirty, in the rain.").assertExists()
        compose.onAllNodesWithText("What is the longest day you have walked?").assertCountEquals(1)
    }

    @Test
    fun anXPostShowsTheArticleItCarriesAndATapOpensThePost() {
        val opened = mutableListOf<String>()
        val article = PostArticle("The quiet economics of walking cities", "A city that is good to walk in.", "https://x.com/i/article/1844")
        val post = testCard("xa", type = "XPost", url = "https://x.com/noor/status/1844", title = article.title)
            .copy(post = Post(bodyHtml = "<p>New piece.</p>", author = "by Noor"), cover = Cover("/media/cover.jpg", 1456, 582), article = article)
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) { CardDetail(post, onClose = {}, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = { opened += it }) }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithText("A city that is good to walk in.", useUnmergedTree = true).assertExists()
        compose.onNodeWithText("Article on X", useUnmergedTree = true).assertExists()
        compose.onAllNodesWithContentDescription("Preview").assertCountEquals(0)
        compose.onNodeWithContentDescription("Article, ${article.title}").performClick()
        compose.runOnIdle { assertEquals(listOf("https://x.com/noor/status/1844"), opened) }
    }

    @Test
    fun anXThreadSaysWhatItSaysThenShowsEveryPost() {
        val summary = "Theo redrew every map in a 1931 walking guide by hand, with one pen weight."
        val thread = testCard("xt", type = "XPost", url = "https://x.com/theo_maps/status/1844", title = "One pen weight: a 1931 walking guide")
            .copy(summary = summary, post = Post(bodyHtml = "<p>A folding booklet this winter.</p>", author = "by Theo Park", thread = listOf(
                ThreadPost("<p>I spent a year redrawing every map in a 1931 walking guide.</p>", gallery = listOf(GalleryImage("/media/hills.jpg", 1200, 800))),
                ThreadPost("<p>Every contour is drawn with the same pen.</p>"),
                ThreadPost("<p>A folding booklet this winter.</p>"),
            )))
        compose.setPhoneContent(true) {
            PageTheme(PageScheme.Light) { CardDetail(thread, onClose = {}, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = {}) }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNodeWithText("What the thread says").assertExists()
        compose.onNodeWithText(summary).assertExists()
        compose.onNodeWithText("Every contour is drawn with the same pen.", useUnmergedTree = true).assertExists()
        compose.onAllNodesWithText("A folding booklet this winter.", useUnmergedTree = true).assertCountEquals(1)
        compose.onNodeWithText("3 posts", substring = true, useUnmergedTree = true).assertExists()
        compose.onAllNodesWithContentDescription("Picture in the thread", useUnmergedTree = true).assertCountEquals(1)
        compose.onRoot().captureRoboImage("screenshots/S11_reader_x_thread_light.png")
    }

    @Test
    fun aSavedPostInTheReader() {
        compose.setPhoneContent(true) {
            PageTheme(PageScheme.Dark) {
                CardDetail(page.copy(type = "XPost", post = Post(bodyHtml = "<p>Ultracite is a zero-configuration preset for ESLint, Biome and Oxlint.</p>")),
                    onClose = {}, onMenu = {}, onRemoveTag = {}, onOpenSource = {})
            }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.onRoot().captureRoboImage("screenshots/S10_reader_post_dark.png")
    }
}

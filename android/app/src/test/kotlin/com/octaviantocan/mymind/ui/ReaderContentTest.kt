package com.octaviantocan.mymind.ui

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.onRoot
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.card.readerParagraphs
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
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
        assertEquals(listOf("Screen recording for agents.", "The README.", "Try it for demos"), readerParagraphs(repository))
        val post = testCard("x1", type = "XPost").copy(description = "Hello", post = Post(bodyHtml = "<p>Hello</p>"))
        assertEquals(listOf("Hello"), readerParagraphs(post))
        val note = testCard("n1", type = "Note", title = null, note = "Just a thought")
        assertEquals(listOf("Just a thought"), readerParagraphs(note))
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
    fun tagsStayQuietUntilEditShowsTheirCrosses() {
        val removed = mutableListOf<String>()
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                CardDetail(page, onClose = {}, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onRemoveTag = { removed += it.name.orEmpty() },
                    onOpenSource = {})
            }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithText("Linting").performScrollTo()
        compose.onNodeWithContentDescription("Remove tag Linting").assertDoesNotExist()
        compose.onNodeWithText("Edit tags").performClick()
        compose.onNodeWithContentDescription("Remove tag Formatting").performClick()
        compose.onNodeWithText("Done").performClick()
        compose.onNodeWithContentDescription("Remove tag Linting").assertDoesNotExist()
        compose.runOnIdle { assertEquals(listOf("Formatting"), removed) }
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

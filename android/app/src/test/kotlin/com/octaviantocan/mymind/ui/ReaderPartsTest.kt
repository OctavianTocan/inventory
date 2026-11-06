package com.octaviantocan.mymind.ui

import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.card.PartChoice
import com.octaviantocan.mymind.ui.card.ReaderActions
import com.octaviantocan.mymind.ui.card.ReaderPart
import com.octaviantocan.mymind.ui.card.partChoices
import com.octaviantocan.mymind.ui.card.partText
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

/** Holding the reader's parts for their own sheets, the note that is not written yet, and Reader Mode. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ReaderPartsTest {
    @get:Rule val compose = createComposeRule()

    private val prose = Json.parseToJsonElement(
        """{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"First paragraph."}]},""" +
            """{"type":"paragraph","content":[{"type":"text","text":"Second paragraph."}]}]}""",
    )
    private val article = testCard("a1", type = "Article", url = "https://example.com/a", title = "Walking",
        tags = listOf(Tag("t1", "walks"), Tag("t2", "essays")))
        .copy(description = "An essay on walking.", prose = prose, cover = Cover(src = "/media/walking.jpg", width = 1600, height = 1067))

    private fun labels(part: ReaderPart, card: com.octaviantocan.mymind.data.Card) = partChoices(part, card).map { it.label }

    @Test
    fun eachPartOffersOnlyWhatItHas() {
        assertEquals(listOf("Copy summary", "Share summary"), labels(ReaderPart.Summary, article))
        assertEquals(listOf("Write a note"), labels(ReaderPart.YourNote, article))
        assertEquals(listOf("Edit note", "Copy note"), labels(ReaderPart.YourNote, article.copy(note = "Reread")))
        assertEquals(listOf("Read in Reader Mode", "Copy text"), labels(ReaderPart.Page, article))
        assertEquals(listOf("Edit tags", "Copy tags"), labels(ReaderPart.Tags, article))
        assertEquals(listOf("Edit tags"), labels(ReaderPart.Tags, article.copy(tags = null)))
        assertEquals(listOf("Save picture", "Copy picture", "Share picture"), labels(ReaderPart.Picture, article))
        val photo = testCard("p1", type = "Image", blob = Blob(path = "/media/p1.jpg", type = "image/jpeg"))
        assertEquals(listOf("Save picture", "Copy picture", "Share picture"), labels(ReaderPart.Picture, photo))
        assertEquals(emptyList<PartChoice>(), partChoices(ReaderPart.Page, testCard("w1")))
        assertEquals(emptyList<PartChoice>(), partChoices(ReaderPart.Picture, testCard("w1")))
        assertEquals("walks, essays", partText(ReaderPart.Tags, article))
        assertEquals(listOf("First paragraph.", "Second paragraph."), readerParagraphs(article))
    }

    @Test
    fun holdingAPartOpensItsSheetAndAnEmptyNoteInvitesOne() {
        val held = mutableListOf<ReaderPart>()
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Light) {
                CardDetail(article, onClose = {}, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = {},
                    actions = ReaderActions(onPart = { held += it }))
            }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithText("An essay on walking.").performScrollTo().performTouchInput { longClick() }
        compose.onNodeWithText("No note yet. Hold here to write one.").performScrollTo().performTouchInput { longClick() }
        compose.onNode(longClickLabelled("Tags actions")).performSemanticsAction(SemanticsActions.OnLongClick)
        compose.onNodeWithText("Hold any part for its own actions").assertDoesNotExist()
        compose.onNodeWithContentDescription("Preview").performScrollTo().performTouchInput { longClick() }
        compose.runOnIdle { assertEquals(listOf(ReaderPart.Summary, ReaderPart.YourNote, ReaderPart.Tags, ReaderPart.Picture), held) }
    }

    @Test
    fun readerModeShowsThePageAloneAndBackReturnsToTheCard() {
        val reading = mutableStateOf(true)
        var closed = 0
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Light) {
                CardDetail(article, onClose = { closed++ }, onMenu = {}, headerIcons = flatSizedCardHeaderIcons(), onOpenSource = {},
                    actions = ReaderActions(onReading = { reading.value = it }), reading = reading.value)
            }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithText("Second paragraph.").assertIsDisplayed()
        compose.onNodeWithText("An essay on walking.").assertDoesNotExist()
        compose.onNodeWithContentDescription("Open source, example.com").assertDoesNotExist()
        compose.onNodeWithContentDescription("Back").performClick()
        compose.runOnIdle { assertEquals(listOf(false, 0), listOf(reading.value, closed)) }
        compose.onNodeWithText("An essay on walking.").assertExists()
        compose.onNodeWithText("Second paragraph.").assertDoesNotExist()
        compose.change { reading.value = true }
        compose.onNodeWithText("Back to the card").performScrollTo().performClick()
        compose.runOnIdle { assertEquals(listOf(false, 0), listOf(reading.value, closed)) }
    }

    private fun longClickLabelled(label: String) =
        SemanticsMatcher("long click $label") { it.config.getOrElseNullable(SemanticsActions.OnLongClick) { null }?.label == label }
}

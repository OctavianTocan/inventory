package com.octaviantocan.mymind.ui

import android.content.Context
import androidx.activity.OnBackPressedDispatcher
import androidx.activity.compose.LocalOnBackPressedDispatcherOwner
import androidx.compose.foundation.ScrollState
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipe
import androidx.compose.ui.test.swipeDown
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Product
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.card.LocalPdfSource
import com.octaviantocan.mymind.ui.card.PdfPages
import com.octaviantocan.mymind.ui.card.PdfSource
import com.octaviantocan.mymind.ui.card.ReaderActions
import com.octaviantocan.mymind.ui.card.ReaderDetails
import com.octaviantocan.mymind.ui.card.pdfParagraphs
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.io.File

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class CardDetailInteractionTest {
    @get:Rule val compose = createComposeRule()
    private val opened = mutableStateOf(true)
    private var closed = 0
    private val scroll = ScrollState(0)
    private lateinit var back: OnBackPressedDispatcher
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "Card details"))

    private val nameCard get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "About this card"))

    private fun open(
        actions: ReaderActions = ReaderActions(),
        details: ReaderDetails = ReaderDetails(),
        card: Card = LibrarySamples.article.copy(description = "Long synthetic card content.\n".repeat(100)),
        pdfs: PdfSource? = null,
    ) {
        compose.setPhoneContent(false) {
            back = checkNotNull(LocalOnBackPressedDispatcherOwner.current).onBackPressedDispatcher
            PageTheme(PageScheme.Dark) {
                CompositionLocalProvider(LocalPdfSource provides pdfs) {
                    if (opened.value) CardDetail(
                        card,
                        onClose = { closed++; opened.value = false },
                        onMenu = {},
                        headerIcons = flatSizedCardHeaderIcons(),
                        scrollState = scroll,
                        actions = actions,
                        details = details,
                    )
                }
            }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    @Test
    fun readerUsesTheFullWindowWithoutSheetStops() {
        open()
        val node = panel.fetchSemanticsNode()
        assertEquals(0f, node.boundsInRoot.top, 2f)
        assertEquals(PHONE_HEIGHT_PX.toFloat(), node.boundsInRoot.bottom, 2f)
        assertTrue(SemanticsActions.Expand !in node.config)
        assertTrue(SemanticsActions.Collapse !in node.config)
        assertEquals(0, closed)
    }

    @Test
    fun accessibilityDismissClosesTheCard() {
        open()
        panel.performSemanticsAction(SemanticsActions.Dismiss) { it() }
        compose.change {}
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    fun pullingDownOnTheHeaderDoesNotMoveOrDismissTheReader() {
        open()
        panel.performTouchInput {
            swipe(Offset(centerX, 30f), Offset(centerX, 600f), durationMillis = 300)
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(0, closed)
        assertEquals(0f, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        compose.onNodeWithContentDescription("Back").assertIsDisplayed()
    }

    @Test
    fun scrollingContentBackToItsTopKeepsTheReaderInPlace() {
        open()
        val body = compose.onNode(hasScrollAction() and !hasText(LibrarySamples.article.title!!))
        compose.runOnIdle { runBlocking { scroll.scrollTo(100) } }
        body.performSemanticsAction(SemanticsActions.ScrollBy) { it(0f, -1000f) }
        compose.mainClock.advanceTimeBy(1500)
        body.performTouchInput { swipeDown(durationMillis = 500) }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(0, scroll.value)
        assertEquals(0f, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        assertEquals(0, closed)
    }

    @Test
    fun theNamePillAndTheHeadSectionEachNameTheCardOnce() {
        open()
        compose.onAllNodesWithText(LibrarySamples.article.title!!).assertCountEquals(2)
    }

    @Test
    fun theNameGrowsIntoItsCardAndBackHidesItBeforeClosingTheReader() {
        val other = LibrarySamples.video
        val opened = mutableListOf<String>()
        var pins = 0
        open(ReaderActions(onTogglePin = { pins++ }, onSpaces = {}, onOpenRecent = { opened += it.id }),
            ReaderDetails(spaces = listOf("Design", "Reading"), recent = listOf(LibrarySamples.article, other)))
        nameCard.assertDoesNotExist()
        compose.onNode(hasText(LibrarySamples.article.title!!) and hasClickAction()).performClick()
        compose.mainClock.advanceTimeBy(1000)
        nameCard.assertExists()
        compose.onNodeWithText("Design, Reading").assertExists()
        compose.onNodeWithText("Copy link").assertDoesNotExist()
        compose.onNodeWithText("Pin").performClick()
        compose.mainClock.advanceTimeBy(1000)
        nameCard.assertExists()
        compose.change { back.onBackPressed() }
        compose.mainClock.advanceTimeBy(1000)
        nameCard.assertDoesNotExist()
        assertEquals(0, closed)
        compose.onNode(hasText(LibrarySamples.article.title!!) and hasClickAction()).performClick()
        compose.mainClock.advanceTimeBy(1000)
        compose.onNodeWithText(other.title!!).performClick()
        compose.mainClock.advanceTimeBy(1000)
        nameCard.assertDoesNotExist()
        assertEquals(listOf(other.id), opened)
        assertEquals(1, pins)
    }

    @Test
    fun systemBackClosesTheCard() {
        open()
        compose.change { back.onBackPressed() }
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    fun aProductShowsItsPriceAndSameVibeOpensACardNearIt() {
        val lamp = testCard("lamp", type = "Product", url = "https://shop.example.test/lamp", title = "Paper lamp")
            .copy(cover = Cover("https://shop.example.test/lamp.jpg", 900, 1200), product = Product("GBP 24.00", "Norrland"))
        val near = LibrarySamples.video
        val shown = mutableListOf<String>()
        open(ReaderActions(onOpenRecent = { shown += it.id }), ReaderDetails(sameVibe = listOf(near)), lamp)
        compose.onNodeWithText("£24").assertExists()
        compose.onNodeWithText("Same Vibe").assertExists()
        compose.onNodeWithContentDescription(near.title!!).performSemanticsAction(SemanticsActions.OnClick)
        assertEquals(listOf(near.id), shown)
    }

    @Test
    fun aPdfShowsEveryPageAndOpensInReaderModeAndInThePhonesViewer() {
        val doc = testCard("doc", type = "Document", title = "Fell walks")
            .copy(blob = Blob("u/doc/fell-walks.pdf", "application/pdf"), cover = Cover("https://example.test/p1.webp", 642, 840))
        val pages = FakePdf(3, "Most visitors arrive by the railway.\nWalk up the lane behind the station.")
        val reading = mutableListOf<Boolean>()
        open(ReaderActions(onReading = { reading += it }), card = doc, pdfs = object : PdfSource {
            override suspend fun file(card: Card) = File("fell-walks.pdf")
            override fun open(file: File) = pages
        })
        compose.onNodeWithText("PDF · 3 pages · 1 min read", substring = true).assertExists()
        compose.onNodeWithContentDescription("Page 1 of 3").assertExists()
        compose.onNodeWithContentDescription("Page 3 of 3").assertExists()
        assertTrue("The page on screen is drawn", 0 in pages.drawn)
        compose.onNodeWithText("Read in Reader Mode").performSemanticsAction(SemanticsActions.OnClick)
        assertEquals(listOf(true), reading)
        compose.onNodeWithContentDescription("Open the PDF").performSemanticsAction(SemanticsActions.OnClick)
        assertEquals(1, pages.viewed)
        compose.change { opened.value = false }
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertTrue("Leaving the reader closes the PDF", pages.closed)
    }

    private class FakePdf(override val count: Int, private val words: String) : PdfPages {
        val drawn = mutableSetOf<Int>()
        var viewed = 0
        var closed = false
        override suspend fun load() = true
        override fun ratio(index: Int) = 0.75f
        override suspend fun draw(index: Int, width: Int): ImageBitmap {
            drawn += index
            return ImageBitmap(width, (width / 0.75f).toInt())
        }
        override suspend fun text() = pdfParagraphs(listOf(words))
        override fun view(context: Context) {
            viewed++
        }
        override fun close() {
            closed = true
        }
    }
}

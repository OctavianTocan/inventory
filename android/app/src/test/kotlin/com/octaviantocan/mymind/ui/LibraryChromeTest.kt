package com.octaviantocan.mymind.ui

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipe
import androidx.compose.ui.test.swipeLeft
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.MosaicState
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class LibraryChromeTest {
    @get:Rule val compose = createComposeRule()
    private val cards = (1..70).map {
        testCard("photo-$it", type = "Image", title = "Photo $it").copy(cover = Cover(width = 1000, height = 1000))
    }
    private val pins = (1..10).map { testCard("pin-$it", type = "Note", title = "Pin $it", note = "Pinned thought $it") }
    private val opened = mutableListOf<String>()
    private val held = mutableListOf<String>()
    private val controls = mutableListOf<String>()
    private var refreshes = 0

    private fun show(
        state: LazyListState = LazyListState(),
        mosaic: MosaicState = MosaicState(),
        feed: List<Card> = cards,
        highlights: List<Card> = pins,
    ) {
        compose.setPhoneContent(true) {
            ShellFrame(Tab.Everything, {}, PageScheme.Dark) {
                EverythingContent(
                    feed = FeedState(feed, endReached = true), searching = false, searchResults = null,
                    typeChips = emptyList(), filter = null, suggestions = emptyList(),
                    onChoose = {}, onRemoveFilter = {}, onLoadMore = {}, onLoadMoreResults = {}, onRetry = {},
                    onOpenCard = { opened += it.id }, onOpenSettings = { controls += "settings" }, onAdd = { controls += "add" },
                    onSerendipity = { controls += "rediscover" }, onLongPressCard = { held += it.id },
                    onRefresh = { refreshes++ }, topOfMind = highlights, gridState = state, mosaic = mosaic,
                )
            }
        }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
    }

    private fun controlBounds(): List<Rect> = listOf("Account settings", "Add", "Rediscover cards").map {
        compose.onNodeWithContentDescription(it).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
    }

    @Test fun galleryScrollKeepsTheControlsFixedAndTheUntitledPinRailStillScrolls() {
        val state = LazyListState()
        show(state)
        val before = controlBounds()
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val rail = compose.onNodeWithContentDescription("Pinned cards").assertIsDisplayed()
        compose.onNodeWithText("Top of Mind").assertDoesNotExist()
        assertTrue(gallery.fetchSemanticsNode().boundsInRoot.top >= before.maxOf { it.bottom })

        rail.performTouchInput { swipeLeft(durationMillis = 500) }
        compose.waitForIdle()
        val railProgress = rail.fetchSemanticsNode().config[SemanticsProperties.HorizontalScrollAxisRange].value()
        assertTrue("A horizontal swipe must move the pinned cards", railProgress > 0f)
        assertEquals(0, state.firstVisibleItemIndex)
        assertEquals(0, state.firstVisibleItemScrollOffset)
        assertEquals(before, controlBounds())
        rail.performScrollToIndex(9)
        compose.onNodeWithContentDescription("Pin 10").performClick()
        assertEquals(listOf("pin-10"), opened)

        gallery.performTouchInput {
            swipe(Offset(centerX, height * 0.82f), Offset(centerX, height * 0.18f), durationMillis = 700)
        }
        compose.waitForIdle()
        assertTrue("The gallery gesture must scroll its pinned header row out of the viewport", state.firstVisibleItemIndex > 0)
        assertEquals("Scrolling the gallery must leave every top control in place", before, controlBounds())
        compose.onNodeWithContentDescription("Account settings").performClick()
        compose.onNodeWithContentDescription("Add").performClick()
        compose.onNodeWithContentDescription("Rediscover cards").performClick()
        assertEquals(listOf("settings", "add", "rediscover"), controls)
    }

    @Test fun aRealPinchKeepsChromeUnscaledAndMixedPreviewTextBesideItsImage() {
        val body = "Keep a quiet moment."
        val localPicture = "android.resource://com.octaviantocan.mymind/drawable/ic_library_grid"
        val mixed = testCard("mixed-note", type = "Note", title = "Weekend", note = body)
            .copy(cover = Cover(localPicture, 1000, 1000))
        val mosaic = MosaicState()
        show(mosaic = mosaic, feed = listOf(mixed) + cards, highlights = pins.take(1))
        val before = controlBounds()
        val rail = compose.onNodeWithContentDescription("Pinned cards").fetchSemanticsNode().boundsInRoot
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val tile = compose.onNodeWithContentDescription("Weekend")
        val tileBefore = tile.assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val galleryBounds = gallery.fetchSemanticsNode().boundsInRoot
        val focal = tileBefore.center - galleryBounds.topLeft
        assertMixedPreview(body)

        gallery.performTouchInput {
            down(0, focal - Offset(width * 0.18f, 0f))
            down(1, focal + Offset(width * 0.18f, 0f))
            updatePointerTo(0, focal - Offset(width * 0.07f, 0f))
            updatePointerTo(1, focal + Offset(width * 0.07f, 0f))
            move(delayMillis = 120)
            up(0)
            moveTo(1, focal + Offset(width * 0.07f, height * 0.1f), delayMillis = 80)
            up(1)
        }
        compose.waitForIdle()
        assertEquals(4, mosaic.columns)
        assertEquals(before, controlBounds())
        assertEquals(rail, compose.onNodeWithContentDescription("Pinned cards").fetchSemanticsNode().boundsInRoot)
        assertTrue(tile.fetchSemanticsNode().boundsInRoot.width < tileBefore.width)
        assertMixedPreview(body)
        assertTrue(opened.isEmpty())
        assertTrue(held.isEmpty())
        assertEquals(0, refreshes)
        tile.performClick()
        assertEquals(listOf("mixed-note"), opened)
    }

    private fun assertMixedPreview(body: String) {
        val text = compose.onNodeWithText(body, useUnmergedTree = true).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val picture = compose.onNode(
            hasContentDescription("Image preview") and hasAnyAncestor(hasContentDescription("Weekend")),
            useUnmergedTree = true,
        ).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        assertTrue("The note and its image must keep separate visible areas", text.bottom <= picture.top + 1f)
        assertTrue(picture.height > 0f && picture.width > 0f)
    }
}

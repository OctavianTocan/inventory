package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipe
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.theme.MymindTheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config

@RunWith(AndroidJUnit4::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class LibraryMosaicTest {
    @get:Rule val compose = createComposeRule()
    private val cards = (1..70).map { testCard("m$it", type = "Image", title = "Photo $it") }

    private fun pinch(inward: Boolean, yFraction: Float = 0.48f) {
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            val begin = width * if (inward) 0.32f else 0.14f
            val end = width * if (inward) 0.14f else 0.32f
            val y = height * yFraction
            down(0, Offset(centerX - begin, y))
            down(1, Offset(centerX + begin, y))
            moveTo(0, Offset(centerX - end, y), delayMillis = 120)
            moveTo(1, Offset(centerX + end, y), delayMillis = 120)
            up(0)
            up(1)
        }
        compose.waitForIdle()
    }

    @Test
    fun realTwoFingerPinchesChangeDensityBothWaysWithoutOpeningACard() {
        val density = MosaicState()
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, mosaic = density, onLongPressCard = { held += it.id })
            }
        }
        pinch(inward = true)
        assertEquals(3, density.columns)
        pinch(inward = false)
        assertEquals(2, density.columns)
        assertTrue(opened.isEmpty())
        assertTrue(held.isEmpty())
        assertTrue(!density.isPinching)
    }

    @Test
    fun aScrolledPinchKeepsTheTouchedRowsCardsVisible() {
        val density = MosaicState()
        val state = LazyListState(firstVisibleItemIndex = 8, firstVisibleItemScrollOffset = 40)
        compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val bounds = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode().boundsInRoot
        var touched = emptyList<String>()
        compose.runOnIdle {
            val point = bounds.height * 0.48f
            val row = state.layoutInfo.visibleItemsInfo.first { point >= it.offset && point < it.offset + it.size }
            touched = mosaicRows(cards, bounds.width, 2, 3f * 450f / 160f)[row.index].cards.map { it.title!! }
        }
        pinch(inward = true)
        assertEquals(3, density.columns)
        assertTrue(state.firstVisibleItemIndex > 0)
        touched.forEach { compose.onNodeWithContentDescription(it).assertIsDisplayed() }
    }

    @Test
    fun tappingAndHoldingStillUseTheCardAtTheSelectedDensity() {
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, mosaic = MosaicState(3), onLongPressCard = { held += it.id })
            }
        }
        compose.onNodeWithContentDescription("Photo 1").performClick()
        compose.onNodeWithContentDescription("Photo 2").performTouchInput { longClick() }
        assertEquals(listOf("m1"), opened)
        assertEquals(listOf("m2"), held)
    }

    @Test
    fun anAppendedPageCanLoadTheNextPageWhileTheEndStaysVisible() {
        var feed by mutableStateOf(FeedState(cards.take(2)))
        var loads = 0
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(feed, onLoadMore = {
                    loads++
                    val size = (feed.cards.size + 2).coerceAtMost(8)
                    feed = FeedState(cards.take(size), endReached = size == 8)
                }, onOpenCard = {})
            }
        }
        compose.waitForIdle()
        assertEquals(8, feed.cards.size)
        assertEquals(3, loads)
    }
    @Test
    fun shrinkingAClippedSingleColumnKeepsThePinchedCardVisible() {
        val density = MosaicState(1)
        val state = LazyListState(firstVisibleItemIndex = 8, firstVisibleItemScrollOffset = 700)
        compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed()
        pinch(inward = true, yFraction = 0.04f)
        assertEquals(2, density.columns)
        val gallery = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode().boundsInRoot
        val kept = compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val focalY = gallery.top + gallery.height * 0.04f
        assertTrue("The resized card must remain at the pinch's height", focalY >= kept.top && focalY < kept.bottom)
    }

    @Test
    fun accessibleResizingKeepsAClippedSingleColumnCardVisible() {
        val density = MosaicState(1)
        val state = LazyListState(firstVisibleItemIndex = 8, firstVisibleItemScrollOffset = 700)
        compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed()
        val action = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode()
            .config[androidx.compose.ui.semantics.SemanticsActions.CustomActions].single { it.label == "Smaller cards" }
        compose.runOnIdle { assertTrue(action.action()) }
        compose.waitForIdle()
        assertEquals(2, density.columns)
        compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed()
    }

    @Test
    fun pinchingInsidePullRefreshDoesNotRefreshOrOpenCardsAndAPullStillWorks() {
        val density = MosaicState()
        var refreshes = 0
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                PullRefresh(false, { refreshes++ }) {
                    LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id },
                        mosaic = density, onLongPressCard = { held += it.id })
                }
            }
        }
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            down(0, Offset(width * 0.2f, height * 0.12f))
            down(1, Offset(width * 0.8f, height * 0.12f))
            moveTo(0, Offset(width * 0.4f, height * 0.6f), delayMillis = 120)
            moveTo(1, Offset(width * 0.6f, height * 0.6f), delayMillis = 120)
            up(0)
            moveTo(1, Offset(width * 0.6f, height * 0.7f), delayMillis = 80)
            up(1)
        }
        compose.waitForIdle()
        assertEquals(3, density.columns)
        assertEquals(0, refreshes)
        assertTrue(opened.isEmpty())
        assertTrue(held.isEmpty())
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            swipe(Offset(centerX, height * 0.12f), Offset(centerX, height * 0.85f), durationMillis = 700)
        }
        compose.waitForIdle()
        assertEquals(1, refreshes)
    }

}

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

    private fun pinch(inward: Boolean) {
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            val begin = width * if (inward) 0.32f else 0.14f
            val end = width * if (inward) 0.14f else 0.32f
            val y = height * 0.48f
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
}

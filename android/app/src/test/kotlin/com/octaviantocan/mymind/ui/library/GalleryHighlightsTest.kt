package com.octaviantocan.mymind.ui.library

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToIndex
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
class GalleryHighlightsTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun aNewFirstPinAppearsAtTheStartAndOpensItsOwnCard() {
        val cards = (1..10).map { testCard("p$it", type = "Article", title = "Pin $it") }
        var pins by mutableStateOf(cards)
        val opened = mutableListOf<String>()
        compose.setContent { MymindTheme(dark = true) { GalleryHighlights(pins, { opened += it.id }) } }
        pins = listOf(testCard("new", type = "Article", title = "New pin")) + cards
        compose.onNodeWithContentDescription("New pin").assertIsDisplayed().performClick()
        assertEquals(listOf("new"), opened)
    }

    @Test
    fun theRailLoadsNearItsEndWithoutChangingPinOrder() {
        val cards = (1..10).map { testCard("p$it", type = "Article", title = "Pin $it") }
        var more = 0
        val opened = mutableListOf<String>()
        compose.setContent { MymindTheme(dark = false) { GalleryHighlights(cards, { opened += it.id }, onLoadMore = { more++ }) } }
        compose.onNode(hasScrollAction()).performScrollToIndex(9)
        compose.onNodeWithContentDescription("Pin 10").performClick()
        assertTrue(more > 0)
        assertEquals(listOf("p10"), opened)
    }
}

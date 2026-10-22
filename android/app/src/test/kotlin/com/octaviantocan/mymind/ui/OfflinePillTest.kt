package com.octaviantocan.mymind.ui

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.LocalOffline
import com.octaviantocan.mymind.ui.library.testCard
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** Without internet the gallery stays on show with a small Offline pill under its controls, instead of a blocking screen. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class OfflinePillTest {
    @get:Rule val compose = createComposeRule()

    @Test fun theGalleryStaysUsableAndSaysItIsOffline() {
        val offline = mutableStateOf(true)
        compose.setPhoneContent(false) {
            CompositionLocalProvider(LocalOffline provides offline.value) {
                EverythingContent(
                    feed = FeedState(cards = listOf(testCard("a", title = "Kept on the phone")), endReached = true), searching = false,
                    searchResults = null, typeChips = emptyList(), filter = null, suggestions = emptyList(),
                    onChoose = {}, onRemoveFilter = {}, onLoadMore = {}, onLoadMoreResults = {},
                    onRetry = {}, onOpenCard = {}, onOpenSettings = {}, onAdd = {},
                )
            }
        }
        compose.onNodeWithText("Offline").assertIsDisplayed()
        compose.onNodeWithContentDescription("Account settings").assertIsDisplayed()
        compose.runOnIdle { offline.value = false }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNodeWithText("Offline").assertDoesNotExist()
    }
}

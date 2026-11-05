package com.octaviantocan.mymind.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.click
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipeDown
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.library.CardGrid
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.QueuedUpload
import com.octaviantocan.mymind.ui.library.SpaceContent
import com.octaviantocan.mymind.ui.library.PinnedRow
import com.octaviantocan.mymind.ui.library.UploadQueueState
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.shell.MymindTabBar
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.upload.AddSheetContent
import com.octaviantocan.mymind.ui.upload.PICK_FILES
import com.octaviantocan.mymind.ui.upload.PHOTO_LIBRARY
import com.octaviantocan.mymind.ui.upload.TAKE_PHOTO
import com.octaviantocan.mymind.ui.upload.UploadQueueStrip
import com.octaviantocan.mymind.ui.upload.queueItemText
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import java.time.Instant
import java.util.TimeZone

/** Taps, long presses and pulls on the round 3 pieces, with a recording haptic feedback. */
@RunWith(AndroidJUnit4::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class InteractionTest {
    @get:Rule
    val compose = createComposeRule()

    private val haptics = mutableListOf<HapticFeedbackType>()

    private fun show(content: @Composable () -> Unit) = compose.setContent {
        val recorder = object : HapticFeedback {
            override fun performHapticFeedback(hapticFeedbackType: HapticFeedbackType) {
                haptics += hapticFeedbackType
            }
        }
        CompositionLocalProvider(LocalHapticFeedback provides recorder) { MymindTheme(dark = false) { content() } }
    }

    private val cards = listOf(
        testCard("a", type = "Note", title = null, note = "First note"),
        testCard("b", type = "Note", title = null, note = "Second note"),
    )

    @Test
    fun tabTapsGiveTheConfirmHaptic() {
        var selected = Tab.Everything
        show { MymindTabBar(Tab.Everything, onSelect = { selected = it }) }
        compose.onNodeWithText("Spaces").performClick()
        assertEquals(Tab.Spaces, selected)
        assertEquals(listOf(HapticFeedbackType.Confirm), haptics)
    }

    @Test
    fun aLongPressOpensTheCardMenuWithTheLongPressHaptic() {
        val opened = mutableListOf<String>()
        val pressed = mutableListOf<String>()
        show { CardGrid(FeedState(cards, endReached = true), onLoadMore = {}, onOpenCard = { opened += it.id }, onLongPressCard = { pressed += it.id }) }
        compose.onNodeWithText("Second note").performTouchInput { longClick() }
        assertEquals(listOf("b"), pressed)
        assertTrue(opened.isEmpty())
        assertEquals(listOf(HapticFeedbackType.LongPress), haptics)
        compose.onNodeWithText("First note").performClick()
        assertEquals(listOf("a"), opened)
        assertEquals(1, haptics.size)
    }

    @Test
    fun cardsPutBeforeTheFirstShowAtTheTopOfAGridAtRest() {
        val many = (1..20).map { testCard("c$it", type = "Note", title = null, note = "Note $it") }
        var feed by mutableStateOf(FeedState(many, endReached = true))
        show { CardGrid(feed, onLoadMore = {}, onOpenCard = {}) }
        feed = feed.copy(cards = listOf(testCard("n", type = "Note", title = null, note = "New note")) + many)
        compose.onNodeWithText("New note").assertIsDisplayed()
    }

    @Test
    fun aNewFirstPinShowsAtTheStartOfTheRow() {
        val pinned = (1..10).map { testCard("p$it", type = "Article", title = "Pin $it") }
        var pins by mutableStateOf(pinned)
        show { PinnedRow(pins, onOpenCard = {}) }
        pins = listOf(testCard("p0", type = "Article", title = "New pin")) + pinned
        compose.onNodeWithText("New pin").assertIsDisplayed()
    }

    @Test
    fun pullingAnOpenedSpaceDownRefreshesItWithoutAHaptic() {
        var refreshed = 0
        show {
            SpaceContent(Space("s1", "One"), FeedState(cards, endReached = true), onBack = {}, onLoadMore = {}, onOpenCard = {}, onAdd = {}, onRefresh = { refreshed++ })
        }
        compose.onRoot().performTouchInput { swipeDown(startY = height * 0.2f, endY = height * 0.9f, durationMillis = 600) }
        compose.waitForIdle()
        assertEquals(1, refreshed)
        assertTrue(haptics.isEmpty())
    }

    @Test
    fun theAddSheetOffersEachSource() {
        var photos = 0
        var files = 0
        var camera = 0
        var withCamera by mutableStateOf(true)
        show {
            SheetFixture("Add to your mind") {
                AddSheetContent({ photos++ }, { files++ }, if (withCamera) ({ camera++ }) else null)
            }
        }
        compose.onNodeWithText(PHOTO_LIBRARY).performClick()
        compose.onNodeWithText(PICK_FILES).performClick()
        compose.onNodeWithText(TAKE_PHOTO).performClick()
        compose.onNodeWithText("New space").assertDoesNotExist()
        assertEquals(listOf(1, 1, 1), listOf(photos, files, camera))
        withCamera = false
        compose.onNodeWithText(TAKE_PHOTO).assertDoesNotExist()
    }

    @Test
    fun theStripCountsTheQueueListsItAndRemovesAFailedFile() {
        val zone = TimeZone.getDefault()
        TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
        try {
            val added = Instant.parse("2026-10-17T12:00:00Z").toEpochMilli()
            var state by mutableStateOf(
                UploadQueueState(
                    listOf(
                        QueuedUpload(1, "a.jpg", "image/jpeg", added, running = true),
                        QueuedUpload(2, "b.pdf", "application/pdf", added, error = "Refused"),
                    ),
                ),
            )
            val dismissed = mutableListOf<Long>()
            show { UploadQueueStrip(state, onToggle = { state = state.copy(details = !state.details) }, onDismiss = { dismissed += it }) }
            compose.onNodeWithText("2 items in your upload queue").performClick()
            compose.onNodeWithText("Image saved on 2026.10.17").assertExists()
            compose.onNodeWithText("Refused").assertExists()
            compose.onNodeWithContentDescription("Remove b.pdf").performClick()
            assertEquals(listOf(2L), dismissed)
            assertEquals("Video saved on 2026.10.17", queueItemText("video/mp4", added))
            assertEquals("PDF saved on 2026.10.17", queueItemText("application/pdf", added))
        } finally {
            TimeZone.setDefault(zone)
        }
    }
}

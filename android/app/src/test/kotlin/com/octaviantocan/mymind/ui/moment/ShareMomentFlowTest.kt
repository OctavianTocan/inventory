package com.octaviantocan.mymind.ui.moment

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.share.ShareDetails
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.PHONE_FONT_SCALE
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.setPhoneContent
import com.octaviantocan.mymind.ui.share.SAVED_FADE_MS
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareSheetState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/**
 * share-moment-shooting-star and share-tag-step-icon-grid: after a share the star plays, the tag grid waits and goes by
 * itself unless touched, what was chosen reaches the saved cards, and a tap while it plays skips it all.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ShareMomentFlowTest {
    @get:Rule
    val compose = createComposeRule()

    private val cameras = Space("s1", "Cameras", "#4f8a5b", count = 41)
    private val state = ShareSheetState(listOf(ShareItem.Text("https://example.com/m6")), SavePhase.Saved).apply {
        spaces = listOf(cameras, Space("s2", "Film", "#3e8db5", count = 23))
        recentTags = listOf("35mm", "vintage")
        suggestion = SpaceSuggestion(cameras, 12)
        saved = 1
    }
    private val details = ArrayList<ShareDetails>()
    private var closed = 0
    private val moment = StarMoment(null, null, seed = 3L)

    private fun show() {
        compose.setPhoneContent(dark = true) {
            ShareMoment(state, count = { FindCount(Rarity.Rare, 40) }, onDetails = { details += it }, onSky = {}, onClosed = { closed++ }, moment = moment, web = null)
        }
    }

    private fun pass(ms: Long) {
        compose.mainClock.advanceTimeBy(ms)
        compose.waitForIdle()
    }

    private fun untilTheGridShows() {
        var waited = 0
        while (compose.onAllNodesWithText("Search tags").fetchSemanticsNodes().isEmpty()) {
            check(waited < 15_000) { "The tag grid never showed" }
            pass(100); waited += 100
        }
    }

    @Test
    fun leftAloneTheGridGoesByItselfWithoutPickingASpace() {
        show()
        untilTheGridShows()
        assertEquals(Rarity.Rare, moment.shownRarity)
        pass(SAVED_FADE_MS - 600)
        assertTrue("It still waits", details.isEmpty())
        pass(1_400)
        assertEquals("A suggested space is never chosen for the user", listOf(ShareDetails()), details)
        assertEquals(0, closed)
        pass(4_000)
        assertEquals("It closes once the find is in the backpack", 1, closed)
    }

    @Test
    fun aTouchKeepsTheGridUntilItIsSaved() {
        show()
        untilTheGridShows()
        compose.onNodeWithText("35mm").performClick()
        pass(30_000)
        assertTrue("A touch stops the wait", details.isEmpty())
        compose.onNodeWithText("Save 1 tag").performClick()
        pass(100)
        assertEquals(listOf(ShareDetails(tags = listOf("35mm"))), details)
        pass(4_000)
        assertEquals(1, closed)
    }

    @Test
    fun aSpaceIsKeptOnlyWhenTapped() {
        show()
        untilTheGridShows()
        compose.onNodeWithText("Cameras").performClick()
        compose.onNodeWithText("Done").performClick()
        pass(100)
        assertEquals(listOf(ShareDetails(spaceIds = setOf("s1"))), details)
    }

    @Test
    fun aTapWhileItPlaysSkipsToTheBackpack() {
        show()
        pass(1_000)
        compose.onNodeWithContentDescription("Saved. Tap to skip").performClick()
        pass(3_500)
        assertTrue(moment.skipped)
        assertEquals(1, closed)
        assertTrue("The tag step is skipped too", details.isEmpty())
    }

    @Test
    fun whenNothingWasSavedTheSheetSaysWhy() {
        state.phase = SavePhase.Saving
        show()
        pass(1_500)
        state.phase = SavePhase.Failed
        pass(1_000)
        compose.onAllNodesWithText("Nothing was saved").fetchSemanticsNodes().let { assertTrue("The share sheet shows", it.isNotEmpty()) }
        assertEquals(StarPhase.Done, moment.phase)
        assertEquals(0, closed)
    }
}

package com.octaviantocan.mymind.ui.share

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.onNodeWithContentDescription
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.share.ShareState
import com.octaviantocan.mymind.ui.PHONE_FONT_SCALE
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.setPhoneContent
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** share-sheet-fades-by-itself: what shows after a share goes by itself unless it is touched. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ShareSheetFadeTest {
    @get:Rule
    val compose = createComposeRule()

    private var closed = 0
    private val state = ShareSheetState(listOf(ShareItem.Text("https://example.com/walking")))

    private fun show() {
        compose.setPhoneContent(false) { ShareSheet(state, onClose = { closed++ }) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun pass(ms: Long) {
        compose.mainClock.advanceTimeBy(ms)
        compose.waitForIdle()
    }

    @Test
    fun leftAloneItFadesFiveSecondsAfterSaving() {
        show()
        pass(10_000)
        assertEquals("It waits while it is still saving", 0, closed)
        state.phase = SavePhase.Saved
        pass(SAVED_FADE_MS - 500)
        assertEquals(0, closed)
        pass(500 + FADE_OUT_MS + 100L)
        assertEquals(1, closed)
    }

    @Test
    fun aTouchKeepsItOpenUntilDone() {
        state.phase = SavePhase.Saved
        show()
        compose.onNodeWithContentDescription("Add a space, a tag or a note").performClick()
        compose.onNodeWithContentDescription("Add a space, a tag or a note").performTextInput("walks")
        pass(30_000)
        assertEquals(0, closed)
        compose.onNodeWithText("Done").performClick()
        pass(1_000)
        assertEquals(1, closed)
    }

    @Test
    fun aShareThatFailedWaitsLonger() {
        state.phase = SavePhase.Failed
        show()
        pass(SAVED_FADE_MS + FADE_OUT_MS + 100L)
        assertEquals(0, closed)
        pass(PROBLEM_FADE_MS - SAVED_FADE_MS)
        assertEquals(1, closed)
    }

    @Test
    fun aProblemFadesAfterEightSecondsUnlessTouched() {
        var gone = 0
        var acted = 0
        compose.setPhoneContent(false) { ShareProblem(ShareState.FileTooBig, onAction = { acted++ }, onClose = { gone++ }) }
        pass(PROBLEM_FADE_MS + FADE_OUT_MS + 100L)
        assertEquals(1, gone)
        assertEquals(0, acted)
    }

    @Test
    fun aTouchedProblemWaitsForItsButton() {
        var gone = 0
        var acted = 0
        compose.setPhoneContent(false) { ShareProblem(ShareState.NoLogin, onAction = { acted++ }, onClose = { gone++ }) }
        pass(SETTLE_MS)
        compose.onNodeWithText("ThyMind keeps things once you’re signed in. Open ThyMind, sign in, then share this again.").performClick()
        pass(30_000)
        assertEquals(0, gone + acted)
        compose.onNodeWithText("Open ThyMind").performClick()
        pass(1_000)
        assertEquals(1, acted)
        assertEquals(0, gone)
    }

    private companion object {
        const val SETTLE_MS = 1_000L
    }
}

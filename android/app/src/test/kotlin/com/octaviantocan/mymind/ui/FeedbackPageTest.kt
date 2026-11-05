package com.octaviantocan.mymind.ui

import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.CLAUDE_CODE
import com.octaviantocan.mymind.data.CODEX
import com.octaviantocan.mymind.data.FeedbackProblem
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.ui.settings.FEEDBACK_CONSENT
import com.octaviantocan.mymind.ui.settings.FEEDBACK_NO_ENGINE
import com.octaviantocan.mymind.ui.settings.FeedbackPage
import com.octaviantocan.mymind.ui.settings.ProfilePage
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The Send feedback page of the Profile sheet. */
@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class FeedbackPageTest {
    @get:Rule
    val compose = createComposeRule()

    private val closing = mutableListOf<AutoCloseable>()

    @After
    fun tearDown() = closing.forEach { it.close() }

    private fun ai(claude: Boolean, codex: Boolean) = AiFixture(claude, codex).also { closing += it }

    private fun show(ai: AiFixture, tracker: TrackerFixture = TrackerFixture(), opened: MutableList<ProfilePage> = mutableListOf(),
                     providers: () -> Unit = {}, pollMs: Long = 60_000) {
        compose.setPhoneContent(dark = false) {
            SheetFixture("Send feedback") { FeedbackPage(ai.account, tracker.tracker, { opened += it }, providers, pollMs = pollMs) }
        }
        settle()
    }

    private fun settle() {
        compose.mainClock.advanceTimeBy(300)
        compose.waitForIdle()
    }

    @Test
    fun withNoEngineSignedInThePageSaysSoAndOpensProviders() {
        var providers = 0
        show(ai(claude = false, codex = false), providers = { providers++ })
        compose.onNodeWithText(FEEDBACK_NO_ENGINE).performClick()
        compose.runOnIdle { assertEquals(1, providers) }
        compose.onNode(hasSetTextAction()).performTextInput("Make the cards rounder")
        settle()
        compose.onNodeWithText("Send").assertIsNotEnabled()
        compose.onNodeWithText(FEEDBACK_CONSENT).assertExists()
    }

    @Test
    fun withOneEngineSignedInItIsTheOnlyOneOffered() {
        val ai = ai(claude = false, codex = true)
        show(ai)
        compose.onNodeWithText("Codex · Sol · Medium").assertExists()
        compose.runOnIdle {
            assertEquals(CODEX, ai.account.feedbackPrefs.choice.value.engine)
            // AI search keeps its own engine.
            assertEquals(CLAUDE_CODE, ai.prefs.choice.value.engine)
        }
    }

    @Test
    fun withBothEnginesItStartsAtTheFirstRecommendedModelAtMediumAndTheLineOpensTheModelPage() {
        val ai = ai(claude = true, codex = true)
        val opened = mutableListOf<ProfilePage>()
        show(ai, opened = opened)
        compose.onNodeWithText("Claude Code · Opus · Medium").assertExists().performClick()
        compose.runOnIdle {
            assertEquals(listOf(ProfilePage.FeedbackModel), opened)
            assertEquals(AiChoice(CLAUDE_CODE, "opus", "medium"), ai.account.feedbackPrefs.choice.value)
            assertEquals(AiChoice(), ai.prefs.choice.value)
        }
        compose.onNodeWithText("Effort").performClick()
        compose.runOnIdle { assertEquals(ProfilePage.FeedbackEffort, opened.last()) }
        compose.onNodeWithText(FEEDBACK_CONSENT).assertExists()
    }

    @Test
    fun feedbackPicksLeaveSearchAlone() {
        val ai = ai(claude = true, codex = true)
        ai.prefs.chooseEngine(CODEX)
        ai.account.feedbackPrefs.chooseEngine(CLAUDE_CODE)
        ai.account.feedbackPrefs.chooseEffort("low")
        assertEquals(CODEX, ai.prefs.choice.value.engine)
        assertEquals("", ai.prefs.choice.value.effort)
        assertEquals("low", ai.account.feedbackPrefs.choice.value.effort)
        assertTrue(ai.sharedPrefs.contains("feedback.engine"))
        assertEquals(CODEX, ai.sharedPrefs.getString("engine", null))
    }

    @Test
    fun theTitlesAreTheNamesTheDesignGives() {
        assertEquals(
            listOf("Search model", "Search effort", "Send feedback", "Feedback model", "Feedback effort"),
            listOf(ProfilePage.Model, ProfilePage.Effort, ProfilePage.Feedback, ProfilePage.FeedbackModel, ProfilePage.FeedbackEffort).map { it.title },
        )
    }

    @Test
    fun aRefusedSendShowsTheServersSentenceAndTheRetryKeepsItsClientId() {
        val ai = ai(claude = true, codex = false)
        val tracker = TrackerFixture().apply { api.refuse = FeedbackProblem("Only Tavi can send feedback for now.") }
        show(ai, tracker)
        compose.onNodeWithText("Send").assertIsNotEnabled()
        compose.onNode(hasSetTextAction()).performTextInput("Make the cards rounder")
        settle()
        compose.onNodeWithText("Send").assertIsEnabled().performClick()
        settle()
        compose.onNodeWithText("Only Tavi can send feedback for now.").assertExists()
        assertTrue(tracker.store.active().isEmpty())

        tracker.api.refuse = null
        compose.onNodeWithText("Send").performClick()
        settle()
        val sends = tracker.api.sent
        assertEquals(2, sends.size)
        assertEquals(sends[0].third, sends[1].third)
        assertEquals("Make the cards rounder", sends[1].first)
        assertEquals(AiChoice(CLAUDE_CODE, "opus", "medium"), sends[1].second)
        assertEquals(listOf("job1"), tracker.store.active())
        compose.onNodeWithText("Your feedback is waiting its turn").assertExists()
        assertEquals(1, tracker.scheduled)

        // Words typed after a failed send are another send.
        assertNotEquals("", sends[1].third)
    }

    @Test
    fun anActiveJobIsShownAndAskedAboutEveryPollUntilItEnds() {
        val ai = ai(claude = true, codex = false)
        val tracker = TrackerFixture()
        tracker.api.answers["job9"] = FeedbackStatus("job9", "working", "Working on your feedback")
        tracker.store.add("job9")
        show(ai, tracker, pollMs = 20_000)
        compose.onNodeWithText("Working on your feedback").assertExists()
        // The form waits while the job runs.
        compose.onNodeWithText("Send").assertDoesNotExist()
        val first = tracker.api.asked.size
        compose.mainClock.advanceTimeBy(20_100)
        compose.waitForIdle()
        assertTrue(tracker.api.asked.size > first)

        tracker.api.answers["job9"] = FeedbackStatus("job9", "failed", "Couldn't ship this: the tests kept failing", "Try again.")
        compose.mainClock.advanceTimeBy(20_100)
        compose.waitForIdle()
        compose.onNodeWithText("Couldn't ship this: the tests kept failing").assertExists()
        // A final job stops the polling and brings the form back.
        val atEnd = tracker.api.asked.size
        compose.mainClock.advanceTimeBy(60_000)
        compose.waitForIdle()
        assertEquals(atEnd, tracker.api.asked.size)
        compose.onNodeWithText("Send").assertExists()
        assertTrue(tracker.store.active().isEmpty())
    }
}


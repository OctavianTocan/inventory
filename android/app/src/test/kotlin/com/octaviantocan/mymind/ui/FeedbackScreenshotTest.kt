package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.ui.settings.FeedbackPage
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The Send feedback page in the Profile sheet, light and dark. */
@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class FeedbackScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private val ai = AiFixture(claude = true, codex = true)

    @After
    fun tearDown() = ai.close()

    private fun page(dark: Boolean) {
        val tracker = TrackerFixture()
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(Tab.Everything, onSelectTab = {}, scheme = if (dark) PageScheme.Dark else PageScheme.Light) { SampleEverything() }
                SheetFixture("Send feedback") { FeedbackPage(ai.account, tracker.tracker, open = {}, onProviders = {}) }
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onNode(hasSetTextAction()).performTextInput("The cards on the Everything page could be a little rounder.")
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    @Test
    fun feedbackLight() {
        page(dark = false)
        compose.onRoot().captureRoboImage("screenshots/feedback_light.png")
    }

    @Test
    fun feedbackDark() {
        page(dark = true)
        compose.onRoot().captureRoboImage("screenshots/feedback_dark.png")
    }

    private companion object {
        const val SETTLE_MS = 1_500L
    }
}

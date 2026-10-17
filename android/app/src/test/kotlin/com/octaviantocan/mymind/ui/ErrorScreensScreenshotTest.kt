package com.octaviantocan.mymind.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.isDialog
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.ui.errors.ApiErrorDialog
import com.octaviantocan.mymind.ui.errors.OfflineScreen
import com.octaviantocan.mymind.ui.errors.WebErrorScreen
import com.octaviantocan.mymind.ui.errors.defaultOfflinePictures
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ErrorScreensScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private fun render(dark: Boolean, content: @Composable () -> Unit) {
        compose.setPhoneContent(dark, content)
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun webError(dark: Boolean, name: String) {
        render(dark) { WebErrorScreen(onRetry = {}) }
        compose.onNodeWithText("Retry").assertExists()
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    private fun dialog(dark: Boolean, name: String) {
        render(dark) { ApiErrorDialog(title = "Sign in failed", message = "The server could not be reached.", onDismiss = {}) }
        compose.onNode(isDialog()).captureRoboImage("screenshots/$name.png")
    }

    private fun offline(dark: Boolean, name: String) {
        render(dark) { OfflineScreen(pictures = flatOfflinePictures(dark)) }
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    private fun offlineShipped(dark: Boolean, name: String) {
        render(dark) { OfflineScreen(pictures = defaultOfflinePictures()) }
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    @Test
    fun webErrorLight() = webError(dark = false, name = "web_error_light")

    @Test
    fun webErrorDark() = webError(dark = true, name = "web_error_dark")

    @Test
    fun apiErrorDialogLight() = dialog(dark = false, name = "api_error_dialog_light")

    @Test
    fun apiErrorDialogDark() = dialog(dark = true, name = "api_error_dialog_dark")

    @Test
    fun offlineLight() = offline(dark = false, name = "offline_light")

    @Test
    fun offlineDark() = offline(dark = true, name = "offline_dark")

    /** Renders the shipped artwork; the files are written for the machine's owner and are never looked at by the agent. */
    @Test
    fun offlineShippedLight() = offlineShipped(dark = false, name = "offline_real_light")

    @Test
    fun offlineShippedDark() = offlineShipped(dark = true, name = "offline_real_dark")

    private companion object {
        const val SETTLE_MS = 1_000L
    }
}

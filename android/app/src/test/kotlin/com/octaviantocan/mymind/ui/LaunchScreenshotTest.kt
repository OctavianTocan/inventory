package com.octaviantocan.mymind.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.signin.SignInOverlay
import com.octaviantocan.mymind.ui.splash.LaunchState
import com.octaviantocan.mymind.ui.splash.SplashPictures
import com.octaviantocan.mymind.ui.splash.defaultSplashPictures
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class LaunchScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private var state by mutableStateOf(LaunchState.Splash)

    private fun start(dark: Boolean, pictures: @Composable () -> SplashPictures, googleLogo: @Composable () -> Painter) {
        compose.setPhoneContent(dark) {
            SignInOverlay(state, onGoogle = {}, onPassword = { _, _ -> }, pictures = pictures(), googleLogo = googleLogo())
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun moveTo(next: LaunchState, name: String) {
        compose.change { state = next }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    private fun sequence(dark: Boolean, suffix: String) {
        start(dark, { flatSplashPictures() }, { BlockPainter(Color.Gray, Size(67.5f, 67.5f)) })
        compose.onRoot().captureRoboImage("screenshots/splash_$suffix.png")
        moveTo(LaunchState.Login, "sign_in_$suffix")
        compose.onNodeWithText("Sign in with Google").assertExists()
        moveTo(LaunchState.LoggingIn, "signing_in_$suffix")
    }

    private fun realPictures(dark: Boolean, suffix: String) {
        start(dark, { defaultSplashPictures() }, { painterResource(R.drawable.svg_google) })
        compose.onRoot().captureRoboImage("screenshots/splash_real_$suffix.png")
        moveTo(LaunchState.Login, "sign_in_real_$suffix")
    }

    private fun keyboard(dark: Boolean, suffix: String) {
        state = LaunchState.Login
        compose.setPhoneContent(dark, keyboardPx = KEYBOARD_PX) {
            SignInOverlay(state, onGoogle = {}, onPassword = { _, _ -> }, pictures = flatSplashPictures(), googleLogo = BlockPainter(Color.Gray, Size(67.5f, 67.5f)))
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/sign_in_keyboard_$suffix.png")
        val google = compose.onNodeWithText("Sign in with Google").fetchSemanticsNode().boundsInRoot
        val title = compose.onNodeWithText("Access\nyour mind").fetchSemanticsNode().boundsInRoot
        val username = compose.onNodeWithText("Username").fetchSemanticsNode().boundsInRoot
        val button = compose.onNodeWithText("Sign in").fetchSemanticsNode().boundsInRoot
        assertTrue("Google button ends at ${google.bottom}, form starts at ${username.top}", google.bottom <= username.top)
        assertTrue("title starts at ${title.top}", title.top >= STATUS_BAR_PX)
        assertTrue("form ends at ${button.bottom}, keyboard starts at ${PHONE_HEIGHT_PX - KEYBOARD_PX}", button.bottom <= PHONE_HEIGHT_PX - KEYBOARD_PX)
    }

    /** With the keyboard up, the title and the Google button stay clear of the password form, which sits on the keyboard. */
    @Test
    fun keyboardLight() = keyboard(dark = false, suffix = "light")

    @Test
    fun keyboardDark() = keyboard(dark = true, suffix = "dark")

    @Test
    fun launchSequenceLight() = sequence(dark = false, suffix = "light")

    @Test
    fun launchSequenceDark() = sequence(dark = true, suffix = "dark")

    /** Renders the shipped artwork; the files are written for the machine's owner and are never looked at by the agent. */
    @Test
    fun shippedArtworkLight() = realPictures(dark = false, suffix = "light")

    @Test
    fun shippedArtworkDark() = realPictures(dark = true, suffix = "dark")

    private companion object {
        const val SETTLE_MS = 2_000L

        /** Keyboard height of the reference phone, navigation bar included (`07-search-focus-*.png`). */
        const val KEYBOARD_PX = 986
    }
}

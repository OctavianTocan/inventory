package com.octaviantocan.mymind.ui

import androidx.compose.foundation.ScrollState
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.ui.settings.SettingsDrawer
import com.octaviantocan.mymind.ui.settings.SettingsPictures
import com.octaviantocan.mymind.ui.settings.defaultSettingsPictures
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import kotlinx.coroutines.runBlocking
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class SettingsScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private val scroll = ScrollState(0)

    private fun open(dark: Boolean, pictures: @Composable () -> SettingsPictures) {
        compose.setPhoneContent(dark) {
            SettingsDrawer(
                drawerState = rememberDrawerState(DrawerValue.Open),
                account = "name@example.com",
                version = "1.0 (1)",
                onSignOut = {},
                pictures = pictures(),
                scrollState = scroll,
            ) {
                ShellFrame(Tab.Everything, onSelectTab = {}, scheme = PageScheme.Dark) { SampleEverything() }
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun scrollToEnd() {
        compose.change { runBlocking { scroll.scrollTo(scroll.maxValue) } }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun capture(name: String) = compose.onRoot().captureRoboImage("screenshots/$name.png")

    private fun sheet(dark: Boolean, suffix: String) {
        open(dark) { flatSettingsPictures(dark) }
        compose.onNodeWithText("Sign out of mymind").assertExists()
        capture("settings_$suffix")
        scrollToEnd()
        capture("settings_scrolled_$suffix")
    }

    private fun shipped(dark: Boolean, suffix: String) {
        open(dark) { defaultSettingsPictures() }
        capture("settings_real_$suffix")
        scrollToEnd()
        capture("settings_real_scrolled_$suffix")
    }

    @Test
    fun settingsLight() = sheet(dark = false, suffix = "light")

    @Test
    fun settingsDark() = sheet(dark = true, suffix = "dark")

    /** Renders the shipped artwork; the files are written for the machine's owner and are never looked at by the agent. */
    @Test
    fun settingsShippedLight() = shipped(dark = false, suffix = "light")

    @Test
    fun settingsShippedDark() = shipped(dark = true, suffix = "dark")

    private companion object {
        const val SETTLE_MS = 1_500L
    }
}

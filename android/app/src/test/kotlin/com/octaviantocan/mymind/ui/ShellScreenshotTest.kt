package com.octaviantocan.mymind.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import com.octaviantocan.mymind.ui.shell.MymindTabBar
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.MymindTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ShellScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private fun render(dark: Boolean, content: @Composable BoxScope.() -> Unit) {
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize().background(MymindTheme.colors.page)) {
                content()
            }
        }
    }

    private fun tabBar(dark: Boolean, name: String) {
        render(dark) { MymindTabBar(Tab.Everything, onSelect = {}, modifier = Modifier.align(Alignment.BottomCenter)) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onNodeWithText("Everything").assertIsSelected()
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    /** Version 03 a moment after a change: the chosen place's name shows beside its icon. */
    private fun namedTabBar(dark: Boolean, name: String) {
        var selected by mutableStateOf(Tab.Everything)
        render(dark) { MymindTabBar(selected, onSelect = { selected = it }, modifier = Modifier.align(Alignment.BottomCenter)) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onNodeWithText("Spaces").performClick()
        compose.mainClock.advanceTimeBy(NAMED_MS)
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    private fun spinner(dark: Boolean, name: String) {
        render(dark) { LoadingSpinner(Modifier.align(Alignment.Center), MymindTheme.colors.tabUnselected) }
        compose.mainClock.advanceTimeBy(SPINNER_MS)
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }

    @Test
    fun tabBarLight() = tabBar(dark = false, name = "tab_bar_light")

    @Test
    fun tabBarDark() = tabBar(dark = true, name = "tab_bar_dark")

    @Test
    fun tabBarNamedLight() = namedTabBar(dark = false, name = "tab_bar_named_light")

    @Test
    fun tabBarNamedDark() = namedTabBar(dark = true, name = "tab_bar_named_dark")

    @Test
    fun spinnerLight() = spinner(dark = false, name = "spinner_light")

    @Test
    fun spinnerDark() = spinner(dark = true, name = "spinner_dark")

    private companion object {
        const val SETTLE_MS = 2_000L
        const val NAMED_MS = 700L
        const val SPINNER_MS = 700L
    }
}

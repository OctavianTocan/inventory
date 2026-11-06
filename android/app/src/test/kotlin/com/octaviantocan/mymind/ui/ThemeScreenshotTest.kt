package com.octaviantocan.mymind.ui

import android.content.Context
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.ui.settings.AppearanceRows
import com.octaviantocan.mymind.ui.settings.FIND_A_THEME
import com.octaviantocan.mymind.ui.settings.ProfileSheetContent
import com.octaviantocan.mymind.ui.settings.SettingsUiState
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.ui.settings.ThemePage
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.ColorThemes
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.theme.LocalThemePrefs
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import com.octaviantocan.mymind.ui.theme.ThemePrefs
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** Pully's colour themes and its Light, Dark and System choice: the Appearance group, the Theme page and the gallery drawn in a picked theme. */
@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ThemeScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private fun prefs(): ThemePrefs {
        val context = ApplicationProvider.getApplicationContext<Context>()
        context.getSharedPreferences("theme", Context.MODE_PRIVATE).edit().clear().commit()
        return ThemePrefs(context)
    }

    /** The gallery and the Theme page in [prefs]' theme, as the app draws them. */
    private fun page(dark: Boolean, prefs: ThemePrefs) {
        compose.setPhoneContent(dark) {
            MymindTheme(dark = dark, theme = prefs.theme) {
                Box(Modifier.fillMaxSize()) {
                    ShellFrame(Tab.Everything, onSelectTab = {}, scheme = if (dark) PageScheme.Dark else PageScheme.Light) { SampleEverything() }
                    SheetFixture("Theme") { ThemePage(prefs) }
                }
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    @Test
    fun themePageLight() {
        page(dark = false, prefs())
        compose.onRoot().captureRoboImage("screenshots/theme_page_light.png")
    }

    @Test
    fun themePageDark() {
        page(dark = true, prefs())
        compose.onRoot().captureRoboImage("screenshots/theme_page_dark.png")
    }

    @Test
    fun aTapAppliesTheThemeAtOnceKeepsItAndStaysOnThePage() {
        val prefs = prefs()
        page(dark = true, prefs)
        compose.onNodeWithText(FIND_A_THEME).assertExists()
        compose.onNodeWithContentDescription(FIND_A_THEME).performTextInput("tokyo")
        compose.mainClock.advanceTimeByFrame()
        compose.onAllNodesWithText("Pully").assertCountEquals(0)
        compose.onNodeWithText("Tokyo Night").performClick()
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.runOnIdle { assertEquals("tokyonight", prefs.id) }
        compose.onNodeWithText("Tokyo Night").assertExists()
        compose.onRoot().captureRoboImage("screenshots/theme_tokyonight_dark.png")
        // A new start reads the same theme back.
        assertEquals("tokyonight", ThemePrefs(ApplicationProvider.getApplicationContext<Context>()).id)
    }

    @Test
    fun noMatchSaysSo() {
        page(dark = false, prefs())
        compose.onNodeWithContentDescription(FIND_A_THEME).performTextInput("zzz")
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithText("No theme matches").assertExists()
    }

    @Test
    fun backdropSunriseLight() = backdrop("sunrise", dark = false)

    @Test
    fun backdropSunriseDark() = backdrop("sunrise", dark = true)

    @Test
    fun backdropOceanDark() = backdrop("ocean", dark = true)

    @Test
    fun backdropAuroraLight() = backdrop("aurora", dark = false)

    @Test
    fun backdropNotebookLight() = backdrop("notebook", dark = false)

    @Test
    fun backdropNotebookDark() = backdrop("notebook", dark = true)

    @Test
    fun backdropGardenLight() = backdrop("garden", dark = false)

    @Test
    fun backdropNightSkyDark() = backdrop("night-sky", dark = true)

    @Test
    fun backdropPostcardLight() = backdrop("postcard", dark = false)

    private fun backdrop(id: String, dark: Boolean) {
        val prefs = prefs().apply { choose(ColorThemes.of(id)) }
        compose.setPhoneContent(dark) {
            MymindTheme(dark = dark, theme = prefs.theme) {
                ShellFrame(Tab.Everything, onSelectTab = {}, scheme = if (dark) PageScheme.Dark else PageScheme.Light) { SampleEverything() }
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/theme_${id}_${if (dark) "dark" else "light"}.png")
    }

    @Test
    fun theBackdropThemesInThePicker() {
        page(dark = false, prefs())
        compose.onNodeWithContentDescription(FIND_A_THEME).performTextInput("o")
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/theme_page_backdrops.png")
    }

    @Test
    fun pullyThemeLight() {
        val prefs = prefs().apply { choose(ColorThemes.Pully) }
        page(dark = false, prefs)
        compose.onRoot().captureRoboImage("screenshots/theme_pully_light.png")
    }

    @Test
    fun theProfileSheetOffersAppearanceAndTheThemeInUse() {
        var opened = 0
        val picks = mutableListOf<PageScheme>()
        compose.setPhoneContent(false) {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(Tab.Everything, onSelectTab = {}, scheme = PageScheme.Light) { SampleEverything() }
                SheetFixture("Profile") {
                    ProfileSheetContent(SettingsUiState(email = "name@example.com", displayName = "Ada Lovelace"), "0.6.0 (11)",
                        SettingsUpdates(), onSignOut = {}, onClose = {}, photoLoader = null,
                        appearance = AppearanceRows(PageScheme.System, { picks += it }, "Tokyo Night") { opened++ })
                }
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onNodeWithText("Appearance").assertExists()
        compose.onNodeWithText("Tokyo Night").assertExists()
        compose.onRoot().captureRoboImage("screenshots/theme_profile_appearance.png")
        compose.onNodeWithText("Dark").performClick()
        compose.onNodeWithText("Theme").performClick()
        compose.runOnIdle { assertEquals(listOf(PageScheme.Dark), picks); assertEquals(1, opened) }
    }

    @Test
    fun aPickedAppearanceDrawsTheChromeAndThePagesWhateverTheAccountSays() {
        val prefs = prefs()
        assertEquals(null, prefs.appearance)
        assertEquals(true, prefs.chromeDark(systemDark = true))
        prefs.chooseAppearance(PageScheme.Light)
        assertEquals(false, prefs.chromeDark(systemDark = true))
        var pageDark: Boolean? = null
        compose.setPhoneContent(false) {
            CompositionLocalProvider(LocalThemePrefs provides prefs) {
                PageTheme(PageScheme.Dark) { pageDark = MymindTheme.colors.dark }
            }
        }
        compose.runOnIdle { assertEquals(false, pageDark) }
        assertEquals(PageScheme.Light, ThemePrefs(ApplicationProvider.getApplicationContext<Context>()).appearance)
    }

    private companion object {
        const val SETTLE_MS = 1_000L
    }
}

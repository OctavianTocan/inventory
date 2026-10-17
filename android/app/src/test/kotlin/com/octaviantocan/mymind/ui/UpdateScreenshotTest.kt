package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.background
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.ui.menu.DarkMenuColors
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.ui.settings.SettingsSheet
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.update.AfterUpdateContent
import com.octaviantocan.mymind.ui.update.ChangelogScreen
import com.octaviantocan.mymind.ui.update.UpdatePhase
import com.octaviantocan.mymind.ui.update.UpdateSheetContent
import com.octaviantocan.mymind.ui.update.UpdateSheetModel
import com.octaviantocan.mymind.update.ReleaseInfo
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class UpdateScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private val release = ReleaseInfo("0.3.0", 4, "/download/mymind-0.3.0.apk", 1000, "ab", notes = listOf("Pull to refresh is faster.", "Uploads keep going in the background.", "Fixed a crash when opening a space."))
    private val entries = listOf(
        ChangelogEntry("0.3.0", 4, "2026-10-20", release.notes),
        ChangelogEntry("0.2.0", 2, "2026-10-17", listOf("Sign in with your mymind account.", "Browse everything, spaces and Serendipity.")),
    )

    private fun shot(name: String, dark: Boolean, content: @Composable () -> Unit) {
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize().background(DarkMenuColors.scrim), contentAlignment = Alignment.BottomCenter) { content() }
        }
        compose.mainClock.advanceTimeBy(500)
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }


    @Test
    fun updateSheetLight() = shot("update_sheet_light", false) { UpdateSheetContent(UpdateSheetModel(release, UpdatePhase.Ready), {}, {}, {}, {}) }

    @Test
    fun updateSheetDark() = shot("update_sheet_dark", true) { UpdateSheetContent(UpdateSheetModel(release, UpdatePhase.Ready), {}, {}, {}, {}) }

    @Test
    fun permissionRowLight() = shot("update_permission_light", false) { UpdateSheetContent(UpdateSheetModel(release, UpdatePhase.Ready, installAllowed = false), {}, {}, {}, {}) }

    @Test
    fun permissionRowDark() = shot("update_permission_dark", true) { UpdateSheetContent(UpdateSheetModel(release, UpdatePhase.Ready, installAllowed = false), {}, {}, {}, {}) }

    @Test
    fun afterUpdateLight() = shot("update_after_light", false) { AfterUpdateContent(entries[0]) {} }

    @Test
    fun afterUpdateDark() = shot("update_after_dark", true) { AfterUpdateContent(entries[0]) {} }

    @Test
    fun changelogLight() = shot("update_changelog_light", false) { ChangelogScreen(entries) {} }

    @Test
    fun changelogDark() = shot("update_changelog_dark", true) { ChangelogScreen(entries) {} }

    @Test
    fun settingsRowsLight() = settingsRows(false)

    @Test
    fun settingsRowsDark() = settingsRows(true)

    @Test
    fun laterCallbackFires() {
        var later = 0
        compose.setPhoneContent(false) { UpdateSheetContent(UpdateSheetModel(release, UpdatePhase.Ready), {}, { later++ }, {}, {}) }
        compose.onNodeWithText("Later").performClick()
        assertEquals(1, later)
    }

    private fun settingsRows(dark: Boolean) {
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize().background(MymindTheme.colors.page)) {
                SettingsSheet(
                    account = "name@example.com",
                    version = "0.2.0 (2)",
                    onSignOut = {},
                    pictures = flatSettingsPictures(dark),
                    updates = SettingsUpdates(true, "Version 0.2.0 (2)", "Up to date", showAllow = true),
                    scrollState = androidx.compose.foundation.ScrollState(Int.MAX_VALUE),
                )
            }
        }
        compose.mainClock.advanceTimeBy(1_500)
        compose.onRoot().captureRoboImage("screenshots/update_settings_${if (dark) "dark" else "light"}.png")
    }
}

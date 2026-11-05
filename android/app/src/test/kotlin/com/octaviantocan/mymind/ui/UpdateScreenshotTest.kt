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
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipeUp
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasAnyDescendant
import androidx.compose.ui.test.hasText
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.changelog.Changelog
import androidx.test.core.app.ApplicationProvider
import android.content.Context
import com.octaviantocan.mymind.ui.menu.DarkMenuColors
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.ui.settings.ProfileSheetContent
import com.octaviantocan.mymind.ui.settings.SettingsUiState
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.update.AfterUpdateSheet
import com.octaviantocan.mymind.ui.update.ChangelogScreen
import com.octaviantocan.mymind.ui.update.UpdateGateContent
import com.octaviantocan.mymind.ui.update.UpdateGateModel
import com.octaviantocan.mymind.ui.update.UpdateStep
import com.octaviantocan.mymind.ui.update.offeredEntries
import com.octaviantocan.mymind.update.ReleaseHistory
import com.octaviantocan.mymind.update.UpdateState
import com.octaviantocan.mymind.update.updateStatusText
import java.io.File
import com.octaviantocan.mymind.update.ReleaseInfo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import kotlin.math.abs

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class UpdateScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private val release = ReleaseInfo("0.3.0", 4, "/download/mymind-0.3.0.apk", 1000, "ab", notes = listOf("Pull to refresh is faster.", "Uploads keep going in the background.", "Fixed a crash when opening a space."))
    private val entries by lazy {
        val context = ApplicationProvider.getApplicationContext<Context>()
        Changelog.parse(context.assets.open("CHANGELOG.md").bufferedReader().use { it.readText() })
    }

    private fun shot(name: String, dark: Boolean, content: @Composable () -> Unit) {
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize().background(DarkMenuColors.scrim), contentAlignment = Alignment.BottomCenter) { content() }
        }
        compose.mainClock.advanceTimeBy(500)
        compose.onRoot().captureRoboImage("screenshots/$name.png")
    }


    private val offered = release.copy(
        history = listOf(
            ReleaseHistory("0.3.0", 4, "2026-11-05", release.notes),
            ReleaseHistory("0.2.1", 3, "2026-11-04", listOf("Search finds cards by meaning.")),
            ReleaseHistory("0.2.0", 2, "2026-11-03", listOf("Already installed.")),
        ),
    )

    private fun gate(step: UpdateStep) = UpdateGateModel(offered, offeredEntries(offered, 2), step)

    @Test
    fun updateGateLight() = shot("update_gate_light", false) { UpdateGateContent(gate(UpdateStep.Idle), {}, {}) }

    @Test
    fun updateGateDark() = shot("update_gate_dark", true) { UpdateGateContent(gate(UpdateStep.Idle), {}, {}) }

    @Test
    fun updateGateDownloadingLight() = shot("update_gate_downloading_light", false) { UpdateGateContent(gate(UpdateStep.Downloading(0.42f)), {}, {}) }

    @Test
    fun updateGateFailedDark() = shot("update_gate_failed_dark", true) { UpdateGateContent(gate(UpdateStep.Failed("The install was cancelled")), {}, {}) }

    @Test
    fun updateGateListsOnlyTheVersionsNewerThanTheInstalledOne() {
        assertEquals(listOf("0.3.0", "0.2.1"), offeredEntries(offered, 2).map { it.versionName })
        assertEquals(listOf("0.3.0"), offeredEntries(release, 2).map { it.versionName })
    }

    private fun showStep(step: UpdateStep): Pair<IntArray, IntArray> {
        val updates = IntArray(1)
        val continues = IntArray(1)
        compose.setPhoneContent(false) { UpdateGateContent(gate(step), { updates[0]++ }, { continues[0]++ }) }
        compose.mainClock.advanceTimeBy(100)
        return updates to continues
    }

    @Test
    fun updateGateStartsTheUpdate() {
        val (updates, _) = showStep(UpdateStep.Idle)
        compose.onNodeWithText("ThyMind 0.3.0 is ready").assertIsDisplayed()
        compose.onNodeWithText("Update to 0.3.0").performClick()
        assertEquals(1, updates[0])
    }

    @Test
    fun updateGateAsksToAllowInstallsAndOffersUpdateAgain() {
        showStep(UpdateStep.NeedsAllow)
        compose.onNodeWithText("Allow ThyMind to install updates first.").assertIsDisplayed()
        compose.onNodeWithText("Update to 0.3.0").assertIsDisplayed()
    }

    @Test
    fun updateGateOffersTryAgainAfterAFailure() {
        val (updates, _) = showStep(UpdateStep.Failed("Download interrupted"))
        compose.onNodeWithText("Download interrupted").assertIsDisplayed()
        compose.onNodeWithText("Try again").performClick()
        assertEquals(1, updates[0])
    }

    @Test
    fun updateGateOffersContinueWhileAndroidWaits() {
        val (updates, continues) = showStep(UpdateStep.Confirm)
        compose.onNodeWithText("Continue update").performClick()
        assertEquals(1, continues[0])
        assertEquals(0, updates[0])
    }

    @Test
    fun updateGateHasNoActionWhileInstalling() {
        showStep(UpdateStep.Installing)
        compose.onNodeWithText("Installing…").assertIsDisplayed()
        compose.onAllNodesWithText("Continue update").assertCountEquals(0)
        compose.onAllNodesWithText("Update to 0.3.0").assertCountEquals(0)
    }

    @Test
    fun afterUpdateLight() = shot("update_after_light", false) { AfterUpdateSheet(entries[0]) {} }

    @Test
    fun afterUpdateDark() = shot("update_after_dark", true) { AfterUpdateSheet(entries[0]) {} }

    @Test
    fun changelogLight() = shot("update_changelog_light", false) { ChangelogScreen(entries) {} }

    @Test
    fun changelogDark() = shot("update_changelog_dark", true) { ChangelogScreen(entries) {} }

    @Test
    fun changelogUsesTheWhatsNewBottomSheet() {
        compose.setPhoneContent(false) { ChangelogScreen(entries) {} }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "What's new")).assertIsDisplayed()
    }

    @Test
    fun historyScrollsIndependentlyAndTheCrossClosesIt() {
        var opened by mutableStateOf(true)
        var closes = 0
        compose.setPhoneContent(false) {
            if (opened) ChangelogScreen(entries) { closes++; opened = false }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "What's new"))
            .performSemanticsAction(SemanticsActions.Expand) { it() }
        compose.mainClock.advanceTimeBy(1500)
        val close = compose.onNodeWithContentDescription("Close")
        val before = close.fetchSemanticsNode().boundsInRoot
        // The newest version's badge, and the New section of each version that has one.
        compose.onAllNodesWithText("New").assertCountEquals(1 + entries.count { entry -> entry.groups.any { it.title == "New" } })
        val body = compose.onNode(hasScrollAction() and hasAnyDescendant(hasText("0.1.0")))
        // The history grows with every release, so swipe until its oldest version shows, up to a swipe a version.
        val oldest = compose.onNodeWithText("0.1.0")
        repeat(entries.size) {
            if (runCatching { oldest.assertIsDisplayed() }.isSuccess) return@repeat
            body.performTouchInput { swipeUp(durationMillis = 500) }
            compose.mainClock.advanceTimeBy(500)
            compose.waitForIdle()
        }
        oldest.assertIsDisplayed()
        // The cross stays put while the history scrolls under it. The sheet settles to within a pixel of where it was
        // measured, and CI has seen it a pixel higher after the swipes (28 Oct 2026), so a pixel either way passes.
        val after = close.fetchSemanticsNode().boundsInRoot
        assertTrue("the cross moved from $before to $after", abs(after.top - before.top) <= 1f && abs(after.left - before.left) <= 1f && after.size == before.size)
        close.performClick()
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(1, closes)
        compose.onNodeWithText("What's new").assertDoesNotExist()
    }

    @Test
    fun emptyChangelogHasAReadableState() {
        compose.setPhoneContent(false) { ChangelogScreen(emptyList()) {} }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNodeWithText("Nothing is written down for this version.").assertIsDisplayed()
    }

    @Test
    fun afterUpdateUsesTheSameSheetAndAndroidBackClosesItOnce() {
        var opened by mutableStateOf(true)
        var closes = 0
        compose.setPhoneContent(false) {
            if (opened) com.octaviantocan.mymind.ui.update.AfterUpdateSheet(entries.first()) { closes++; opened = false }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.onNodeWithText("What's new").assertIsDisplayed()
        compose.runOnIdle {
            (org.robolectric.shadows.ShadowDialog.getLatestDialog() as androidx.activity.ComponentDialog)
                .onBackPressedDispatcher.onBackPressed()
        }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(1, closes)
        compose.onNodeWithText("What's new").assertDoesNotExist()
    }

    @Test
    fun settingsRowsLight() = settingsRows(false)

    @Test
    fun settingsRowsDark() = settingsRows(true)

    @Test
    fun settingsReadyAllowedLight() = settingsRows(false, UpdateState.Ready(release, File("a")), allowed = true, name = "ready_allowed")

    @Test
    fun settingsReadyAllowedDark() = settingsRows(true, UpdateState.Ready(release, File("a")), allowed = true, name = "ready_allowed")

    @Test
    fun settingsReadyNotAllowedLight() = settingsRows(false, UpdateState.Ready(release, File("a")), allowed = false, name = "ready_not_allowed")

    @Test
    fun settingsReadyNotAllowedDark() = settingsRows(true, UpdateState.Ready(release, File("a")), allowed = false, name = "ready_not_allowed")

    @Test
    fun settingsInstallingLight() = settingsRows(false, UpdateState.Installing(release), allowed = true, name = "installing")

    @Test
    fun settingsInstallingDark() = settingsRows(true, UpdateState.Installing(release), allowed = true, name = "installing")

    @Test
    fun settingsFailedLight() = settingsRows(false, UpdateState.Failed("The install was cancelled", release), allowed = true, name = "failed")

    @Test
    fun settingsFailedDark() = settingsRows(true, UpdateState.Failed("The install was cancelled", release), allowed = true, name = "failed")

    @Test
    fun readyStatusNamesTheVersion() {
        val ready = UpdateState.Ready(release, File("a"))
        assertEquals("0.3.0 is ready", updateStatusText(ready, true))
        assertEquals("0.3.0 is ready, allow installs below", updateStatusText(ready, false))
    }

    private fun settingsRows(dark: Boolean, state: UpdateState = UpdateState.UpToDate, allowed: Boolean = true, name: String = "") {
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize().background(MymindTheme.colors.page)) {
                SheetFixture("Profile") {
                    ProfileSheetContent(
                        SettingsUiState(email = "name@example.com"),
                        "0.2.0 (2)",
                        SettingsUpdates(true, "Version 0.2.0 (2)", updateStatusText(state, allowed), showAllow = !allowed),
                        onSignOut = {},
                        onClose = {},
                        photoLoader = null,
                    )
                }
            }
        }
        compose.mainClock.advanceTimeBy(1_500)
        compose.onRoot().captureRoboImage("screenshots/update_settings${if (name.isEmpty()) "" else "_$name"}_${if (dark) "dark" else "light"}.png")
    }
}

package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.ConnectedAgent
import com.octaviantocan.mymind.ui.settings.AgentsUiState
import com.octaviantocan.mymind.ui.settings.CONNECTED_AGENTS
import com.octaviantocan.mymind.ui.settings.ConnectedAgentsPage
import com.octaviantocan.mymind.ui.settings.ProfileSheetContent
import com.octaviantocan.mymind.ui.settings.SIGN_OUT
import com.octaviantocan.mymind.ui.settings.SettingsUiState
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import org.junit.Assert.assertEquals
import java.time.LocalDate
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The profile sheet the account circle opens. */
@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class SettingsScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private val account = SettingsUiState(email = "name@example.com", displayName = "Ada Lovelace")

    private fun sheet(dark: Boolean, updates: SettingsUpdates = SettingsUpdates(true, "Version 0.6.0 (11)", "You're up to date"),
                      onSignOut: () -> Unit = {}, onClose: () -> Unit = {}) {
        compose.setPhoneContent(dark) {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(Tab.Everything, onSelectTab = {}, scheme = if (dark) PageScheme.Dark else PageScheme.Light) { SampleEverything() }
                SheetFixture("Profile") { ProfileSheetContent(account, "0.6.0 (11)", updates, onSignOut, onClose, photoLoader = null) }
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    @Test
    fun profileLight() {
        sheet(dark = false)
        compose.onRoot().captureRoboImage("screenshots/settings_light.png")
    }

    @Test
    fun profileDark() {
        sheet(dark = true)
        compose.onRoot().captureRoboImage("screenshots/settings_dark.png")
    }

    @Test
    fun theSheetShowsTheAccountAndItsActionsWork() {
        var signedOut = 0
        var closed = 0
        var changelog = 0
        var checks = 0
        var allow by mutableStateOf(false)
        compose.setPhoneContent(false) {
            SheetFixture("Profile") {
                ProfileSheetContent(account, "0.6.0 (11)",
                    SettingsUpdates(true, "Version 0.6.0 (11)", "You're up to date", showAllow = allow,
                        onCheck = { checks++ }, onChangelog = { changelog++ }),
                    onSignOut = { signedOut++ }, onClose = { closed++ }, photoLoader = null)
            }
        }
        compose.onNodeWithText("Ada Lovelace").assertExists()
        compose.onNodeWithText("name@example.com").assertExists()
        compose.onNodeWithText("Allow installs").assertDoesNotExist()
        compose.onNodeWithText("Check for updates").performClick()
        compose.onNodeWithText("What's new").performClick()
        compose.onNodeWithText(SIGN_OUT).performClick()
        compose.runOnIdle { assertEquals(listOf(1, 1, 1, 1), listOf(checks, changelog, closed, signedOut)) }
        allow = true
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithText("Allow installs").assertExists()
    }

    private val agents = listOf(
        ConnectedAgent("aa11", "Claude", "2026-11-01T12:00:00Z", "2026-11-05T12:00:00Z"),
        ConnectedAgent("bb22", "ChatGPT", "2026-11-04T12:00:00Z", "2026-11-04T12:00:00Z"),
    )

    @Test
    fun connectedAgentsSitAboveSignOut() {
        var opened = 0
        compose.setPhoneContent(false) {
            SheetFixture("Profile") {
                ProfileSheetContent(account, "0.6.0 (11)", SettingsUpdates(), onSignOut = {}, onClose = {}, photoLoader = null,
                    onAgents = { opened++ })
            }
        }
        compose.onNodeWithText(CONNECTED_AGENTS).performClick()
        compose.runOnIdle { assertEquals(1, opened) }
    }

    @Test
    fun removingAConnectedAgentAsksOnceMore() {
        val removed = mutableListOf<String>()
        var loads = 0
        var state by mutableStateOf(AgentsUiState(agents))
        compose.setPhoneContent(false) {
            SheetFixture(CONNECTED_AGENTS) {
                ConnectedAgentsPage(state, onLoad = { loads++ }, today = LocalDate.of(2026, 11, 5), onRemove = { id ->
                    removed += id
                    state = AgentsUiState(state.agents!!.filterNot { it.id == id })
                })
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onNodeWithText("Connected 1 Nov · used today").assertExists()
        compose.onNodeWithText("Connected yesterday").assertExists()
        compose.onNodeWithText("Claude").performClick()
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithText("Tap to remove").assertExists()
        compose.onRoot().captureRoboImage("screenshots/settings_agents.png")
        compose.runOnIdle { assertEquals(emptyList<String>(), removed) }
        compose.onNodeWithText("Claude").performClick()
        compose.mainClock.advanceTimeByFrame()
        compose.runOnIdle { assertEquals(listOf("aa11"), removed) }
        compose.onNodeWithText("Claude").assertDoesNotExist()
        compose.onNodeWithText("ChatGPT").assertExists()
        compose.runOnIdle { assertEquals(1, loads) }
    }

    private companion object {
        const val SETTLE_MS = 1_500L
    }
}

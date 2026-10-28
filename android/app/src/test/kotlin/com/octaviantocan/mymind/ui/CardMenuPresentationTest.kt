package com.octaviantocan.mymind.ui

import androidx.activity.ComponentDialog
import androidx.activity.OnBackPressedDispatcher
import androidx.activity.compose.LocalOnBackPressedDispatcherOwner
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.menu.*
import com.octaviantocan.mymind.data.*
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.shell.MainShell
import kotlinx.coroutines.CompletableDeferred
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okhttp3.mockwebserver.RecordedRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.assertSame
import org.junit.After
import org.robolectric.shadows.ShadowDialog
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class CardMenuPresentationTest {
    @get:Rule val compose = createComposeRule()
    private var opened by mutableStateOf(true)
    private var page by mutableStateOf<CardSubmenu?>(null)
    private var closed = 0
    private lateinit var back: OnBackPressedDispatcher
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, page?.title ?: "Choose one"))

    private fun show(items: List<CardMenuItem> = cardMenuItems(LibrarySamples.grid[1], null)) {
        compose.setPhoneContent(false) {
            Box(Modifier.fillMaxSize())
            if (opened) CardMenuSheet(
                items,
                onSelect = { if (it.action == CardMenuAction.AddTags) page = CardSubmenu.Tags },
                onDismiss = { closed++; opened = false },
                submenu = page,
                onBack = { page = null },
                submenuContent = {
                    back = LocalOnBackPressedDispatcherOwner.current!!.onBackPressedDispatcher
                    AddTagsPage(listOf("design"), emptyList(), onSave = {}, focusOnOpen = false)
                },
                accent = Color(0xFF1D9BF0),
            )
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    @Test fun fullWidthPaperReachesTheBottomAndTheFloatingCrossClosesIt() {
        show()
        val bounds = panel.fetchSemanticsNode().boundsInRoot
        val title = compose.onNodeWithText("Choose one").fetchSemanticsNode().boundsInRoot
        assertEquals("Animated sheet title stays centered", bounds.center.x, title.center.x, 1f)
        assertEquals(0f, bounds.left, 1f)
        assertEquals(1080f, bounds.right, 1f)
        // Pully translates a tall panel; the extra paper continues below the window.
        assertTrue("Paper must continue through the navigation bar", bounds.bottom >= PHONE_HEIGHT_PX - 2f)
        compose.onNodeWithContentDescription("Close card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp).performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test fun dragDownFromTheGrabberDismissesTheSheetOnce() {
        show()
        panel.performTouchInput { swipe(Offset(centerX, 12f), Offset(centerX, height - 20f), durationMillis = 250) }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test fun tagsUseOneSharedHeaderAndBothBackControlsReturnToActions() {
        show()
        // Dialog layout and the page's size animation have separate frame passes.
        compose.mainClock.autoAdvance = true
        panel.performSemanticsAction(SemanticsActions.Expand)
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        val headerTop = compose.onNodeWithContentDescription("Close card actions").fetchSemanticsNode().boundsInRoot.top
        compose.onNodeWithText("Add tags").performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        compose.onAllNodesWithText("Add tags").assertCountEquals(1)
        compose.onNodeWithContentDescription("Close card actions").assertDoesNotExist()
        assertEquals("Changing pages at the tall detent keeps the header in place", headerTop,
            compose.onNodeWithContentDescription("Back").fetchSemanticsNode().boundsInRoot.top, 1f)
        compose.onNodeWithContentDescription("Back").assertIsDisplayed().assertWidthIsEqualTo(44.dp).performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(0, closed)
        assertEquals(null, page)
        compose.onNodeWithText("Choose one").assertIsDisplayed()
        compose.onNodeWithContentDescription("Close card actions").assertIsDisplayed()
        assertEquals(headerTop, compose.onNodeWithContentDescription("Close card actions").fetchSemanticsNode().boundsInRoot.top, 1f)
        compose.onNodeWithText("Add tags").performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        compose.runOnUiThread { back.onBackPressed() }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(0, closed)
        assertEquals(null, page)
        compose.onNodeWithText("Choose one").assertIsDisplayed()
        compose.runOnUiThread { back.onBackPressed() }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test fun childPagesKeepTheSamePanelAndRestoreTheParentScroll() {
        val actions = List(3) { CardMenuAction.entries.filter { it != CardMenuAction.AddTags } }.flatten() + CardMenuAction.AddTags
        show(actions.map { CardMenuItem(it) })
        compose.mainClock.autoAdvance = true
        val panelId = panel.fetchSemanticsNode().id
        compose.onNodeWithText("Add tags").performScrollTo()
        val before = compose.onNode(hasScrollAction()).fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value()
        assertTrue("The parent must actually have scrolled", before > 0f)
        compose.onNodeWithText("Add tags").performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals("The child is inside the same panel", panelId, panel.fetchSemanticsNode().id)
        val child = compose.onNode(hasScrollAction()).fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value()
        assertEquals("A child starts at its own scroll position", 0f, child, 1f)
        compose.onNodeWithContentDescription("Back").performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(panelId, panel.fetchSemanticsNode().id)
        val restored = compose.onNode(hasScrollAction()).fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value()
        assertEquals("Returning restores the parent's scroll", before, restored, 1f)
        assertEquals(0, closed)
    }

    private var settingsServer: MockWebServer? = null

    @After fun closeSettingsServer() { settingsServer?.shutdown() }

    private fun showEmptySpacesShell(): FakeLibrary {
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("selected-card", type = "Note", title = "Selected card", note = "A note")))))
        val local = MockWebServer().apply {
            dispatcher = object : Dispatcher() {
                override fun dispatch(request: RecordedRequest) = MockResponse().setBody("""{"email":"reader@example.test","kind":"password"}""")
            }
            start()
        }
        settingsServer = local
        val store = object : TokenStore {
            override suspend fun read(): String? = null
            override suspend fun write(token: String) = Unit
            override suspend fun clear() = Unit
        }
        val settings = SettingsViewModel(Repository(MymindApi(OkHttpClient(), local.url("/").toString()), Session(store)))
        val model = LibraryViewModel(library)
        compose.setPhoneContent(false) { MainShell(model, settings) }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
        compose.onNodeWithContentDescription("Selected card").performTouchInput { longClick() }
        settlePages()
        return library
    }

    private fun settlePages() {
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    private fun sheet(title: String) = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, title))

    private fun openCreation() {
        compose.onNodeWithText("New space").performScrollTo().performClick()
        settlePages()
    }

    @Test fun newSpaceOpensInPlaceInTheSameSheetAndBothBackControlsReturnToChooseOne() {
        showEmptySpacesShell()
        sheet("Choose one").performSemanticsAction(SemanticsActions.Expand)
        settlePages()
        val panelId = sheet("Choose one").fetchSemanticsNode().id
        val dialog = ShadowDialog.getLatestDialog() as ComponentDialog
        val headerTop = compose.onNodeWithContentDescription("Close card actions").fetchSemanticsNode().boundsInRoot.top
        openCreation()
        compose.onNodeWithContentDescription("Space name").assertIsDisplayed()
        assertEquals(panelId, sheet("Add to space").fetchSemanticsNode().id)
        assertSame(dialog, ShadowDialog.getLatestDialog())
        assertEquals(headerTop, compose.onNodeWithContentDescription("Back").fetchSemanticsNode().boundsInRoot.top, 1f)
        compose.onNodeWithContentDescription("Back").performClick()
        settlePages()
        sheet("Choose one").assertExists()
        openCreation()
        compose.runOnUiThread { dialog.onBackPressedDispatcher.onBackPressed() }
        settlePages()
        sheet("Choose one").assertExists()
        assertEquals(panelId, sheet("Choose one").fetchSemanticsNode().id)
        compose.runOnUiThread { dialog.onBackPressedDispatcher.onBackPressed() }
        settlePages()
        sheet("Choose one").assertDoesNotExist()
        compose.onNodeWithContentDescription("Selected card").assertExists()
    }

    @Test fun createAndAddKeepsAFailureInTheFormAndAddsTheCardOnRetry() {
        val library = showEmptySpacesShell()
        val dialog = ShadowDialog.getLatestDialog()
        openCreation()
        compose.onNodeWithText("Create and add").assertIsNotEnabled()
        compose.onNodeWithContentDescription("Space name").performTextInput("  Weekend  ")
        compose.onNodeWithContentDescription("Space color #ff8fa4").performClick()
        compose.runOnIdle { library.writeFailure = ApiFailure.Network }
        compose.onNodeWithText("Create and add").performScrollTo().performClick()
        settlePages()
        compose.onNodeWithText("Cannot reach the server. Check your connection.").assertExists()
        compose.onNodeWithContentDescription("Space name").assertTextContains("  Weekend  ")
        sheet("Add to space").assertExists()
        assertSame(dialog, ShadowDialog.getLatestDialog())
        compose.runOnIdle {
            library.writeFailure = null
            library.spaceList = listOf(Space("s-new", "Weekend", "#ff8fa4"))
        }
        compose.onNodeWithText("Create and add").performScrollTo().performClick()
        settlePages()
        sheet("Add to space").assertExists()
        compose.onNodeWithContentDescription("Space name").assertDoesNotExist()
        compose.onNode(hasContentDescription("Weekend") and isSelected()).assertIsDisplayed()
        assertSame(dialog, ShadowDialog.getLatestDialog())
        compose.runOnIdle {
            assertEquals(2, library.calls.count { it == "newSpace:Weekend:#ff8fa4" })
            assertEquals(1, library.calls.count { it == "space:s-new:selected-card" })
        }
    }

    @Test fun pendingCreationSurvivesBackAndReopeningTheFormWithoutAnotherPost() {
        val library = showEmptySpacesShell()
        val gate = CompletableDeferred<Unit>()
        compose.runOnIdle { library.createSpaceGate = gate }
        openCreation()
        compose.onNodeWithContentDescription("Space name").performTextInput("Pending space")
        compose.onNodeWithText("Create and add").performScrollTo().performClick()
        settlePages()
        compose.onNodeWithText("Creating…").assertIsNotEnabled()
        compose.onNodeWithContentDescription("Back").performClick()
        settlePages() // Remove the outgoing form and cancel any composition-owned jobs.
        sheet("Choose one").assertExists()
        openCreation()
        compose.onNodeWithContentDescription("Space name").assertIsNotEnabled()
        compose.onNodeWithText("Creating…").performScrollTo().assertIsNotEnabled()
        compose.runOnIdle {
            assertTrue("Back must not cancel the POST", !gate.isCancelled)
            assertEquals(1, library.calls.count { it.startsWith("newSpace:") })
            library.spaceList = listOf(Space("s-new", "Pending space", "#75ffc0"))
            gate.complete(Unit)
        }
        settlePages()
        sheet("Add to space").assertExists()
        compose.onNode(hasContentDescription("Pending space") and isSelected()).assertIsDisplayed()
        compose.runOnIdle {
            assertEquals(1, library.calls.count { it.startsWith("newSpace:") })
            assertEquals(1, library.calls.count { it == "space:s-new:selected-card" })
        }
    }

    @Test fun pendingCreationStillAddsTheCardAfterTheWholeSheetIsDismissed() {
        val library = showEmptySpacesShell()
        val gate = CompletableDeferred<Unit>()
        compose.runOnIdle { library.createSpaceGate = gate }
        openCreation()
        compose.onNodeWithContentDescription("Space name").performTextInput("Created offscreen")
        compose.onNodeWithText("Create and add").performScrollTo().performClick()
        settlePages()
        val dialog = ShadowDialog.getLatestDialog() as ComponentDialog
        repeat(2) {
            compose.runOnUiThread { dialog.onBackPressedDispatcher.onBackPressed() }
            settlePages()
        }
        sheet("Choose one").assertDoesNotExist()
        compose.runOnIdle {
            library.spaceList = listOf(Space("s-new", "Created offscreen", "#75ffc0"))
            gate.complete(Unit)
        }
        settlePages()
        compose.onNodeWithContentDescription("Selected card").performTouchInput { longClick() }
        settlePages()
        compose.onNode(hasContentDescription("Created offscreen") and isSelected()).assertIsDisplayed()
        compose.onNodeWithText("All spaces").assertIsDisplayed()
        compose.runOnIdle {
            assertEquals(1, library.calls.count { it.startsWith("newSpace:") })
            assertEquals(1, library.calls.count { it == "space:s-new:selected-card" })
        }
    }

    @Test fun holdingDeleteClosesTheSheetAndUndoBringsTheCardBack() {
        val library = showEmptySpacesShell()
        sheet("Choose one").performSemanticsAction(SemanticsActions.Expand)
        settlePages()
        val row = compose.onNode(SemanticsMatcher("Delete card row") { it.config.getOrElseNullable(SemanticsActions.OnClick) { null }?.label == "Delete card" })
        row.performScrollTo()
        compose.mainClock.autoAdvance = false
        row.performTouchInput { down(center) }
        compose.mainClock.advanceTimeBy(DELETE_HOLD_MS + 100)
        row.performTouchInput { up() }
        compose.mainClock.autoAdvance = true
        settlePages()
        sheet("Choose one").assertDoesNotExist()
        compose.onNodeWithContentDescription("Selected card").assertDoesNotExist()
        compose.onNodeWithText("Selected card deleted").assertIsDisplayed()
        compose.onNodeWithText("Undo").performClick()
        settlePages()
        compose.onNodeWithContentDescription("Selected card").assertExists()
        compose.onNodeWithText("Selected card deleted").assertDoesNotExist()
        compose.runOnIdle { assertTrue(library.calls.none { it.startsWith("trash:") }) }
    }
}

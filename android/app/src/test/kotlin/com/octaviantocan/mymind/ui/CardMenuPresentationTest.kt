package com.octaviantocan.mymind.ui

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
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
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
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, page?.title ?: "Card actions"))

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
        compose.onNodeWithText("Card actions").assertIsDisplayed()
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
        compose.onNodeWithText("Card actions").assertIsDisplayed()
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
}

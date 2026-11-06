package com.octaviantocan.mymind.ui

import android.provider.Settings
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.click
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.unit.dp
import androidx.test.core.app.ApplicationProvider
import com.octaviantocan.mymind.ui.shell.FOOT_PILL_TAG
import com.octaviantocan.mymind.ui.shell.FootBarLayer
import com.octaviantocan.mymind.ui.shell.FootBars
import com.octaviantocan.mymind.ui.shell.FootPill
import com.octaviantocan.mymind.ui.shell.LibraryFootBar
import com.octaviantocan.mymind.ui.shell.LocalFootBars
import com.octaviantocan.mymind.ui.shell.MymindTabBar
import com.octaviantocan.mymind.ui.shell.Tab
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
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
class FootBarTest {
    @get:Rule val compose = createComposeRule()

    private fun show(bars: FootBars, page: @Composable () -> Unit) {
        compose.setPhoneContent(false) {
            CompositionLocalProvider(LocalFootBars provides bars) {
                Box(Modifier.fillMaxSize()) {
                    page()
                    FootBarLayer(bars, Modifier.align(Alignment.BottomCenter))
                }
            }
        }
        compose.mainClock.autoAdvance = true
        settle()
    }

    private fun settle() {
        compose.mainClock.advanceTimeBy(1200)
        compose.waitForIdle()
    }

    @Test fun onlyTheLatestRegisteredPageOwnsTheOneBarAndDisposalRestoresTheParent() {
        val bars = FootBars()
        var child by mutableStateOf(false)
        show(bars) {
            FootPill("parent") { Text("Parent field") }
            if (child) FootPill("child") { Text("Child field") }
        }
        compose.onNodeWithText("Parent field").assertExists()
        compose.runOnIdle { assertEquals(1, bars.slots.size); child = true }
        settle()
        compose.onNodeWithText("Child field").assertExists()
        compose.onNodeWithText("Parent field").assertDoesNotExist()
        compose.runOnIdle { assertEquals(2, bars.slots.size); child = false }
        settle()
        compose.onAllNodesWithText("Parent field").assertCountEquals(1)
        compose.onNodeWithText("Child field").assertDoesNotExist()
        compose.runOnIdle { assertEquals(1, bars.slots.size) }
    }

    @Test fun navigationHasTwoTabsAndSearchAndKeepsItsRegistrationWhenTheTabChanges() {
        val bars = FootBars()
        var selected by mutableStateOf(Tab.Everything)
        var searched = 0
        show(bars) { MymindTabBar(selected, onSelect = { selected = it }, onSearch = { searched++ }) }
        val slot = bars.top
        compose.onNodeWithText("All").assertIsSelected()
        compose.onNodeWithText("Create").assertDoesNotExist()
        compose.onNodeWithText("Serendipity").assertDoesNotExist()
        compose.onNodeWithText("Spaces").performClick()
        settle()
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onAllNodesWithText("All").assertCountEquals(1)
        compose.onNodeWithContentDescription("Search").performClick()
        compose.runOnIdle { assertEquals(1, searched); assertSame(slot, bars.top); assertEquals(1, bars.slots.size) }
    }

    @Test fun expandingSearchKeepsTheBottomEdgeAndItsPaddingDoesNotOpenTheCardBehind() {
        val bars = FootBars()
        var searching by mutableStateOf(false)
        var query by mutableStateOf("")
        var openedCard = 0
        show(bars) {
            Box(Modifier.fillMaxSize().clickable { openedCard++ })
            LibraryFootBar(Tab.Everything, {}, searching, query, { query = it }, { searching = true }, { searching = false })
        }
        val searchBounds = compose.onNodeWithContentDescription("Search").fetchSemanticsNode().boundsInRoot
        val before = searchBounds.bottom
        val pill = compose.onNodeWithTag(FOOT_PILL_TAG).fetchSemanticsNode().boundsInRoot
        val inset = with(compose.density) { 4.dp.toPx() }
        // The pill's own padding, before its first tab, is the pill's.
        compose.onRoot().performTouchInput { click(Offset(pill.left + inset, pill.center.y)) }
        compose.runOnIdle { assertEquals(0, openedCard) }
        // Beside the small navigation pill the screen's corners belong to the cards (version 03).
        compose.onRoot().performTouchInput { click(Offset(pill.left / 2, pill.center.y)) }
        compose.runOnIdle { assertEquals(1, openedCard); openedCard = 0 }
        compose.onNodeWithContentDescription("Search").performClick()
        settle()
        val close = compose.onNodeWithContentDescription("Close search").fetchSemanticsNode().boundsInRoot
        assertEquals(before, close.bottom, 1f)
        val field = compose.onNodeWithContentDescription("Search your inventory")
        field.assertIsFocused()
        field.performTextInput("forest")
        compose.runOnIdle { assertEquals("forest", query) }
        // The pill starts 12dp in from the screen: this tap is on its left padding, before its search icon.
        val fieldBounds = field.fetchSemanticsNode().boundsInRoot
        compose.onRoot().performTouchInput { click(Offset(47f, fieldBounds.center.y)) }
        field.assertIsFocused()
        compose.runOnIdle { assertEquals(0, openedCard) }
        compose.onNodeWithContentDescription("Clear search").performClick()
        compose.runOnIdle { assertEquals("", query) }
        compose.onNodeWithContentDescription("Close search").performClick()
        settle()
        compose.onNodeWithContentDescription("Search your inventory").assertDoesNotExist()
        compose.onAllNodesWithText("All").assertCountEquals(1)
        compose.runOnIdle { assertEquals(1, bars.slots.size); assertEquals(0, openedCard) }
    }

    @Test fun searchWidensTheOnePillAndCarriesTheOneCircleToTheEnd() {
        val bars = FootBars()
        var searching by mutableStateOf(false)
        show(bars) {
            LibraryFootBar(Tab.Spaces, {}, searching, "", {}, { searching = true }, { searching = false })
        }
        val root = compose.onRoot().fetchSemanticsNode().boundsInRoot
        val (side, gap) = with(compose.density) { 12.dp.toPx() to 8.dp.toPx() }
        val pill = compose.onNodeWithTag(FOOT_PILL_TAG).fetchSemanticsNode().boundsInRoot
        val circle = compose.onNodeWithContentDescription("Search").fetchSemanticsNode().boundsInRoot
        assertEquals(pill.height, circle.height, 1f)
        assertEquals("The circle sits 8dp beside the pill", gap, circle.left - pill.right, 1f)
        assertEquals("The pill and circle are centred together", pill.left - root.left, root.right - circle.right, 2f)
        assertTrue("The navigation pill is only as wide as its two icons", pill.width < root.width / 2)
        compose.mainClock.autoAdvance = false
        compose.onNodeWithContentDescription("Search").performClick()
        // Partway through, the tabs and the field share the one pill, already wider, and the one circle already closes.
        compose.mainClock.advanceTimeBy(100)
        compose.onAllNodesWithTag(FOOT_PILL_TAG).assertCountEquals(1)
        compose.onAllNodesWithContentDescription("Search").assertCountEquals(0)
        val opening = compose.onNodeWithTag(FOOT_PILL_TAG).fetchSemanticsNode().boundsInRoot
        assertTrue("The pill widens", opening.width > pill.width + 1f)
        assertEquals(pill.top, opening.top, 1f)
        assertEquals(pill.bottom, opening.bottom, 1f)
        compose.mainClock.autoAdvance = true
        settle()
        val wide = compose.onNodeWithTag(FOOT_PILL_TAG).fetchSemanticsNode().boundsInRoot
        val close = compose.onNodeWithContentDescription("Close search").fetchSemanticsNode().boundsInRoot
        assertEquals(side, wide.left - root.left, 1f)
        assertEquals(side, root.right - close.right, 1f)
        assertEquals(gap, close.left - wide.right, 1f)
        assertEquals(circle.size, close.size)
        assertEquals(pill.bottom, wide.bottom, 1f)
        val field = compose.onNodeWithContentDescription("Search your inventory").fetchSemanticsNode().boundsInRoot
        assertTrue("The field sits inside the navigation's pill", wide.contains(field.center))
        compose.onNodeWithText("Spaces").assertDoesNotExist()
        compose.onNodeWithContentDescription("Close search").performClick()
        settle()
        compose.onNodeWithText("Spaces").assertIsSelected()
        assertEquals(pill, compose.onNodeWithTag(FOOT_PILL_TAG).fetchSemanticsNode().boundsInRoot)
        assertEquals(circle, compose.onNodeWithContentDescription("Search").fetchSemanticsNode().boundsInRoot)
        compose.runOnIdle { assertEquals(1, bars.slots.size) }
    }

    @Test fun compactPillTapsOpenItWithoutReachingThePageBehind() {
        val bars = FootBars()
        var expanded = 0
        var behind = 0
        show(bars) {
            Box(Modifier.fillMaxSize().clickable { behind++ })
            FootPill("compact", compact = "Ask", onExpand = { expanded++ }) { Text("Expanded field") }
        }
        val label = compose.onNodeWithText("Ask").fetchSemanticsNode().boundsInRoot
        // The compact pill is 104dp wide, with its word centred; tap the paper just beside the word.
        compose.onRoot().performTouchInput { click(Offset(label.left - 20f, label.center.y)) }
        compose.runOnIdle { assertEquals(1, expanded); assertEquals(0, behind) }
    }

    @Test fun tabsAreIconsWhoseNameShowsForAMomentAfterAChange() {
        var selected by mutableStateOf(Tab.Everything)
        show(FootBars()) { MymindTabBar(selected, onSelect = { selected = it }) }
        val target = with(compose.density) { 44.dp.toPx() }
        fun name(tab: Tab) = compose.onNodeWithText(tab.label, useUnmergedTree = true).fetchSemanticsNode().boundsInRoot
        fun pill() = compose.onNodeWithTag(FOOT_PILL_TAG).fetchSemanticsNode().boundsInRoot
        val rest = pill()
        // Arriving shows no name, only the two icons; each tab is still a full touch target.
        assertEquals(0f, name(Tab.Everything).width, 1f)
        assertEquals(0f, name(Tab.Spaces).width, 1f)
        for (tab in Tab.entries) {
            val bounds = compose.onNodeWithText(tab.label).fetchSemanticsNode().boundsInRoot
            assertTrue("${tab.label} is at least 44dp wide and tall", bounds.width >= target && bounds.height >= target)
        }
        compose.mainClock.autoAdvance = false
        compose.onNodeWithText("Spaces").performClick()
        compose.mainClock.advanceTimeBy(600)
        compose.onNodeWithText("Spaces").assertIsSelected()
        assertTrue("The chosen place's name shows", name(Tab.Spaces).width > target)
        assertEquals(0f, name(Tab.Everything).width, 1f)
        val named = pill()
        assertTrue("The pill grows with the name", named.width > rest.width + target)
        assertEquals("It grows on both sides, the pill and circle staying centred", rest.center.x, named.center.x, 1f)
        compose.mainClock.advanceTimeBy(1600)
        assertEquals("The name folds away after a moment", 0f, name(Tab.Spaces).width, 1f)
        assertEquals(rest.width, pill().width, 1f)
        compose.mainClock.autoAdvance = true
    }

    @Test fun navigationRespondsWithSystemAnimationsDisabled() {
        val resolver = ApplicationProvider.getApplicationContext<android.content.Context>().contentResolver
        val previous = Settings.Global.getFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f)
        Settings.Global.putFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, 0f)
        try {
            var selected by mutableStateOf(Tab.Everything)
            show(FootBars()) { MymindTabBar(selected, onSelect = { selected = it }) }
            compose.onNodeWithText("Spaces").performClick()
            compose.onNodeWithText("Spaces").assertIsSelected()
            compose.runOnIdle { assertTrue(selected == Tab.Spaces) }
            // Without animations the name is there at once rather than easing out.
            compose.mainClock.advanceTimeByFrame()
            val name = compose.onNodeWithText("Spaces", useUnmergedTree = true).fetchSemanticsNode().boundsInRoot
            assertTrue("The name shows at once", name.width > 0f)
        } finally {
            Settings.Global.putFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, previous)
        }
    }
}

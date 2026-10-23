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
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.test.core.app.ApplicationProvider
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
        compose.onNodeWithText("Everything").assertIsSelected()
        compose.onNodeWithText("Create").assertDoesNotExist()
        compose.onNodeWithText("Serendipity").assertDoesNotExist()
        compose.onNodeWithText("Spaces").performClick()
        settle()
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onAllNodesWithText("Everything").assertCountEquals(1)
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
        compose.onRoot().performTouchInput { click(Offset(47f, searchBounds.center.y)) }
        compose.runOnIdle { assertEquals(0, openedCard) }
        compose.onNodeWithContentDescription("Search").performClick()
        settle()
        val close = compose.onNodeWithContentDescription("Close search").fetchSemanticsNode().boundsInRoot
        assertEquals(before, close.bottom, 1f)
        val field = compose.onNodeWithContentDescription("Search your mind")
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
        compose.onNodeWithContentDescription("Search your mind").assertDoesNotExist()
        compose.onAllNodesWithText("Everything").assertCountEquals(1)
        compose.runOnIdle { assertEquals(1, bars.slots.size); assertEquals(0, openedCard) }
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

    @Test fun eachTabKeepsHalfTheCapsuleToTapWhileItsLabelSitsAtItsOuterEdge() {
        var selected by mutableStateOf(Tab.Everything)
        show(FootBars()) { MymindTabBar(selected, onSelect = { selected = it }) }
        val everything = compose.onNodeWithText("Everything").fetchSemanticsNode().boundsInRoot
        val spaces = compose.onNodeWithText("Spaces").fetchSemanticsNode().boundsInRoot
        val label = compose.onNodeWithText("Spaces", useUnmergedTree = true).fetchSemanticsNode().boundsInRoot
        assertEquals(everything.width, spaces.width, 1f)
        assertTrue("The last label sits at the end of its half", spaces.right - label.right < spaces.width / 3)
        compose.onRoot().performTouchInput { click(Offset(spaces.left + 6f, spaces.center.y)) }
        compose.runOnIdle { assertEquals(Tab.Spaces, selected) }
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
        } finally {
            Settings.Global.putFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, previous)
        }
    }
}

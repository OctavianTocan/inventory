package com.octaviantocan.mymind.ui

import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuContent
import com.octaviantocan.mymind.ui.menu.DELETE_HOLD_MS
import com.octaviantocan.mymind.ui.menu.InlineSpaces
import com.octaviantocan.mymind.ui.menu.SpaceChoices
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** Choose one, the card's sheet: its order, Add to space in place and Delete card held rather than tapped. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ChooseOneTest {
    @get:Rule val compose = createComposeRule()

    private val article = testCard("a", type = "Article", url = "https://example.com/a", title = "An article")
    private val deleteRow = SemanticsMatcher("Delete card row") { it.config.getOrElseNullable(SemanticsActions.OnClick) { null }?.label == "Delete card" }

    private fun top(text: String) = compose.onNodeWithText(text).fetchSemanticsNode().boundsInRoot.top

    @Test
    fun choicesLeadThenTilesTwoByTwoThenDeleteAlone() {
        compose.setPhoneContent(false) { CardMenuContent(cardMenuItems(article.copy(pinPosition = 1), null), onSelect = {}) }
        compose.mainClock.advanceTimeBy(500)
        assertTrue(top("Add to space") < top("Open source"))
        assertEquals(top("Open source"), top("Pin"), 1f)
        assertTrue(top("Pin") < top("Add tags"))
        assertTrue(top("Add tags") < top("Delete card"))
        compose.onNodeWithContentDescription("Pinned").assertExists()
        compose.onNodeWithText("Hold to delete").assertExists()
    }

    @Test
    fun deleteFillsWhileHeldKeepsTheCardWhenLetGoAndGoesOnceTheHoldCompletes() {
        val chosen = mutableListOf<CardMenuAction>()
        var held = 0f
        compose.setPhoneContent(false) { CardMenuContent(cardMenuItems(article, null), onSelect = { chosen += it.action }, onDeleteHold = { held = it }) }
        compose.mainClock.advanceTimeBy(500)
        val row = compose.onNode(deleteRow)
        row.performTouchInput { down(center) }
        compose.mainClock.advanceTimeBy(DELETE_HOLD_MS / 2)
        compose.onNodeWithText("Keep holding to delete").assertExists()
        assertTrue("Held halfway, not $held", held in 0.3f..0.7f)
        row.performTouchInput { up() }
        compose.mainClock.advanceTimeBy(100)
        compose.onNodeWithText("Delete card").assertExists()
        assertEquals(0f, held)
        assertTrue(chosen.isEmpty())

        row.performTouchInput { down(center) }
        compose.mainClock.advanceTimeBy(DELETE_HOLD_MS + 100)
        assertEquals(listOf(CardMenuAction.DeleteCard), chosen)
        row.performTouchInput { up() }
        row.performSemanticsAction(SemanticsActions.OnClick)
        assertEquals("Accessibility deletes with one action", listOf(CardMenuAction.DeleteCard, CardMenuAction.DeleteCard), chosen)
    }

    @Test
    fun addToSpaceShowsTheSuggestedSpaceFirstAndTogglesInPlace() {
        val spaces = listOf(Space("s1", "Reading", "#19aad1"), Space("s2", "Walks", "#17c37b"), Space("s3", "Later"), Space("s4", "Hidden"))
        val card = mutableStateOf(article.copy(spaces = listOf("s3")))
        val toggled = mutableListOf<String>()
        var all = 0
        compose.setPhoneContent(false) {
            InlineSpaces(SpaceChoices(spaces, card.value.spaces.orEmpty().toSet(), suggestion = SpaceSuggestion(spaces[1], 2)), card.value,
                onToggle = { toggled += it.id }, onAll = { all++ })
        }
        compose.mainClock.advanceTimeBy(500)
        compose.onNodeWithText("In Later").assertExists()
        compose.onNodeWithText("Suggested").assertExists()
        compose.onNode(hasContentDescription("Later")).assertIsSelected()
        compose.onNode(hasContentDescription("Hidden")).assertDoesNotExist()
        assertTrue(compose.onNode(hasContentDescription("Walks")).fetchSemanticsNode().boundsInRoot.left <
            compose.onNode(hasContentDescription("Later")).fetchSemanticsNode().boundsInRoot.left)
        compose.onNode(hasContentDescription("Walks")).performClick()
        compose.onNodeWithText("All spaces").performClick()
        assertEquals(listOf("s2"), toggled)
        assertEquals(1, all)
    }
}

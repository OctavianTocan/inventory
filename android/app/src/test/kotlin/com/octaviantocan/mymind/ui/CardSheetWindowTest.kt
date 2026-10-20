package com.octaviantocan.mymind.ui

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SheetState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.card.CardSheetDialog
import com.octaviantocan.mymind.ui.menu.CardMenuSheet
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class CardSheetWindowTest {
    @get:Rule val compose = createComposeRule()
    private val opened = mutableStateOf(true)
    private val actions = mutableStateOf(false)
    private var closed = 0
    @OptIn(ExperimentalMaterial3Api::class)
    private lateinit var menuState: SheetState
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "Card details"))

    @OptIn(ExperimentalMaterial3Api::class)
    private fun show() {
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                Box(Modifier.fillMaxSize())
                menuState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
                if (opened.value) CardSheetDialog(LibrarySamples.article, onClose = { closed++; opened.value = false }, onMenu = { actions.value = true }, actionsVisible = actions.value)
                if (actions.value) CardMenuSheet(cardMenuItems(LibrarySamples.article, null), onSelect = { actions.value = false }, onDismiss = { actions.value = false }, sheetState = menuState)
            }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    @Test
    fun crossAndActionsAreMatchingFloatingButtonsAndCrossAnimatesOut() {
        show()
        compose.onNodeWithContentDescription("Close card").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Close card").performClick()
        assertEquals(0, closed)
        compose.mainClock.advanceTimeBy(500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun actionsOpenAboveTheCardAndReturnToTheSameExpandedSheet() {
        show()
        // The two windows have independent layout passes before their entrance animations can run.
        compose.mainClock.autoAdvance = true
        panel.performSemanticsAction(SemanticsActions.Expand) { it() }
        compose.mainClock.advanceTimeBy(1500)
        val before = panel.fetchSemanticsNode().boundsInRoot.top
        compose.onNodeWithContentDescription("Card actions").performClick()
        compose.waitForIdle()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        compose.onNodeWithText("Add tags").assertIsDisplayed().performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(0, closed)
        assertEquals(before, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        compose.onNodeWithContentDescription("Close card").assertIsDisplayed()
    }
}

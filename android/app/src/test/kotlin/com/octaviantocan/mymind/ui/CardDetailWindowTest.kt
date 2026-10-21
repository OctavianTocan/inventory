package com.octaviantocan.mymind.ui

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SheetState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipeUp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.card.CardDetailDialog
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.menu.CardMenuSheet
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
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
class CardDetailWindowTest {
    @get:Rule val compose = createComposeRule()
    private val opened = mutableStateOf(true)
    private val actions = mutableStateOf(false)
    private var closed = 0
    private val scroll = ScrollState(0)
    @OptIn(ExperimentalMaterial3Api::class)
    private lateinit var menuState: SheetState
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "Card details"))

    @OptIn(ExperimentalMaterial3Api::class)
    private fun show(card: Card = LibrarySamples.article) {
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                Box(Modifier.fillMaxSize())
                menuState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
                if (opened.value) CardDetailDialog(card, onClose = { closed++; opened.value = false }, onMenu = { actions.value = true }, actionsVisible = actions.value, scrollState = scroll)
                if (actions.value) CardMenuSheet(cardMenuItems(card, null), onSelect = { actions.value = false }, onDismiss = { actions.value = false }, sheetState = menuState, accent = cardAccent(card))
            }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    @Test
    fun crossAndActionsAreMatchingFloatingButtonsAndCrossClosesTheReader() {
        show()
        compose.onNodeWithContentDescription("Close card").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Close card").performClick()
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    @Config(fontScale = 2f)
    fun exceptionallyLongTitlesRemainScrollableWithControlsAndBodyVisible() {
        val title = (1..20).joinToString("\n") { "Heading line $it" }
        show(LibrarySamples.article.copy(title = title))
        val header = compose.onNode(hasText(title) and hasScrollAction()).assertIsDisplayed()
        val body = compose.onNode(hasScrollAction() and !hasText(title)).assertIsDisplayed()
        assertTrue(header.fetchSemanticsNode().boundsInRoot.height <= PHONE_HEIGHT_PX / 4f + 1f)
        assertTrue(body.fetchSemanticsNode().boundsInRoot.height > with(compose.density) { 100.dp.toPx() })
        val range = header.fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange]
        assertTrue(range.maxValue() > 0f)
        val readerTop = panel.fetchSemanticsNode().boundsInRoot.top
        header.performTouchInput { swipeUp(durationMillis = 500) }
        compose.mainClock.advanceTimeBy(1500)
        assertTrue("A swipe scrolls the heading", range.value() > 0f)
        assertEquals(readerTop, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        assertEquals(0f, body.fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value(), 1f)
        header.performSemanticsAction(SemanticsActions.ScrollBy) { it(0f, range.maxValue()) }
        compose.mainClock.advanceTimeBy(500)
        compose.waitForIdle()
        assertEquals(range.maxValue(), range.value(), 1f)
        compose.onNodeWithContentDescription("Close card").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
    }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun actionsOpenAboveTheReaderAndReturnToTheSameScrollPosition() {
        show()
        // The two windows have independent layout passes before their entrance animations can run.
        compose.mainClock.autoAdvance = true
        val body = compose.onNode(hasScrollAction() and !hasText(LibrarySamples.article.title!!))
        body.performSemanticsAction(SemanticsActions.ScrollBy) { it(0f, 200f) }
        compose.mainClock.advanceTimeBy(500)
        val before = scroll.value
        assertTrue(before > 0)
        compose.onNodeWithContentDescription("Card actions").performClick()
        compose.waitForIdle()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        compose.onNodeWithText("Add tags").assertIsDisplayed().performClick()
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(0, closed)
        assertEquals(before, scroll.value)
        assertEquals(0f, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        compose.onNodeWithContentDescription("Close card").assertIsDisplayed()
    }
}

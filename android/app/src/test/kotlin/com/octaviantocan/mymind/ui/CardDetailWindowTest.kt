package com.octaviantocan.mymind.ui

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.key
import com.octaviantocan.mymind.ui.card.CardSheetPosition
import com.octaviantocan.mymind.ui.menu.rememberCardMenuPosition
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
    private lateinit var menuState: CardSheetPosition
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "Card details"))

    private fun show(card: Card = LibrarySamples.article) {
        compose.setPhoneContent(false) {
            PageTheme(PageScheme.Dark) {
                Box(Modifier.fillMaxSize())
                menuState = key(actions.value) { rememberCardMenuPosition() }
                if (opened.value) CardDetailDialog(card, onClose = { closed++; opened.value = false }, onMenu = { actions.value = true }, actionsProgress = if (actions.value) menuState.scrim else 0f, scrollState = scroll)
                if (actions.value) CardMenuSheet(cardMenuItems(card, null), onSelect = { actions.value = false }, onDismiss = { actions.value = false }, position = menuState, accent = cardAccent(card))
            }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    @Test
    fun crossAndActionsAreMatchingFloatingButtonsAndCrossClosesTheReader() {
        show()
        compose.onNodeWithContentDescription("Back").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Back").performClick()
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    @Config(fontScale = 2f)
    fun exceptionallyLongTitlesUseOneRowWhileTheBodyAndControlsStayVisible() {
        val title = (1..20).joinToString("\n") { "Heading line $it" }
        show(LibrarySamples.article.copy(title = title))
        val header = compose.onNode(hasText(title) and !hasScrollAction()).assertIsDisplayed()
        val body = compose.onNode(hasScrollAction() and !hasText(title)).assertIsDisplayed()
        val layouts = mutableListOf<androidx.compose.ui.text.TextLayoutResult>()
        header.performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(layouts) }
        assertEquals("The complete title remains accessible in a one-row pill", 1, layouts.single().lineCount)
        val headerBounds = header.fetchSemanticsNode().boundsInRoot
        assertTrue(body.fetchSemanticsNode().boundsInRoot.height > with(compose.density) { 100.dp.toPx() })
        body.performSemanticsAction(SemanticsActions.ScrollBy) { it(0f, 200f) }
        compose.mainClock.advanceTimeBy(500)
        compose.waitForIdle()
        assertEquals("Reading scrolls independently of its header", headerBounds, header.fetchSemanticsNode().boundsInRoot)
        compose.onNodeWithContentDescription("Back").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        compose.onNodeWithContentDescription("Card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
    }

    @Test
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
        compose.onNodeWithContentDescription("Back").assertIsDisplayed()
    }
}

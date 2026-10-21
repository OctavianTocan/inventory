package com.octaviantocan.mymind.ui

import androidx.activity.OnBackPressedDispatcher
import androidx.activity.compose.LocalOnBackPressedDispatcherOwner
import androidx.compose.foundation.ScrollState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipe
import androidx.compose.ui.test.swipeDown
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import kotlinx.coroutines.runBlocking
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
class CardDetailInteractionTest {
    @get:Rule val compose = createComposeRule()
    private val opened = mutableStateOf(true)
    private var closed = 0
    private val scroll = ScrollState(0)
    private lateinit var back: OnBackPressedDispatcher
    private val panel get() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "Card details"))

    private fun open() {
        compose.setPhoneContent(false) {
            back = checkNotNull(LocalOnBackPressedDispatcherOwner.current).onBackPressedDispatcher
            PageTheme(PageScheme.Dark) {
                if (opened.value) CardDetail(
                    LibrarySamples.article.copy(description = "Long synthetic card content.\n".repeat(100)),
                    onClose = { closed++; opened.value = false },
                    onMenu = {},
                    headerIcons = flatSizedCardHeaderIcons(),
                    scrollState = scroll,
                )
            }
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
    }

    @Test
    fun readerUsesTheFullWindowWithoutSheetStops() {
        open()
        val node = panel.fetchSemanticsNode()
        assertEquals(0f, node.boundsInRoot.top, 2f)
        assertEquals(PHONE_HEIGHT_PX.toFloat(), node.boundsInRoot.bottom, 2f)
        assertTrue(SemanticsActions.Expand !in node.config)
        assertTrue(SemanticsActions.Collapse !in node.config)
        assertEquals(0, closed)
    }

    @Test
    fun accessibilityDismissClosesTheCard() {
        open()
        panel.performSemanticsAction(SemanticsActions.Dismiss) { it() }
        compose.change {}
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    fun pullingDownOnTheHeaderDoesNotMoveOrDismissTheReader() {
        open()
        panel.performTouchInput {
            swipe(Offset(centerX, 30f), Offset(centerX, 600f), durationMillis = 300)
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(0, closed)
        assertEquals(0f, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        compose.onNodeWithContentDescription("Close card").assertIsDisplayed()
    }

    @Test
    fun scrollingContentBackToItsTopKeepsTheReaderInPlace() {
        open()
        val body = compose.onNode(hasScrollAction() and !hasText(LibrarySamples.article.title!!))
        compose.runOnIdle { runBlocking { scroll.scrollTo(100) } }
        body.performSemanticsAction(SemanticsActions.ScrollBy) { it(0f, -1000f) }
        compose.mainClock.advanceTimeBy(1500)
        body.performTouchInput { swipeDown(durationMillis = 500) }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(0, scroll.value)
        assertEquals(0f, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        assertEquals(0, closed)
    }

    @Test
    fun theReaderShowsOneTitle() {
        open()
        compose.onAllNodesWithText(LibrarySamples.article.title!!).assertCountEquals(1)
    }

    @Test
    fun systemBackClosesTheCard() {
        open()
        compose.change { back.onBackPressed() }
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }
}

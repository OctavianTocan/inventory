package com.octaviantocan.mymind.ui

import androidx.activity.OnBackPressedDispatcher
import androidx.activity.compose.LocalOnBackPressedDispatcherOwner
import androidx.compose.foundation.ScrollState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.click
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipe
import com.octaviantocan.mymind.ui.card.CardSheet
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
class CardSheetInteractionTest {
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
                if (opened.value) CardSheet(
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
    fun opensAtHalfHeightAndCanExpandAndCollapse() {
        open()
        val normal = panel.fetchSemanticsNode().boundsInRoot.top
        assertTrue(normal > PHONE_HEIGHT_PX * 0.4f)
        assertTrue(normal < PHONE_HEIGHT_PX * 0.65f)
        panel.performSemanticsAction(SemanticsActions.Expand) { it() }
        compose.mainClock.advanceTimeBy(1500)
        assertTrue(panel.fetchSemanticsNode().boundsInRoot.top < normal / 2f)
        panel.performSemanticsAction(SemanticsActions.Collapse) { it() }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(normal, panel.fetchSemanticsNode().boundsInRoot.top, 2f)
        assertEquals(0, closed)
    }

    @Test
    fun dismissNotifiesTheShellOnlyAfterAnimatingOutAndOnlyOnce() {
        open()
        panel.performSemanticsAction(SemanticsActions.Dismiss) { it() }
        panel.performSemanticsAction(SemanticsActions.Dismiss) { it() }
        assertEquals(0, closed)
        compose.mainClock.advanceTimeBy(500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    fun swipingTheGrabberDownDismissesTheCard() {
        open()
        panel.performTouchInput {
            swipe(Offset(centerX, 30f), Offset(centerX, 600f), durationMillis = 300)
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    fun scrollingContentToItsTopDoesNotDragTheSheetUntilTheNextSwipe() {
        open()
        panel.performSemanticsAction(SemanticsActions.Expand) { it() }
        compose.mainClock.advanceTimeBy(1500)
        val expanded = panel.fetchSemanticsNode().boundsInRoot.top
        compose.runOnIdle { runBlocking { scroll.scrollTo(100) } }
        panel.performTouchInput {
            swipe(Offset(centerX, 500f), Offset(centerX, 1000f), durationMillis = 1000)
        }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(0, scroll.value)
        assertEquals(expanded, panel.fetchSemanticsNode().boundsInRoot.top, 2f)

        panel.performTouchInput {
            swipe(Offset(centerX, 500f), Offset(centerX, 1200f), durationMillis = 500)
        }
        compose.mainClock.advanceTimeBy(1500)
        assertTrue(panel.fetchSemanticsNode().boundsInRoot.top > expanded + 200f)
        assertEquals(0, closed)
    }

    @Test
    fun tappingOutsideSlidesTheCardOut() {
        open()
        compose.onRoot().performTouchInput { click(Offset(centerX, 300f)) }
        assertEquals(0, closed)
        compose.mainClock.advanceTimeBy(500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }

    @Test
    fun systemBackSlidesTheCardOut() {
        open()
        compose.runOnUiThread { back.onBackPressed() }
        assertEquals(0, closed)
        compose.mainClock.advanceTimeBy(500)
        compose.waitForIdle()
        assertEquals(1, closed)
        panel.assertDoesNotExist()
    }
}

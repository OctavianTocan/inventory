package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.card.CardHeader
import com.octaviantocan.mymind.ui.card.ReaderHeader
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class CardHeaderLayoutTest(private val title: String, private val fontScale: Float, private val hasMenu: Boolean) {
    @get:Rule val compose = createComposeRule()

    @Test
    fun fullTitleWrapsWithoutCrowdingEitherControl() {
        compose.setPhoneContent(false) {
            val density = LocalDensity.current.density
            CompositionLocalProvider(LocalDensity provides Density(density, fontScale)) {
                Box(Modifier.width(320.dp).testTag("header")) {
                    CardHeader(title, onClose = {}, onMenu = if (hasMenu) ({}) else null)
                }
            }
        }
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        val text = compose.onNodeWithText(title).assertIsDisplayed()
        val layouts = mutableListOf<TextLayoutResult>()
        text.performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(layouts) }
        val layout = layouts.single()
        assertFalse("The title must not clip or ellipsize: size=${layout.size}, paragraph=${layout.multiParagraph.width}x${layout.multiParagraph.height}, widthOverflow=${layout.didOverflowWidth}, heightOverflow=${layout.didOverflowHeight}", layout.hasVisualOverflow)
        assertTrue("Long titles must wrap", layout.lineCount > 1)
        assertEquals(title.length, layout.getLineEnd(layout.lineCount - 1))

        val titleBounds = text.fetchSemanticsNode().boundsInRoot
        val gap = with(compose.density) { 8.dp.toPx() }
        val close = compose.onNodeWithContentDescription("Close card").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
        val closeBounds = close.fetchSemanticsNode().boundsInRoot
        val firstLineCenter = titleBounds.top + (layout.getLineTop(0) + layout.getLineBottom(0)) / 2f
        assertEquals("Align the buttons with the title's first line", firstLineCenter, closeBounds.center.y, 1f)
        assertTrue("Leave space between the cross and title", titleBounds.left >= closeBounds.right + gap - 1f)
        if (hasMenu) {
            val menu = compose.onNodeWithContentDescription("Card actions").assertIsDisplayed().assertWidthIsEqualTo(44.dp)
            assertTrue("Leave space between the title and menu", titleBounds.right <= menu.fetchSemanticsNode().boundsInRoot.left - gap + 1f)
        }
        assertEquals("Keep the title centered with or without a menu", compose.onNodeWithTag("header").fetchSemanticsNode().boundsInRoot.center.x, titleBounds.center.x, 1f)
    }

    @Test
    fun readerTitleUsesOneCenteredRowWithSeparateControls() {
        compose.setPhoneContent(false) {
            val density = LocalDensity.current.density
            CompositionLocalProvider(LocalDensity provides Density(density, fontScale)) {
                Box(Modifier.width(320.dp).testTag("header")) {
                    ReaderHeader(title, open = false, onBack = {}, onName = {}, onShare = if (hasMenu) ({}) else null)
                }
            }
        }
        compose.mainClock.advanceTimeBy(100)
        compose.waitForIdle()
        val text = compose.onNodeWithText(title).assertIsDisplayed()
        val layouts = mutableListOf<TextLayoutResult>()
        text.performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(layouts) }
        assertEquals("The reader title stays on one row", 1, layouts.single().lineCount)
        val bounds = text.fetchSemanticsNode().boundsInRoot
        val header = compose.onNodeWithTag("header").fetchSemanticsNode().boundsInRoot
        val close = compose.onNodeWithContentDescription("Back").fetchSemanticsNode().boundsInRoot
        assertEquals(header.center.x, bounds.center.x, 1f)
        assertEquals(bounds.center.y, close.center.y, 1f)
        assertTrue(bounds.left > close.right)
        if (hasMenu) {
            val share = compose.onNodeWithContentDescription("Share").fetchSemanticsNode().boundsInRoot
            assertEquals(bounds.center.y, share.center.y, 1f)
            assertTrue(bounds.right < share.left)
        }
    }

    companion object {
        @JvmStatic
        @ParameterizedRobolectricTestRunner.Parameters(name = "fontScale={1}, menu={2}, title={0}")
        fun cases() = listOf(
            arrayOf<Any>("An article opened in the card view", 1.08f, true),
            arrayOf<Any>("An article opened in the card view", 2f, true),
            arrayOf<Any>("An article opened in the card view", 2f, false),
            arrayOf<Any>("AReallyLongUnbrokenTitleThatStillNeedsToBeReadable", 1.08f, true),
            arrayOf<Any>("An article opened\nin the card view", 1.08f, true),
        )
    }
}

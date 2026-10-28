package com.octaviantocan.mymind.ui.library

import android.content.Context
import android.provider.Settings
import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.click
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.swipe
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.toSize
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.theme.MymindTheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config

@RunWith(AndroidJUnit4::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class LibraryMosaicTest {
    @get:Rule val compose = createComposeRule()
    private val cards = (1..70).map { testCard("m$it", type = "Image", title = "Photo $it") }

    private fun pinch(inward: Boolean, yFraction: Float = 0.48f) {
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            val begin = width * if (inward) 0.32f else 0.14f
            val end = width * if (inward) 0.14f else 0.32f
            val y = height * yFraction
            down(0, Offset(centerX - begin, y))
            down(1, Offset(centerX + begin, y))
            moveTo(0, Offset(centerX - end, y), delayMillis = 120)
            moveTo(1, Offset(centerX + end, y), delayMillis = 120)
            up(0)
            up(1)
        }
        compose.waitForIdle()
    }

    @Test
    fun cardsKeepFollowingTheFingersAcrossFramesBeforeRelease() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density)
            }
        }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val before = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        val bounds = gallery.fetchSemanticsNode().boundsInRoot
        val focal = Offset(before.center.x - bounds.left, before.center.y - bounds.top)
        gallery.performTouchInput {
            down(0, focal - Offset(width * 0.15f, 0f))
            down(1, focal + Offset(width * 0.15f, 0f))
            moveTo(0, focal - Offset(width * 0.11f, 0f), delayMillis = 32)
            moveTo(1, focal + Offset(width * 0.11f, 0f), delayMillis = 32)
        }
        compose.waitForIdle()
        val middle = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        gallery.performTouchInput {
            moveTo(0, focal - Offset(width * 0.065f, 0f), delayMillis = 32)
            moveTo(1, focal + Offset(width * 0.065f, 0f), delayMillis = 32)
        }
        compose.waitForIdle()
        val smaller = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        assertTrue("A later pointer frame must keep shrinking the same live card", smaller.width < middle.width - 10f)
        assertTrue("The focal card must stay at the fingers while down", kotlin.math.abs(smaller.center.y - before.center.y) < 4f)
        gallery.performTouchInput { up(0); up(1) }
    }

    @Test
    fun aPinchBetweenStopsNeverLeavesEmptyMarginsOrPansTheGallery() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val focal = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.center - root.topLeft
        fun coverage(): Pair<Float, Float> {
            val shown = cards.indices.mapNotNull { index ->
                compose.onAllNodesWithContentDescription("Photo ${index + 1}").fetchSemanticsNodes().firstOrNull()?.boundsInRoot
            }.filter { it.bottom > root.top && it.top < root.bottom }
            return shown.minOf { it.left } to shown.maxOf { it.right }
        }
        for (spread in listOf(0.26f, 0.09f)) {
            gallery.performTouchInput {
                down(0, focal - Offset(width * 0.15f, 0f))
                down(1, focal + Offset(width * 0.15f, 0f))
                updatePointerTo(0, focal - Offset(width * spread, 0f))
                updatePointerTo(1, focal + Offset(width * spread, 0f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
            assertTrue("The pinch must be between stops", density.position % 1f != 0f)
            val (left, right) = coverage()
            assertTrue("No empty margin on the left at ${density.position}: $left", left <= root.left + 1f)
            assertTrue("No empty margin on the right at ${density.position}: $right", right >= root.right - 1f)
            val before = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.center
            gallery.performTouchInput {
                updatePointerTo(0, focal - Offset(width * spread, -height * 0.1f))
                updatePointerTo(1, focal + Offset(width * spread, height * 0.1f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
            val after = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.center
            assertEquals("Moving both fingers together must not pan the gallery", before.y, after.y, 2f)
            gallery.performTouchInput { up(0); up(1) }
            compose.waitForIdle()
            density.larger(); density.smaller()
            compose.waitForIdle()
        }
    }

    @Test
    fun diagonalPinchAnchorsTheActualTwoPointerCentroidAndLeavesTheHeaderUnscaled() {
        val density = MosaicState(2)
        val state = LazyListState()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density,
                    header = { Box(Modifier.height(80.dp).testTag("fixed-header")) })
            }
        }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val headerHeight = compose.onNodeWithTag("fixed-header").fetchSemanticsNode().boundsInRoot.height
        compose.runOnIdle { state.requestScrollToItem(9) }
        compose.waitForIdle()
        val before = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val focal = before.center - root.topLeft
        gallery.performTouchInput {
            down(0, focal - Offset(width * 0.12f, height * 0.08f))
            down(1, focal + Offset(width * 0.12f, height * 0.08f))
        }
        gallery.performTouchInput {
            updatePointerTo(0, focal - Offset(width * 0.096f, height * 0.064f))
            updatePointerTo(1, focal + Offset(width * 0.096f, height * 0.064f))
            move(delayMillis = 16)
        }
        compose.waitForIdle()
        val during = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        assertTrue(during.width < before.width - 10f)
        assertEquals("The centroid, not the first finger, anchors the card", before.center.y, during.center.y, 3f)
        assertTrue(focal.x + root.left in during.left..during.right)
        gallery.performTouchInput { up(0); up(1) }
        compose.waitForIdle()
        val after = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        assertEquals(before.center.y, after.center.y, 3f)
        compose.runOnIdle { state.requestScrollToItem(0) }
        compose.waitForIdle()
        assertEquals(headerHeight, compose.onNodeWithTag("fixed-header").fetchSemanticsNode().boundsInRoot.height, 1f)
    }

    @Test
    fun overshootingTheDensityLimitRespondsToTheFirstReversedPointerFrame() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        val squareCards = cards.map { it.copy(cover = Cover(width = 1000, height = 1000)) }
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(squareCards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val focal = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.center - root.topLeft
        gallery.performTouchInput {
            down(0, focal - Offset(width * 0.2f, 0f))
            down(1, focal + Offset(width * 0.2f, 0f))
            updatePointerTo(0, focal - Offset(width * 0.015f, 0f))
            updatePointerTo(1, focal + Offset(width * 0.015f, 0f))
            move(delayMillis = 16)
        }
        compose.waitForIdle()
        assertEquals(4f, density.position, 0f)
        val smallest = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.width
        gallery.performTouchInput {
            updatePointerTo(0, focal - Offset(width * 0.018f, 0f))
            updatePointerTo(1, focal + Offset(width * 0.018f, 0f))
            move(delayMillis = 16)
        }
        compose.waitForIdle()
        assertTrue("Reversal must move before either finger is lifted", density.position < 3.5f)
        val reversed = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.width
        assertTrue("The live card must enlarge on reversal: $smallest -> $reversed at ${density.position}", reversed > smallest + 5)
        gallery.performTouchInput { up(0); up(1) }
    }

    @Test
    fun cancellationAndTheOneFingerTailDoNotActivateCardsAndScrollingStillFlings() {
        val density = MosaicState(2)
        val state = LazyListState()
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        var refreshes = 0
        compose.setContent {
            MymindTheme(dark = true) {
                PullRefresh(false, { refreshes++ }) {
                    LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, state = state,
                        mosaic = density, onLongPressCard = { held += it.id })
                }
            }
        }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        gallery.performTouchInput {
            down(0, Offset(width * 0.15f, height * 0.35f))
            down(1, Offset(width * 0.85f, height * 0.35f))
            updatePointerTo(0, Offset(width * 0.25f, height * 0.35f))
            updatePointerTo(1, Offset(width * 0.75f, height * 0.35f))
            move(delayMillis = 16)
            cancel()
        }
        compose.waitForIdle()
        assertTrue(!density.isPinching)
        gallery.performTouchInput {
            down(0, Offset(width * 0.15f, height * 0.35f))
            down(1, Offset(width * 0.85f, height * 0.35f))
            updatePointerTo(0, Offset(width * 0.28f, height * 0.35f))
            updatePointerTo(1, Offset(width * 0.72f, height * 0.35f))
            move(delayMillis = 16)
            up(0)
            moveTo(1, Offset(width * 0.72f, height * 0.7f), delayMillis = 700)
            up(1)
        }
        compose.waitForIdle()
        assertTrue(opened.isEmpty() && held.isEmpty())
        assertEquals(0, refreshes)
        gallery.performTouchInput { swipe(Offset(centerX, height * 0.8f), Offset(centerX, height * 0.2f), durationMillis = 200) }
        compose.waitForIdle()
        assertTrue("The real scroll state must still receive a one-finger fling", state.firstVisibleItemIndex > 0)
        assertTrue(opened.isEmpty() && held.isEmpty())
    }

    @Test
    fun reducedMotionStillTracksTheFingersThenNormalizesTheFinalGeometry() {
        val resolver = ApplicationProvider.getApplicationContext<Context>().contentResolver
        Settings.Global.putFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, 0f)
        try {
            val density = MosaicState(2)
            compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, mosaic = density) } }
            val gallery = compose.onNodeWithContentDescription("Card gallery")
            val before = compose.onNodeWithContentDescription("Photo 1").fetchSemanticsNode().boundsInRoot.width
            gallery.performTouchInput {
                down(0, Offset(width * 0.1f, height * 0.25f))
                down(1, Offset(width * 0.9f, height * 0.25f))
                updatePointerTo(0, Offset(width * 0.2f, height * 0.25f))
                updatePointerTo(1, Offset(width * 0.8f, height * 0.25f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
            assertTrue(compose.onNodeWithContentDescription("Photo 1").fetchSemanticsNode().boundsInRoot.width < before)
            assertEquals(heldPosition(2.6667f), density.position, 0.01f)
            gallery.performTouchInput { up(0); up(1) }
            compose.waitForIdle()
            assertEquals(3, density.columns)
            assertEquals(3f, density.position, 0f)
            assertEquals(3, density.referenceColumns)
        } finally {
            Settings.Global.putFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f)
        }
    }

    @Test
    fun anotherPinchCanInterruptSettlingWithoutResettingItsStartingGeometry() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 6)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        compose.mainClock.autoAdvance = false
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.3f))
            down(1, Offset(width * 0.9f, height * 0.3f))
            updatePointerTo(0, Offset(width * 0.18f, height * 0.3f))
            updatePointerTo(1, Offset(width * 0.82f, height * 0.3f))
            move(delayMillis = 16)
            up(0); up(1)
        }
        compose.mainClock.advanceTimeByFrame()
        val before = density.position
        gallery.performTouchInput {
            down(0, Offset(width * 0.2f, height * 0.3f))
            down(1, Offset(width * 0.8f, height * 0.3f))
        }
        compose.mainClock.advanceTimeByFrame()
        assertEquals(before, density.position, 0.05f)
        gallery.performTouchInput {
            updatePointerTo(0, Offset(width * 0.1f, height * 0.3f))
            updatePointerTo(1, Offset(width * 0.9f, height * 0.3f))
            move(delayMillis = 16)
        }
        compose.mainClock.advanceTimeByFrame()
        assertTrue(density.position < before - 0.2f)
        gallery.performTouchInput { up(0); up(1) }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
        assertTrue(!density.isPinching)
        assertEquals(density.columns.toFloat(), density.position, 0f)
    }

    @Test
    fun crossingAStopGlidesEachCardToItsNewPlaceInsteadOfSwappingGrids() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        fun shown(): Map<String, androidx.compose.ui.geometry.Rect> = (10..36).mapNotNull { n ->
            compose.onAllNodesWithContentDescription("Photo $n").fetchSemanticsNodes().also { assertTrue("Photo $n is drawn once", it.size <= 1) }
                .firstOrNull()?.let { "Photo $n" to androidx.compose.ui.geometry.Rect(it.positionInRoot, it.size.toSize()) }
        }.toMap()
        compose.mainClock.autoAdvance = false
        try {
            gallery.performTouchInput {
                down(0, Offset(width * 0.1f, height * 0.3f))
                down(1, Offset(width * 0.9f, height * 0.3f))
                updatePointerTo(0, Offset(width * 0.19f, height * 0.3f))
                updatePointerTo(1, Offset(width * 0.81f, height * 0.3f))
                move(delayMillis = 16)
            }
            compose.mainClock.advanceTimeByFrame()
            compose.mainClock.advanceTimeByFrame()
            val start = shown()
            compose.mainClock.advanceTimeBy(100)
            val middle = shown()
            compose.mainClock.advanceTimeBy(400)
            val end = shown()
            val moved = start.keys.filter { it in middle && it in end && (end.getValue(it).topLeft - start.getValue(it).topLeft).getDistance() > 40f }
            assertTrue("Crossing a stop must move cards to new places", moved.isNotEmpty())
            for (name in moved) {
                val (a, m, b) = Triple(start.getValue(name), middle.getValue(name), end.getValue(name))
                assertTrue("$name must be on its way midway", (m.topLeft - a.topLeft).getDistance() > 4f && (b.topLeft - m.topLeft).getDistance() > 4f)
                assertTrue("$name must lie between its two places midway",
                    m.left in minOf(a.left, b.left) - 1f..maxOf(a.left, b.left) + 1f && m.top in minOf(a.top, b.top) - 1f..maxOf(a.top, b.top) + 1f)
            }
            gallery.performTouchInput { up(0); up(1) }
        } finally {
            compose.mainClock.autoAdvance = true
            compose.waitForIdle()
        }
    }

    @Test
    fun aNoteZoomsWithoutRewrappingBetweenStops() {
        val body = "Send your landing page to five people who have never heard of you and ask them what you sell. " +
            "If they cannot say it in one sentence, the page is not done."
        val notes = (1..40).map { testCard("note-$it", type = "Note", title = "Note $it", note = if (it == 6) body else "A short note, number $it.") }
        val density = MosaicState(4)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(notes, endReached = true), {}, {}, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        fun lines(): List<Int> {
            val results = mutableListOf<TextLayoutResult>()
            compose.onNodeWithText(body, useUnmergedTree = true).performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(results) }
            val layout = results.single()
            return (0 until layout.lineCount).map { layout.getLineEnd(it, visibleEnd = true) }
        }
        fun bounds() = compose.onNodeWithContentDescription("Note 6").fetchSemanticsNode().boundsInRoot
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val focal = bounds().center - root.topLeft
        val before = lines()
        val width = bounds().width
        gallery.performTouchInput {
            down(0, focal - Offset(this.width * 0.1f, 0f))
            down(1, focal + Offset(this.width * 0.1f, 0f))
        }
        val widths = mutableListOf<Float>()
        for (step in 1..6) {
            gallery.performTouchInput {
                updatePointerTo(0, focal - Offset(this.width * (0.1f + step * 0.004f), 0f))
                updatePointerTo(1, focal + Offset(this.width * (0.1f + step * 0.004f), 0f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
            assertTrue("The pinch must stay between 4 and 3 columns: ${density.position}", density.position > 3f && density.position < 4f)
            assertEquals("The note's words must keep their lines while it grows", before, lines())
            widths += bounds().width
        }
        assertTrue("The note must grow with the fingers: $width -> $widths", widths.zipWithNext().all { (a, b) -> b > a } && widths.last() > width + 8f)
        gallery.performTouchInput { up(0); up(1) }
        compose.waitForIdle()
    }

    @Test
    fun aLowOpacityIncomingGridDoesNotInterceptTheDominantCardsTapDuringSettle() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        val opened = mutableListOf<String>()
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        compose.mainClock.autoAdvance = false
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.4f))
            down(1, Offset(width * 0.9f, height * 0.4f))
            updatePointerTo(0, Offset(width * 0.19f, height * 0.4f))
            updatePointerTo(1, Offset(width * 0.81f, height * 0.4f))
            move(delayMillis = 16)
            up(0); up(1)
        }
        compose.mainClock.advanceTimeByFrame()
        assertTrue(density.position in heldPosition(2.55f)..heldPosition(2.65f))
        compose.onNodeWithContentDescription("Photo 20").performTouchInput { click() }
        compose.runOnIdle { assertEquals(listOf("m20"), opened) }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
    }

    @Test
    fun nonfocalTapPressSurvivesTheProjectionHandoff() = pressAfterPinch(20, longPress = false)

    @Test
    fun focalTapPressSurvivesTheProjectionHandoff() = pressAfterPinch(19, longPress = false)

    @Test
    fun nonfocalLongPressSurvivesTheProjectionHandoff() = pressAfterPinch(20, longPress = true)

    @Test
    fun focalLongPressSurvivesTheProjectionHandoff() = pressAfterPinch(19, longPress = true)

    private fun pressAfterPinch(number: Int, longPress: Boolean) {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id },
                    state = state, mosaic = density, onLongPressCard = { held += it.id })
            }
        }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val focal = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.center - root.topLeft
        compose.mainClock.autoAdvance = false
        try {
            gallery.performTouchInput {
                down(0, focal - Offset(width * 0.2f, 0f))
                down(1, focal + Offset(width * 0.2f, 0f))
                updatePointerTo(0, focal - Offset(width * 0.155f, 0f))
                updatePointerTo(1, focal + Offset(width * 0.155f, 0f))
                move(delayMillis = 16)
                up(0); up(1)
            }
            compose.mainClock.advanceTimeByFrame()
            assertTrue(density.position in heldPosition(2.55f)..heldPosition(2.65f))
            val point = compose.onNodeWithContentDescription("Photo $number").fetchSemanticsNode().boundsInRoot.center - root.topLeft
            gallery.performTouchInput { down(0, point) }
            val duration = if (longPress) 700L else 128L
            compose.mainClock.advanceTimeBy(duration)
            gallery.performTouchInput { advanceEventTime(duration); up(0) }
            compose.runOnIdle {
                assertEquals(if (longPress) emptyList<String>() else listOf("m$number"), opened)
                assertEquals(if (longPress) listOf("m$number") else emptyList<String>(), held)
            }
        } finally {
            compose.mainClock.autoAdvance = true
            compose.waitForIdle()
        }
    }

    @Test
    fun aPostPinchPressCanBecomeANormalDragWithoutKeepingTheFocalAnchorLocked() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, state = state,
                    mosaic = density, onLongPressCard = { held += it.id })
            }
        }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val focal = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot.center - root.topLeft
        compose.mainClock.autoAdvance = false
        try {
            gallery.performTouchInput {
                down(0, focal - Offset(width * 0.2f, 0f))
                down(1, focal + Offset(width * 0.2f, 0f))
                updatePointerTo(0, focal - Offset(width * 0.155f, 0f))
                updatePointerTo(1, focal + Offset(width * 0.155f, 0f))
                move(delayMillis = 16)
                up(0); up(1)
            }
            compose.mainClock.advanceTimeByFrame()
            val before = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
            val point = before.center - root.topLeft
            gallery.performTouchInput {
                down(0, point)
                moveTo(0, point - Offset(0f, 250f), delayMillis = 64)
            }
            compose.mainClock.advanceTimeByFrame()
            val during = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
            assertTrue("Passing drag slop must release focal anchoring while the finger stays down", during.center.y < before.center.y - 100f)
            gallery.performTouchInput { up(0) }
        } finally {
            compose.mainClock.autoAdvance = true
            compose.waitForIdle()
        }
        assertTrue(opened.isEmpty() && held.isEmpty())
        assertEquals(density.columns.toFloat(), density.position, 0f)
    }

    @Test
    fun projectedOverscanCannotInterceptTheUnscaledHeaderDuringSettle() {
        val density = MosaicState(2)
        var headerClicks = 0
        val opened = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, mosaic = density,
                    header = { Box(Modifier.fillMaxWidth().height(80.dp).testTag("header-action").clickable { headerClicks++ }) })
            }
        }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        compose.mainClock.autoAdvance = false
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.4f))
            down(1, Offset(width * 0.9f, height * 0.4f))
            updatePointerTo(0, Offset(width * 0.18f, height * 0.4f))
            updatePointerTo(1, Offset(width * 0.82f, height * 0.4f))
            move(delayMillis = 16)
            up(0); up(1)
        }
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithTag("header-action").performTouchInput { click() }
        compose.runOnIdle { assertEquals(1, headerClicks); assertTrue(opened.isEmpty()) }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
    }

    @Test
    fun projectedOverscanCannotInterceptTheRealRetryFooterDuringSettle() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 35)
        var retries = 0
        val opened = mutableListOf<String>()
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, error = "Offline"), { retries++ }, { opened += it.id }, state = state, mosaic = density) } }
        compose.onNodeWithText("Retry").assertIsDisplayed()
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        compose.mainClock.autoAdvance = false
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.3f))
            down(1, Offset(width * 0.9f, height * 0.3f))
            updatePointerTo(0, Offset(width * 0.18f, height * 0.3f))
            updatePointerTo(1, Offset(width * 0.82f, height * 0.3f))
            move(delayMillis = 16)
            up(0); up(1)
        }
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithText("Retry").performTouchInput { click() }
        compose.runOnIdle { assertEquals(1, retries); assertTrue(opened.isEmpty()) }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
    }

    @Test
    fun removingAndRestoringTheGalleryDuringSettleNormalizesSharedStateAndKeepsTheFocalCard() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        var shown by mutableStateOf(true)
        compose.setContent { MymindTheme(dark = true) { if (shown) LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val root = gallery.fetchSemanticsNode().boundsInRoot
        val before = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        val focal = before.center - root.topLeft
        compose.mainClock.autoAdvance = false
        gallery.performTouchInput {
            down(0, focal - Offset(width * 0.2f, 0f))
            down(1, focal + Offset(width * 0.2f, 0f))
            updatePointerTo(0, focal - Offset(width * 0.15f, 0f))
            updatePointerTo(1, focal + Offset(width * 0.15f, 0f))
            move(delayMillis = 16)
            up(0); up(1)
        }
        compose.mainClock.advanceTimeByFrame()
        assertTrue(density.position % 1f != 0f)
        compose.runOnIdle { shown = false }
        compose.mainClock.advanceTimeByFrame()
        assertEquals(3f, density.position, 0f)
        assertEquals(3, density.referenceColumns)
        compose.runOnIdle { shown = true }
        compose.mainClock.advanceTimeByFrame()
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
        val after = compose.onNodeWithContentDescription("Photo 19").fetchSemanticsNode().boundsInRoot
        assertEquals(before.center.y, after.center.y, 4f)
        val previous = state.firstVisibleItemIndex
        gallery.performTouchInput { swipe(Offset(centerX, height * 0.8f), Offset(centerX, height * 0.2f), durationMillis = 200) }
        compose.waitForIdle()
        assertTrue(state.firstVisibleItemIndex > previous)
    }

    @Test
    fun accessibleDensityChangesRemainSafeWhileTheFeedIsEmpty() {
        val density = MosaicState(2)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(emptyList(), endReached = true), {}, {}, mosaic = density) } }
        val smaller = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode()
            .config[androidx.compose.ui.semantics.SemanticsActions.CustomActions].single { it.label == "Smaller cards" }
        compose.runOnIdle { assertTrue(smaller.action()) }
        compose.waitForIdle()
        assertEquals(3f, density.position, 0f)
        assertEquals(3, density.referenceColumns)
    }

    @Test
    fun anUnavailableAccessibilityStepDoesNotCancelTheCurrentSettle() {
        val density = MosaicState(3)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        compose.mainClock.autoAdvance = false
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.3f))
            down(1, Offset(width * 0.9f, height * 0.3f))
            updatePointerTo(0, Offset(width * 0.18f, height * 0.3f))
            updatePointerTo(1, Offset(width * 0.82f, height * 0.3f))
            move(delayMillis = 16)
            up(0); up(1)
        }
        compose.mainClock.advanceTimeByFrame()
        assertEquals(4, density.columns)
        assertTrue(density.position < 4f)
        val smaller = gallery.fetchSemanticsNode().config[androidx.compose.ui.semantics.SemanticsActions.CustomActions]
            .single { it.label == "Smaller cards" }
        compose.runOnIdle { assertTrue(!smaller.action()) }
        compose.mainClock.advanceTimeBy(500)
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
        assertEquals(4f, density.position, 0f)
        assertEquals(4, density.referenceColumns)
    }

    @Test
    fun realTwoFingerPinchesChangeDensityBothWaysWithoutOpeningACard() {
        val density = MosaicState()
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, mosaic = density, onLongPressCard = { held += it.id })
            }
        }
        pinch(inward = true)
        assertEquals(4, density.columns)
        pinch(inward = false)
        assertEquals(2, density.columns)
        assertTrue(opened.isEmpty())
        assertTrue(held.isEmpty())
        assertTrue(!density.isPinching)
    }

    @Test
    fun aScrolledPinchKeepsTheTouchedRowsCardsVisible() {
        val density = MosaicState()
        val state = LazyListState(firstVisibleItemIndex = 8, firstVisibleItemScrollOffset = 40)
        compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val bounds = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode().boundsInRoot
        var touched = emptyList<String>()
        compose.runOnIdle {
            val point = bounds.height * 0.48f
            val row = state.layoutInfo.visibleItemsInfo.first { point >= it.offset && point < it.offset + it.size }
            touched = mosaicRows(cards, bounds.width, 2, 3f * 450f / 160f)[row.index].cards.map { it.title!! }
        }
        pinch(inward = true)
        assertEquals(4, density.columns)
        assertTrue("The pinch must retain its scroll offset", state.firstVisibleItemIndex > 0 || state.firstVisibleItemScrollOffset > 0)
        touched.forEach { compose.onNodeWithContentDescription(it).assertIsDisplayed() }
    }

    @Test
    fun tappingAndHoldingStillUseTheCardAtTheSelectedDensity() {
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id }, mosaic = MosaicState(3), onLongPressCard = { held += it.id })
            }
        }
        compose.onNodeWithContentDescription("Photo 1").performClick()
        compose.onNodeWithContentDescription("Photo 2").performTouchInput { longClick() }
        assertEquals(listOf("m1"), opened)
        assertEquals(listOf("m2"), held)
    }

    @Test
    fun anAppendedPageCanLoadTheNextPageWhileTheEndStaysVisible() {
        var feed by mutableStateOf(FeedState(cards.take(2)))
        var loads = 0
        compose.setContent {
            MymindTheme(dark = true) {
                LibraryMosaic(feed, onLoadMore = {
                    loads++
                    val size = (feed.cards.size + 2).coerceAtMost(8)
                    feed = FeedState(cards.take(size), endReached = size == 8)
                }, onOpenCard = {})
            }
        }
        compose.waitForIdle()
        assertEquals(8, feed.cards.size)
        assertEquals(3, loads)
    }
    @Test
    fun shrinkingAClippedSingleColumnKeepsThePinchedCardVisible() {
        val density = MosaicState(1)
        val state = LazyListState(firstVisibleItemIndex = 8, firstVisibleItemScrollOffset = 700)
        compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed()
        pinch(inward = true, yFraction = 0.04f)
        assertEquals(2, density.columns)
        val gallery = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode().boundsInRoot
        val kept = compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val focalY = gallery.top + gallery.height * 0.04f
        assertTrue("The resized card must remain at the pinch's height", focalY >= kept.top && focalY < kept.bottom)
    }

    @Test
    fun accessibleResizingKeepsAClippedSingleColumnCardVisible() {
        val density = MosaicState(1)
        val state = LazyListState(firstVisibleItemIndex = 8, firstVisibleItemScrollOffset = 700)
        compose.setContent { MymindTheme(dark = false) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed()
        val action = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode()
            .config[androidx.compose.ui.semantics.SemanticsActions.CustomActions].single { it.label == "Smaller cards" }
        compose.runOnIdle { assertTrue(action.action()) }
        compose.waitForIdle()
        assertEquals(2, density.columns)
        compose.onNodeWithContentDescription("Photo 9").assertIsDisplayed()
    }

    @Test
    fun pinchingInsidePullRefreshDoesNotRefreshOrOpenCardsAndAPullStillWorks() {
        val density = MosaicState()
        var refreshes = 0
        val opened = mutableListOf<String>()
        val held = mutableListOf<String>()
        compose.setContent {
            MymindTheme(dark = true) {
                PullRefresh(false, { refreshes++ }) {
                    LibraryMosaic(FeedState(cards, endReached = true), {}, { opened += it.id },
                        mosaic = density, onLongPressCard = { held += it.id })
                }
            }
        }
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            down(0, Offset(width * 0.2f, height * 0.12f))
            down(1, Offset(width * 0.8f, height * 0.12f))
            moveTo(0, Offset(width * 0.4f, height * 0.6f), delayMillis = 120)
            moveTo(1, Offset(width * 0.6f, height * 0.6f), delayMillis = 120)
            up(0)
            moveTo(1, Offset(width * 0.6f, height * 0.7f), delayMillis = 80)
            up(1)
        }
        compose.waitForIdle()
        assertEquals(4, density.columns)
        assertEquals(0, refreshes)
        assertTrue(opened.isEmpty())
        assertTrue(held.isEmpty())
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            swipe(Offset(centerX, height * 0.12f), Offset(centerX, height * 0.85f), durationMillis = 700)
        }
        compose.waitForIdle()
        assertEquals(1, refreshes)
    }

}

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
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasContentDescription
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
import kotlin.math.ln
import kotlin.math.pow

@RunWith(AndroidJUnit4::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class LibraryMosaicTest {
    @get:Rule val compose = createComposeRule()
    private val cards = (1..70).map { testCard("m$it", type = "Image", title = "Photo $it") }
    private val longNote = "Send your landing page to five people who have never heard of you and ask them what you sell. " +
        "If they cannot say it in one sentence, the page is not done."
    private val notes = (1..40).map { testCard("note-$it", type = "Note", title = "Note $it", note = if (it == 6) longNote else "A short note, number $it.") }

    /** Pinches by [stops] density stops (30% each) toward more columns ([inward]) or fewer. */
    private fun pinch(inward: Boolean, yFraction: Float = 0.48f, stops: Float = 2f) {
        compose.onNodeWithContentDescription("Card gallery").performTouchInput {
            val scale = 1.3f.pow(stops)
            val begin = width * if (inward) 0.32f else 0.32f / scale
            val end = width * if (inward) 0.32f / scale else 0.32f
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
    fun aGalleryShownBeforeItsCardsArrivePinchesOnceTheyDo() {
        // Opening the app shows the gallery a moment before its cards: "the moment I open the app pinching doesnt work
        // until I switch to spaces and back" (Tavi, 28 Oct 2026).
        val density = MosaicState(2)
        var feed by mutableStateOf(FeedState(loading = true))
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(feed, {}, {}, mosaic = density) } }
        compose.runOnIdle { feed = FeedState(cards, endReached = true) }
        pinch(inward = true)
        assertEquals(4, density.columns)
        pinch(inward = false)
        assertEquals(2, density.columns)
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
        for (spread in listOf(0.18f, 0.125f)) {
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
            // The grid stops at the gallery's two 16 dp margins, and the cards cover everything between them.
            val margin = 16f * 450f / 160f
            assertTrue("No empty margin on the left at ${density.position}: $left", left <= root.left + margin + 1f)
            assertTrue("No empty margin on the right at ${density.position}: $right", right >= root.right - margin - 1f)
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
            assertEquals(steppedPosition(2f - ln(0.75f) / ln(1.3f)), density.position, 0.01f)
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
        compose.mainClock.advanceTimeBy(64)
        assertTrue(density.position < before - 0.2f)
        gallery.performTouchInput { up(0); up(1) }
        compose.mainClock.autoAdvance = true
        compose.waitForIdle()
        assertTrue(!density.isPinching)
        assertEquals(density.columns.toFloat(), density.position, 0f)
    }

    @Test
    fun crossingAStopMovesEachCardAcrossManyPointerFrames() {
        val density = MosaicState(2)
        val state = LazyListState(firstVisibleItemIndex = 8)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        fun shown(): Map<String, androidx.compose.ui.geometry.Rect> = (10..36).mapNotNull { n ->
            compose.onAllNodesWithContentDescription("Photo $n").fetchSemanticsNodes().also { assertTrue("Photo $n has one touchable drawing", it.size <= 1) }
                .firstOrNull()?.let { "Photo $n" to androidx.compose.ui.geometry.Rect(it.positionInRoot, it.size.toSize()) }
        }.toMap()
        gallery.performTouchInput {
            down(0, Offset(width * 0.1f, height * 0.3f))
            down(1, Offset(width * 0.9f, height * 0.3f))
        }
        val frames = mutableListOf(shown())
        for (step in 1..8) {
            gallery.performTouchInput {
                updatePointerTo(0, Offset(width * (0.1f + step * 0.011f), height * 0.3f))
                updatePointerTo(1, Offset(width * (0.9f - step * 0.011f), height * 0.3f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
            assertTrue("The pinch must stay between two and three columns: ${density.position}", density.position > 2f && density.position < 3f)
            frames += shown()
        }
        val moved = frames.first().keys.filter { name ->
            frames.all { name in it } && (frames.last().getValue(name).topLeft - frames.first().getValue(name).topLeft).getDistance() > 40f
        }
        assertTrue("Crossing a stop must move cards to new places", moved.isNotEmpty())
        for (name in moved) {
            val places = frames.map { it.getValue(name).topLeft }.zipWithNext().count { (a, b) -> (b - a).getDistance() > 1f }
            assertTrue("$name must move on several pointer frames, not all at once: $places", places >= 3)
        }
        gallery.performTouchInput { up(0); up(1) }
        compose.waitForIdle()
        assertEquals(3, density.columns)
    }

    /**
     * Pinches a gallery of photos and notes from [from] columns until the fingers are at [finger], lets go after holding
     * still ([hold]) or while still moving, and steps the landing frame by frame. Returns the density on each pinch and
     * landing frame, and checks that no photo moves once the landing has reached its stop.
     */
    private fun landPartway(from: Int, finger: Float, hold: Boolean): List<Float> {
        val mixed = cards.take(40).zip(notes) { photo, note -> listOf(photo, note) }.flatten()
        val density = MosaicState(from)
        val state = LazyListState(firstVisibleItemIndex = 8)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(mixed, endReached = true), {}, {}, state = state, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val span = if (from == 2) 0.8f else 0.5f
        fun inset(at: Float) = (1f - span * 1.3f.pow(from - at)) / 2f
        fun photos(): Map<String, androidx.compose.ui.geometry.Rect> = compose.onAllNodes(hasContentDescription("Photo ", substring = true))
            .fetchSemanticsNodes().associate { it.config[SemanticsProperties.ContentDescription].first() to it.boundsInRoot }
        gallery.performTouchInput {
            down(0, Offset(width * inset(from.toFloat()), height * 0.4f))
            down(1, Offset(width * (1f - inset(from.toFloat())), height * 0.4f))
        }
        compose.mainClock.autoAdvance = false
        val positions = mutableListOf<Float>()
        try {
            compose.mainClock.advanceTimeByFrame()
            for (step in 1..6) {
                val at = inset(from + (finger - from) * step / 6)
                gallery.performTouchInput {
                    updatePointerTo(0, Offset(width * at, height * 0.4f))
                    updatePointerTo(1, Offset(width * (1f - at), height * 0.4f))
                    move(delayMillis = 16)
                }
                compose.mainClock.advanceTimeByFrame()
                positions += density.position
            }
            if (hold) repeat(12) { compose.mainClock.advanceTimeByFrame() }
            gallery.performTouchInput { up(0); up(1) }
            while (density.position != density.columns.toFloat()) {
                assertTrue("The landing must end: ${density.position}", positions.size < 60)
                compose.mainClock.advanceTimeByFrame()
                positions += density.position
            }
            val landed = photos()
            assertTrue(landed.isNotEmpty())
            repeat(4) { frame ->
                compose.mainClock.advanceTimeByFrame()
                for ((name, place) in photos()) landed[name]?.let {
                    assertEquals("$name must stay where the landing left it, frame $frame (top)", it.top, place.top, 1.5f)
                    assertEquals("$name must stay where the landing left it, frame $frame (left)", it.left, place.left, 1.5f)
                }
            }
            assertEquals(density.columns, density.referenceColumns)
        } finally {
            compose.mainClock.autoAdvance = true
            compose.waitForIdle()
        }
        return positions
    }

    @Test
    fun aPinchLetGoBeforeTheNextSizeLandsThereWithoutAJump() {
        // "It's when I pinch and let go without fully getting to the next stage. Then it pops, instead of being smooth"
        // (Tavi, 28 Oct 2026). The list took the new size's rows a frame after the scroll position meant for them.
        val positions = landPartway(2, 2.35f, hold = true)
        assertEquals(3f, positions.last(), 0f)
        // The fingers were still, so the landing starts from still rather than at its fastest.
        assertTrue("The landing's first frame must start slowly: $positions", positions[6] - positions[5] < 0.03f)
        assertTrue("No frame of the landing may lurch: $positions", positions.drop(5).zipWithNext().all { (a, b) -> b - a < 0.12f })
    }

    @Test
    fun aPinchOutLetGoBeforeTheNextSizeLandsThereWithoutAJump() {
        assertEquals(2f, landPartway(3, 2.65f, hold = true).last(), 0f)
    }

    @Test
    fun aLandingStartsAtTheSpeedTheCardsHad() {
        val positions = landPartway(2, 2.35f, hold = false)
        val pinching = positions[5] - positions[4]
        val landing = positions[6] - positions[5]
        assertTrue("The landing must carry on at about the pinch's speed: $pinching then $landing", landing > 0.5f * pinching && landing < 2f * pinching)
    }

    @Test
    fun theCardUnderTheFingersChangesShapeWithoutChangingLine() {
        // A screenshot stands two squares high where it starts a band at two columns, but falls in a band's second row
        // at three and takes one square there: the card under the fingers changes shape between the stops, not line.
        fun shot(n: Int) = testCard("shot-$n", type = "Image", title = "Screenshot $n").copy(cover = Cover("/media/shot-$n.jpg", 1080, 2400))
        fun note(n: Int) = testCard("note-$n", type = "Note", title = "Note $n", note = "A short note, number $n.")
        val cards = listOf(shot(0), note(1), note(2), shot(3)) + (4..30).map { if (it % 3 == 0) shot(it) else note(it) }
        val density = MosaicState(2)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(cards, endReached = true), {}, {}, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val root = gallery.fetchSemanticsNode().boundsInRoot
        fun drawn(): androidx.compose.ui.geometry.Rect = compose.onAllNodesWithContentDescription("Screenshot 3").fetchSemanticsNodes()
            .also { assertEquals("Screenshot 3 has one touchable drawing", 1, it.size) }.single()
            .let { androidx.compose.ui.geometry.Rect(it.positionInRoot, it.size.toSize()) }
        val start = drawn()
        assertTrue("Screenshot 3 starts two squares high: $start", start.height > 1.9f * start.width)
        val fingers = start.center - root.topLeft
        gallery.performTouchInput {
            down(0, fingers - Offset(width * 0.1f, 0f))
            down(1, fingers + Offset(width * 0.1f, 0f))
        }
        var left = start.left
        for (step in 1..10) {
            gallery.performTouchInput {
                updatePointerTo(0, fingers - Offset(width * (0.1f - step * 0.002f), 0f))
                updatePointerTo(1, fingers + Offset(width * (0.1f - step * 0.002f), 0f))
                move(delayMillis = 16)
            }
            compose.waitForIdle()
            assertTrue("The pinch must stay between two and three columns: ${density.position}", density.position > 2f && density.position < 3f)
            // Its place at three columns is to the right, so it slides that way rather than out past the left edge.
            val now = drawn()
            assertTrue("Screenshot 3 must move steadily to its new place at ${density.position}: $now after left $left", now.left >= left - 1f)
            left = now.left
        }
        gallery.performTouchInput { up(0); up(1) }
    }

    @Test
    fun aNoteZoomsWithoutRewrappingBetweenStops() {
        val density = MosaicState(4)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(notes, endReached = true), {}, {}, mosaic = density) } }
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        fun lines() = noteLines()
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
    fun aNoteKeepsItsLinesAtEveryDensity() {
        val density = MosaicState(3)
        compose.setContent { MymindTheme(dark = true) { LibraryMosaic(FeedState(notes, endReached = true), {}, {}, mosaic = density) } }
        fun width() = compose.onNodeWithContentDescription("Note 6").fetchSemanticsNode().boundsInRoot.width
        val before = noteLines()
        val small = width()
        val gallery = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode()
        compose.runOnIdle { gallery.config[SemanticsActions.CustomActions].first { it.label == "Larger cards" }.action() }
        compose.waitForIdle()
        assertEquals(2, density.columns)
        assertEquals("A note is a picture of itself, with the same lines at every density", before, noteLines())
        assertTrue("The note must be drawn larger: $small -> ${width()}", width() > small * 1.4f)
    }

    /** Where each line of [longNote] ends in its tile's layout. */
    private fun noteLines(): List<Int> {
        val results = mutableListOf<TextLayoutResult>()
        compose.onNodeWithText(longNote, useUnmergedTree = true).performSemanticsAction(SemanticsActions.GetTextLayoutResult) { it(results) }
        val layout = results.single()
        return (0 until layout.lineCount).map { layout.getLineEnd(it, visibleEnd = true) }
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
        compose.mainClock.advanceTimeBy(48)
        assertTrue("The gallery must still be landing: ${density.position}", density.position > 2f && density.position < 3f)
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
            compose.mainClock.advanceTimeBy(48)
            assertTrue("The gallery must still be landing: ${density.position}", density.position > 2f && density.position < 3f)
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
        compose.mainClock.advanceTimeBy(48)
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
            // The gallery lays its cards out inside two 16 dp margins, with its gap for two columns.
            val dp = 450f / 160f
            val inner = bounds.width - 32f * dp
            touched = mosaicRows(cards, inner, 2, mosaicGapDp(2, inner / dp) * dp)[row.index].cards.map { it.title!! }
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
        pinch(inward = true, yFraction = 0.04f, stops = 1f)
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

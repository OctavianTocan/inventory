package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.Cover
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.pow
import kotlin.math.roundToInt

class MosaicLayoutTest {
    private val cards = (1..53).map { n ->
        testCard("m$n", type = "Image").copy(cover = Cover(width = if (n % 3 == 0) 1600 else 800, height = 1000))
    }

    private fun photo(id: String, width: Int, height: Int) = testCard(id, type = "Image")
        .copy(cover = Cover("/media/$id.jpg", width = width, height = height))

    @Test
    fun bandsKeepEveryCardInOrderAndFitTheViewport() {
        for (columns in 1..4) {
            val level = mosaicLevel(cards, width = 1080f, columns = columns, gap = 8f)
            assertEquals(cards.map { it.id }, level.rows.flatMap { it.cards }.map { it.id })
            assertTrue(level.rows.all { it.height.isFinite() && it.height > 0f })
            assertTrue(level.cells.all { it.left >= -0.01f && it.right <= 1080.01f && it.width > 0f && it.height > 0f })
            assertTrue(level.cells.zipWithNext().all { (a, b) -> a.top <= b.top })
        }
    }

    @Test
    fun picturesTakeShapesLikeGooglePhotos() {
        assertEquals(MosaicSpan(1, 2), mosaicSpan(photo("screenshot", 1080, 2400)))
        assertEquals(MosaicSpan(1, 1), mosaicSpan(testCard("note", type = "Note", note = "A thought")))
        val wide = (1..40).map { mosaicSpan(photo("wide$it", 1920, 1080)) }
        assertTrue("About half the wide pictures lie two across", wide.count { it == MosaicSpan(2, 1) } in 10..30)
        val square = (1..140).map { mosaicSpan(photo("square$it", 1000, 1000)) }
        assertTrue("A few pictures are big", square.count { it == MosaicSpan(2, 2) } in 8..40)
        assertTrue(square.none { it == MosaicSpan(2, 1) || it == MosaicSpan(1, 2) })
    }

    @Test
    fun cardsWithWordsAndAPictureShareTheShowcasesMixOfSizes() {
        assertEquals(MosaicSpan(1, 2), mosaicSpan(photo("noted", 1080, 2400).copy(note = "Words beside it")))
        val articles = (1..140).map { mosaicSpan(photo("article$it", 1200, 630).copy(type = "Article", title = "A headline")) }
        assertTrue("About one article with a picture in seven is big", articles.count { it == MosaicSpan(2, 2) } in 8..40)
        assertTrue("Words over a wide picture never lie two across", articles.none { it == MosaicSpan(2, 1) })
        val notes = (1..140).map { mosaicSpan(testCard("words$it", type = "Note", note = "Only words")) }
        assertTrue("Words alone stay one square", notes.all { it == MosaicSpan(1, 1) })
    }

    @Test
    fun gapsFollowTheShowcaseAndTightenWithMoreColumns() {
        // A 384 dp phone less its two 16 dp margins, as the gallery and the web showcase lay it out.
        assertEquals(listOf(16f, 8f, 6f, 6f), (1..4).map { mosaicGapDp(it, 352f) })
        assertEquals(24f, mosaicGapDp(1, 1200f))
        assertEquals(6f, mosaicGapDp(4, 0f))
    }

    @Test
    fun aBigTileWithWordsKeepsItsNeighboursTextSize() {
        val big = (1..60).map { photo("big$it", 1200, 900).copy(type = "Article", title = "A headline") }
            .first { mosaicSpan(it) == MosaicSpan(2, 2) }
        val sample = listOf(big) + (1..8).map { testCard("n$it", type = "Note", note = "Note $it") }
        val layouts = MosaicLayouts(sample, 1000f, listOf(16f, 8f, 6f, 6f))
        for (columns in 2..4) {
            val level = layouts.level(columns)
            val scale = { index: Int -> level.cells[index].width / layouts.wordsFrame(index, columns).width }
            assertEquals("At $columns columns", scale(1), scale(0), 0.01f)
            assertEquals(level.cells[0].height / level.cells[0].width,
                layouts.wordsFrame(0, columns).let { it.height / it.width }, 0.01f)
        }
        assertEquals(layouts.wordsFrame(0, 2), layouts.wordsFrame(0, 3))
        assertEquals(layouts.wordsFrame(1, 2), layouts.wordsFrame(1, 4))
    }

    @Test
    fun aScreenshotStandsTwoSquaresHighAndTheBandAroundItHasNoHoles() {
        val square = (1..20).map { photo("square$it", 1000, 1000) }.first { mosaicSpan(it) == MosaicSpan(1, 1) }
        val sample = listOf(square, photo("shot", 1080, 2400)) + (1..6).map { testCard("t$it", type = "Note", note = "Note $it") }
        val level = mosaicLevel(sample, 1080f, 3, 8f)
        val unit = (1080f - 16f) / 3
        assertEquals(2 * unit + 8f, level.cells[1].height, 0.01f)
        assertEquals(listOf(0, 0, 0, 0, 0), level.rowOf.take(5).toList())
        assertEquals(level.rows.first().height, level.cells[1].height, 0.01f)
    }

    @Test
    fun everyBandButTheLastFillsItsSquares() {
        val mixed = (1..90).map { n ->
            when (n % 5) {
                0 -> photo("shot$n", 1080, 2400)
                1 -> photo("wide$n", 1920, 1080)
                2 -> testCard("note$n", type = "Note", note = "Note $n")
                else -> photo("photo$n", 1000, 1000)
            }
        }
        for (columns in 2..4) {
            val level = mosaicLevel(mixed, 1080f, columns, 8f)
            val unit = (1080f - 8f * (columns - 1)) / columns
            level.rows.dropLast(1).forEachIndexed { band, row ->
                val squares = (level.starts[band] until level.starts[band] + row.cards.size).sumOf { index ->
                    val cell = level.cells[index]
                    ((cell.width + 8f) / (unit + 8f)).roundToInt() * ((cell.height + 8f) / (unit + 8f)).roundToInt()
                }
                assertEquals("Band $band at $columns columns", columns * ((row.height + 8f) / (unit + 8f)).roundToInt(), squares)
            }
            for (a in level.cells.indices) for (b in a + 1 until level.cells.size) {
                val overlap = level.cells[a].intersect(level.cells[b])
                assertTrue("$columns columns collided: $a/$b", overlap.width <= 0f || overlap.height <= 0f)
            }
        }
    }

    @Test
    fun everyDensityStopChangesTileSizeForPortraitAndLandscapePhotos() {
        for (ratio in listOf(0.65f, 1f, 1.15f, 1.8f)) {
            val photos = (1..24).map { photo("photo$it-$ratio", (1000 * ratio).toInt(), 1000) }
            var previousUnit = Float.POSITIVE_INFINITY
            for (columns in 1..4) {
                val level = mosaicLevel(photos, 1080f, columns, 8f)
                val unit = level.cells.minOf { it.width }
                assertTrue("$columns columns must make $ratio-aspect photos smaller", unit < previousUnit)
                assertEquals(columns, level.cells.map { it.left.roundToInt() }.distinct().size)
                previousUnit = unit
            }
        }
    }

    @Test
    fun missingAndBrokenDimensionsHaveFiniteFallbacks() {
        val broken = listOf(
            testCard("a", type = "Note", note = "A note"),
            testCard("b", type = "Image").copy(cover = Cover(width = 0, height = -1)),
            testCard("c", type = "Image").copy(cover = Cover(width = Int.MAX_VALUE, height = 1)),
        )
        for (columns in 1..4) {
            val rows = mosaicRows(broken, 384f, columns, 3f)
            assertEquals(3, rows.sumOf { it.cards.size })
            assertTrue(rows.all { it.height.isFinite() && it.height > 0f })
        }
        assertTrue(mosaicRows(broken, 0f, 2, 3f).isEmpty())
        assertTrue(mosaicRows(broken, Float.NaN, 2, 3f).isEmpty())
    }

    @Test
    fun anIncompleteLastBandDoesNotTurnOnePhotoIntoAHero() {
        val lone = (1..20).map { photo("lone$it", 800, 1000) }.first { mosaicSpan(it) == MosaicSpan(1, 1) }
        val cell = mosaicLevel(listOf(lone), 1080f, 4, 8f).cells.single()
        assertTrue(cell.width <= 1080f / 4)
    }

    @Test
    fun continuousGeometryUsesCanonicalStopsAndKeepsOrderAndFiniteBoundsBetweenThem() {
        val layouts = MosaicLayouts(cards, 1080f, 8f)
        for (position in listOf(1f, 1.25f, 1.7f, 2f, 2.4f, 2.9f, 3.5f, 4f)) {
            val cells = cards.indices.map { layouts.cell(it, position) }
            assertTrue(cells.all { it.left >= -0.01f && it.right <= 1080.01f && it.width > 0f && it.height > 0f })
            assertTrue(cells.zipWithNext().all { (a, b) -> a.top <= b.top })
            assertTrue(cells.maxOf { it.bottom } <= layouts.height(position) + 0.01f)
            if (position % 1f == 0f) assertEquals(cells.maxOf { it.bottom }, layouts.height(position), 0.01f)
            if (position % 1f == 0f) assertEquals(layouts.level(position.toInt()).cells, cells)
        }
    }

    @Test
    fun aLargeLibraryOnlySelectsTheBoundedVisibleWindow() {
        val many = (0 until 10_000).map { cards[it % cards.size].copy(id = "large-$it") }
        val layouts = MosaicLayouts(many, 1080f, 8f)
        val y = layouts.height(2.6f) / 2
        val selected = layouts.window(2.6f, y, 2200f)
        assertTrue(selected.first > 0 && selected.last < many.lastIndex)
        assertTrue("Pointer frames must only interpolate a viewport window", selected.count() < 100)
        many.indices.filter { layouts.cell(it, 2.6f).let { cell -> cell.bottom > y && cell.top < y + 2200f } }
            .forEach { assertTrue(it in selected) }
    }

    @Test
    fun eachLiveProjectionKeepsTheRowsOrderedAndCollisionFreeAroundTheSameFocalCard() {
        val layouts = MosaicLayouts(cards, 1080f, 8f)
        val focal = 21
        for (position in listOf(1.4f, 1.5f, 1.6f, 2.4f, 2.5f, 2.6f, 3.4f, 3.5f, 3.6f)) {
            val desired = layouts.cell(focal, position)
            for (columns in listOf(kotlin.math.floor(position).toInt(), kotlin.math.ceil(position).toInt())) {
                val projection = layouts.projection(columns, position, focal)
                val cells = cards.indices.map(projection::cell)
                assertEquals(desired.left, cells[focal].left, 0.01f)
                assertEquals(desired.top, cells[focal].top, 0.01f)
                assertEquals(desired.width, cells[focal].width, 0.01f)
                assertEquals(layouts.level(columns).cells[focal].height * projection.scale, cells[focal].height, 0.01f)
                assertTrue(cells.zipWithNext().all { (a, b) -> a.top <= b.top })
                for (a in cells.indices) for (b in a + 1 until cells.size) {
                    val overlap = cells[a].intersect(cells[b])
                    assertTrue("Projection $columns at $position collided: $a/$b", overlap.width <= 0f || overlap.height <= 0f)
                }
                val origin = cells[focal].top
                val selected = projection.window(origin, 2200f)
                cells.indices.filter { cells[it].bottom > origin && cells[it].top < origin + 2200f }
                    .forEach { assertTrue(it in selected) }
            }
        }
    }

    @Test
    fun onePinchCanCrossSeveralStopsAndReverseBeforeRelease() {
        val state = MosaicState(2)
        state.beginPinch()
        state.zoomPinch(0.5f)
        state.advanceHop(1f)
        assertEquals(4f, state.position, 0.001f)
        state.zoomPinch(1.69f)
        state.advanceHop(1f)
        assertEquals(2f, state.position, 0.001f)
        state.zoomPinch(2f)
        state.advanceHop(1f)
        assertEquals(1f, state.position, 0.001f)
        state.finishPinch()
        assertEquals(1, state.columns)
        assertEquals(3, MosaicState.Saver.restore(3)!!.columns)
    }

    @Test
    fun cardsFollowTheFingersFromTheFirstMoveAndHopPastHalfway() {
        val state = MosaicState(3)
        state.beginPinch()
        state.zoomPinch(1.01f)
        assertTrue("The first pointer move must change the size: ${state.position}", state.position < 3f - 0.01f)
        var last = state.position
        repeat(8) {
            state.zoomPinch(1.01f)
            assertTrue("Every move toward fewer columns must enlarge the cards", state.position < last)
            last = state.position
        }
        // One stop is a 30% pinch; the cards follow 60% of the way and hop the rest once the fingers pass halfway.
        state.zoomPinch(1.3f / 1.01f.pow(9))
        assertTrue("A hop must ease, not jump: ${state.position}", state.hopping && state.position > 2.6f)
        state.advanceHop(0.5f)
        val midway = state.position
        state.advanceHop(1f)
        assertEquals(2f, state.position, 0.001f)
        assertTrue("The hop must be on its way halfway through: $midway", midway < last && midway > 2f)
        assertFalse(state.hopping)
    }

    @Test
    fun releaseLandsTheWayTheFingersMovedOnceAQuarterIn() {
        assertEquals(2, releaseColumns(2.74f, -1))
        assertEquals(3, releaseColumns(2.76f, -1))
        assertEquals(3, releaseColumns(2.26f, 1))
        assertEquals(2, releaseColumns(2.24f, 1))
        assertEquals(2, releaseColumns(2.4f, 0))
        val state = MosaicState(3)
        state.beginPinch()
        state.zoomPinch(1.3f.pow(0.3f))
        state.finishPinch()
        assertEquals("Past a quarter of the way to two columns lands there", 2, state.columns)
        val other = MosaicState(2)
        other.beginPinch()
        other.zoomPinch(1.3f.pow(-0.2f))
        other.finishPinch()
        assertEquals("A fifth of the way to three columns falls back", 2, other.columns)
    }

    @Test
    fun aStepRunsFromTheGesturesHomeStop() {
        val state = MosaicState(3)
        state.beginPinch()
        state.zoomPinch(1.3f.pow(0.3f))
        state.between().let {
            assertEquals(3, it.from)
            assertEquals(2, it.to)
            assertEquals(0.18f, it.progress, 0.01f)
        }
        state.zoomPinch(1.3f.pow(1.1f))
        state.advanceHop(1f)
        state.between().let {
            assertEquals(2, it.from)
            assertEquals(1, it.to)
        }
        assertEquals(1.8f, steppedPosition(fingerPosition(1.8f)), 0.001f)
    }

    @Test
    fun theBlurredFillFadesInFromAStopAndOutTowardTheNext() {
        assertEquals(0f, mosaicFillShown(0f), 0f)
        assertEquals(0f, mosaicFillShown(1f), 0f)
        assertEquals(1f, mosaicFillShown(0.5f), 0f)
        assertEquals(1f, mosaicFillShown(0.2f), 0.001f)
        assertEquals(1f, mosaicFillShown(0.8f), 0.001f)
        assertTrue("The fill must be all but gone a frame from a stop", mosaicFillShown(0.01f) < 0.01f && mosaicFillShown(0.99f) < 0.01f)
        val rising = (0..20).map { mosaicFillShown(it / 100f) }
        assertTrue("The fill must fade in without steps: $rising", rising.zipWithNext().all { (a, b) -> b > a && b - a < 0.12f })
    }

    @Test
    fun pinchAndAccessibleStepsRespectTheDensityLimits() {
        val state = MosaicState()
        state.beginPinch()
        state.zoomPinch(0.8f)
        state.finishPinch()
        assertEquals(3, state.columns)
        // The gallery lands each released pinch on its stop.
        state.position = state.columns.toFloat()
        state.beginPinch()
        state.zoomPinch(1.25f)
        state.finishPinch()
        assertEquals(2, state.columns)
        repeat(10) { state.larger() }
        assertEquals(1, state.columns)
        assertFalse(state.larger())
        repeat(10) { state.smaller() }
        assertEquals(4, state.columns)
        assertFalse(state.smaller())
        state.beginPinch()
        state.zoomPinch(Float.NaN)
        state.finishPinch()
        assertEquals(4, state.columns)
        assertFalse(state.isPinching)
    }

    @Test
    fun aLandingStartsAtTheCardsSpeedAndComesToRestWithoutOvershooting() {
        for (lead in listOf(0f, 0.5f, 1f, 3f, 7f)) {
            assertEquals(0f, settleProgress(0f, lead), 0f)
            assertEquals(1f, settleProgress(1f, lead), 1e-6f)
            val curve = (0..100).map { settleProgress(it / 100f, lead) }
            assertTrue("Lead $lead must never turn back or overshoot", curve.zipWithNext().all { (a, b) -> b >= a - 1e-6f } && curve.max() <= 1f + 1e-6f)
            // It leaves at the lead's speed and arrives at rest.
            assertEquals(lead.coerceAtMost(3f), settleProgress(0.001f, lead) / 0.001f, 0.01f)
            assertEquals(0f, (1f - settleProgress(0.999f, lead)) / 0.001f, 0.01f)
        }
        assertEquals(250f, settleMillis(1f), 1e-3f)
        assertEquals(250f, settleMillis(-1f), 1e-3f)
        assertEquals("The last of a stop after a hop is not rushed", 103f, settleMillis(0.17f), 1f)
        // At the most it takes, it is the plain ease-out.
        assertEquals(1f - 0.5f * 0.5f * 0.5f, settleProgress(0.5f, 3f), 1e-6f)
        // 0.3 stops per 100 ms toward a stop 0.6 away, landed in 150 ms: the cards leave at 0.75 times the average speed.
        assertEquals(0.75f, settleLead(0.003f, 0.6f, 150f), 1e-4f)
        assertEquals("Moving away from the stop starts from still", 0f, settleLead(0.003f, -0.6f, 150f), 0f)
        assertEquals(3f, settleLead(1f, 0.6f, 150f), 0f)
    }

    @Test
    fun aPinchsSpeedIsTheDensitysLastMoveAndZeroAfterAHold() {
        val speed = MosaicSpeed()
        assertEquals(0f, speed.at(100, 2f), 0f)
        for (frame in 0..10) speed.add(frame * 16L, 2f + frame * 0.016f)
        assertEquals(0.001f, speed.at(160, 2.16f), 1e-5f)
        assertEquals("Fingers held still before letting go give no speed", 0f, speed.at(400, 2.16f), 0f)
        speed.clear()
        assertEquals(0f, speed.at(400, 2.16f), 0f)
    }

    @Test
    fun reelsGiveEveryCardAPageAndDrawItWholeInside() {
        val sample = listOf(photo("wide", 1200, 800), photo("tall", 800, 1600), testCard("note", type = "Note", note = "A thought to keep"))
        val layouts = MosaicLayouts(sample, 352f, List(4) { 8f }, page = 800f, bleed = 16f)
        assertTrue("With a page, one column is the reels", layouts.isReels(1))
        assertFalse(layouts.isReels(2))
        val reels = layouts.level(1)
        assertEquals("Each page reaches the screen's edges", androidx.compose.ui.geometry.Rect(-16f, 800f, 368f, 1600f), reels.cells[1])
        assertEquals(sample.map { it.id }, reels.rows.flatMap { it.cards }.map { it.id })
        assertEquals(2400f, reels.height, 0f)
        val wide = layouts.reelContent(0, reels.cells[0], 1f)
        assertEquals("A wide picture shows whole across its page", 384f, wide.width, 0.5f)
        assertEquals(256f, wide.height, 0.5f)
        assertEquals("and in the page's middle", 400f, wide.center.y, 0.5f)
        val tall = layouts.reelContent(1, reels.cells[1], 1f)
        assertEquals(384f, tall.width, 0.5f)
        assertEquals(768f, tall.height, 0.5f)
        val plain = MosaicLayouts(sample, 352f, List(4) { 8f })
        assertFalse("Without a page, one column is a grid", plain.isReels(1))
        assertEquals("Words keep their one-column lines on a page", plain.wordsFrame(2, 1), layouts.wordsFrame(2, 1))
        val note = layouts.reelContent(2, reels.cells[2], 1f)
        val one = plain.level(1).cells[2]
        assertEquals("and their one-column shape", one.width / one.height, note.width / note.height, 0.001f)
        assertEquals("Before reels a card fills its cell", one, layouts.reelContent(2, one, 0f))
    }

    @Test
    fun withReelsOneColumnIsThemAndAPinchInClosesThem() {
        val state = MosaicState(2, reels = true)
        assertEquals(0f, state.reel, 0f)
        state.beginPinch()
        state.zoomPinch(1.3f)
        state.advanceHop(1f)
        assertEquals(1f, state.position, 0.001f)
        state.finishPinch()
        assertEquals(1, state.columns)
        assertTrue(state.inReels)
        assertEquals(1f, state.reel, 0f)
        state.position = 1f
        state.beginPinch()
        state.zoomPinch(1.3f)
        state.finishPinch()
        assertEquals("Nothing lies past the reels", 1, state.columns)
        state.position = 1f
        state.beginPinch()
        state.zoomPinch(1f / 1.3f)
        state.advanceHop(1f)
        state.finishPinch()
        assertEquals(2, state.columns)
        assertFalse(state.inReels)
        assertTrue(state.larger())
        assertTrue(state.inReels)
        assertFalse(state.larger())
        val plain = MosaicState(1)
        assertEquals("A gallery without reels has none", 0f, plain.reel, 0f)
        assertFalse(plain.inReels)
        assertEquals("A gallery saved at the reels' old zero columns opens them", 1, MosaicState.saver(reels = true).restore(0)!!.columns)
        assertTrue(MosaicState.saver(reels = true).restore(2)!!.reels)
        assertEquals(1, MosaicState.Saver.restore(0)!!.columns)
    }
}

package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.Cover
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
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
        assertEquals(MosaicSpan(1, 1), mosaicSpan(photo("noted", 1080, 2400).copy(note = "Words beside it")))
        val wide = (1..40).map { mosaicSpan(photo("wide$it", 1920, 1080)) }
        assertTrue("About half the wide pictures lie two across", wide.count { it == MosaicSpan(2, 1) } in 10..30)
        val square = (1..140).map { mosaicSpan(photo("square$it", 1000, 1000)) }
        assertTrue("A few pictures are big", square.count { it == MosaicSpan(2, 2) } in 8..40)
        assertTrue(square.none { it == MosaicSpan(2, 1) || it == MosaicSpan(1, 2) })
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
        assertEquals(4f, state.position, 0.001f)
        state.zoomPinch(2f)
        assertEquals(2f, state.position, 0.001f)
        state.zoomPinch(2f)
        assertEquals(1f, state.position, 0.001f)
        state.finishPinch()
        assertEquals(1, state.columns)
        assertEquals(3, MosaicState.Saver.restore(3)!!.columns)
    }

    @Test
    fun pinchAndAccessibleStepsRespectTheDensityLimits() {
        val state = MosaicState()
        state.beginPinch()
        state.zoomPinch(0.6f)
        state.finishPinch()
        assertEquals(3, state.columns)
        state.beginPinch()
        state.zoomPinch(1.6f)
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
}

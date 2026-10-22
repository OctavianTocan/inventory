package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.Cover
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class MosaicLayoutTest {
    private val cards = (1..53).map { n ->
        testCard("m$n", type = "Image").copy(cover = Cover(width = if (n % 3 == 0) 1600 else 800, height = 1000))
    }

    @Test
    fun rowsKeepEveryCardInOrderAndFitTheViewport() {
        for (columns in 1..4) {
            val rows = mosaicRows(cards, width = 1080f, columns = columns, gap = 8f)
            assertEquals(cards.map { it.id }, rows.flatMap { it.cards }.map { it.id })
            rows.forEach { row ->
                assertTrue(row.height.isFinite() && row.height > 0f)
                assertTrue(row.widths.all { it.isFinite() && it > 0f })
                assertTrue(row.widths.sum() + 8f * (row.cards.size - 1) <= 1080.01f)
            }
        }
    }

    @Test
    fun theBrickRowsUseDifferentWidthsForDifferentPictures() {
        val row = mosaicRows(cards.take(2), 1080f, 2, 8f).single()
        val varied = mosaicRows(listOf(cards[0], cards[2]), 1080f, 2, 8f).single()
        assertEquals(row.widths[0], row.widths[1], 0.01f)
        assertTrue(varied.widths[1] > varied.widths[0] * 1.5f)
    }

    @Test
    fun everyDensityStopChangesTileSizeForPortraitAndLandscapePhotos() {
        for (ratio in listOf(0.65f, 1f, 1.15f, 1.8f)) {
            val photos = (1..24).map { testCard("photo$it", type = "Image")
                .copy(cover = Cover(width = (1000 * ratio).toInt(), height = 1000)) }
            var previousWidth = Float.POSITIVE_INFINITY
            for (columns in 1..4) {
                val rows = mosaicRows(photos, 1080f, columns, 8f)
                assertTrue("$columns columns must make $ratio-aspect photos smaller", rows.first().widths.first() < previousWidth)
                assertTrue(rows.all { it.cards.size == columns })
                previousWidth = rows.first().widths.first()
            }
            val finalRow = mosaicRows(photos.take(1), 1080f, 4, 8f).single()
            assertTrue("A short last row must keep landscape thumbnails small", finalRow.widths.single() <= 270f)
        }
    }

    @Test
    fun missingAndBrokenDimensionsHaveFiniteFallbacks() {
        val broken = listOf(
            testCard("a", type = "Note", note = "A note"),
            testCard("b", type = "Image").copy(cover = Cover(width = 0, height = -1)),
            testCard("c", type = "Image").copy(cover = Cover(width = Int.MAX_VALUE, height = 1)),
        )
        val rows = mosaicRows(broken, 384f, 3, 3f)
        assertEquals(3, rows.sumOf { it.cards.size })
        assertTrue(rows.all { it.height.isFinite() && it.height > 0f })
        assertTrue(mosaicRows(broken, 0f, 2, 3f).isEmpty())
        assertTrue(mosaicRows(broken, Float.NaN, 2, 3f).isEmpty())
    }

    @Test
    fun anIncompleteLastRowDoesNotTurnOnePhotoIntoAHero() {
        val row = mosaicRows(cards.take(1), 1080f, 4, 8f).single()
        assertTrue(row.widths.single() <= 1080f / 4)
        assertTrue(row.widths.single() < 1080f / 2)
    }

    @Test
    fun continuousGeometryUsesCanonicalStopsAndKeepsOrderAndFiniteBoundsBetweenThem() {
        val layouts = MosaicLayouts(cards, 1080f, 8f)
        for (position in listOf(1f, 1.25f, 1.7f, 2f, 2.4f, 2.9f, 3.5f, 4f)) {
            val cells = cards.indices.map { layouts.cell(it, position) }
            assertTrue(cells.all { it.left >= -0.01f && it.right <= 1080.01f && it.width > 0f && it.height > 0f })
            assertTrue(cells.zipWithNext().all { (a, b) -> a.top <= b.top })
            assertEquals(cells.last().bottom, layouts.height(position), 0.01f)
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
                assertEquals(desired.height, cells[focal].height, 0.01f)
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

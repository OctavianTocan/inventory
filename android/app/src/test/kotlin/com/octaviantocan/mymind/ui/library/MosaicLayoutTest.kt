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
        assertTrue(row.height <= 1080f / 4)
        assertTrue(row.widths.single() < 1080f / 2)
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

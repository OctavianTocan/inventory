package com.octaviantocan.mymind.ui.card

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.unit.Velocity
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class CardSheetMotionTest {
    private fun cardSheetTarget(offset: Float, velocity: Float, speed: Float, height: Float) =
        cardSheetTarget(offset, velocity, speed, listOf(0f, height / 2f), height)

    private val height = 1000f
    private val speed = 500f

    @Test
    fun slowReleaseChoosesTheNearestHeight() {
        assertEquals(0f, cardSheetTarget(200f, 0f, speed, height), 0f)
        assertEquals(500f, cardSheetTarget(300f, 0f, speed, height), 0f)
    }

    @Test
    fun downwardFlingCollapsesBeforeDismissing() {
        assertEquals(500f, cardSheetTarget(0f, 700f, speed, height), 0f)
        assertEquals(height, cardSheetTarget(500f, 700f, speed, height), 0f)
    }

    @Test
    fun upwardFlingExpandsOrReturnsToTheNormalHeight() {
        assertEquals(0f, cardSheetTarget(500f, -700f, speed, height), 0f)
        assertEquals(500f, cardSheetTarget(700f, -700f, speed, height), 0f)
    }

    @Test
    fun slowDragDismissesOnlyPastThirtyPercentBelowNormal() {
        assertEquals(500f, cardSheetTarget(650f, 0f, speed, height), 0f)
        assertEquals(height, cardSheetTarget(651f, 0f, speed, height), 0f)
    }

    @Test
    fun velocityBelowTheThresholdUsesDistance() {
        assertEquals(500f, cardSheetTarget(500f, 499f, speed, height), 0f)
        assertEquals(0f, cardSheetTarget(0f, -499f, speed, height), 0f)
    }

    @Test
    fun aDragThatScrolledContentCannotPullTheSheetDownAtItsTop() = runTest {
        val position = CardSheetPosition()
        position.apply { half = height / 2f; fills = true }.measured(height.toInt(), backgroundScope, false)
        position.dragBy(-height)
        val scroll = CardSheetScroll(position, backgroundScope, speed, { false }) { error("dismissed") }
        scroll.onPreFling(Velocity.Zero)

        scroll.onPostScroll(Offset(0f, 100f), Offset.Zero, NestedScrollSource.UserInput)
        assertEquals(Offset.Zero, scroll.onPostScroll(Offset.Zero, Offset(0f, 100f), NestedScrollSource.UserInput))
        assertEquals(0f, position.offset, 0f)
        assertEquals(Velocity.Zero, scroll.onPreFling(Velocity(0f, 1000f)))

        // After the finger lifts, a new pull from the top belongs to the sheet.
        assertEquals(Offset(0f, 100f), scroll.onPostScroll(Offset.Zero, Offset(0f, 100f), NestedScrollSource.UserInput))
        assertEquals(100f, position.offset, 0f)
    }

    @Test
    fun anUpwardDragRaisesTheSheetAndLetsItsFlingReachTheContent() = runTest {
        val position = CardSheetPosition()
        position.apply { half = height / 2f; fills = true }.measured(height.toInt(), backgroundScope, false)
        position.dragBy(0f)
        val scroll = CardSheetScroll(position, backgroundScope, speed, { false }) { error("dismissed") }

        assertEquals(Offset(0f, -100f), scroll.onPreScroll(Offset(0f, -100f), NestedScrollSource.UserInput))
        assertEquals(Velocity.Zero, scroll.onPreFling(Velocity(0f, -1000f)))
        assertEquals(0f, position.offset, 0f)
    }

    @Test
    fun flingingToTheTopDoesNotMoveTheSheet() = runTest {
        val position = CardSheetPosition()
        position.apply { half = height / 2f; fills = true }.measured(height.toInt(), backgroundScope, false)
        position.dragBy(-height)
        val scroll = CardSheetScroll(position, backgroundScope, speed, { false }) { error("dismissed") }
        scroll.onPreFling(Velocity.Zero)

        assertEquals(Offset.Zero, scroll.onPostScroll(Offset.Zero, Offset(0f, 100f), NestedScrollSource.SideEffect))
        assertEquals(0f, position.offset, 0f)
    }
}

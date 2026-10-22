package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlin.math.roundToInt

/** Size of the add control retained on Spaces. */
val SEARCH_BAR_HEIGHT = 55.dp

/** Spacing used by the existing Spaces and card grid controls. */
val PAGE_GUTTER = 10.dp

private val BAR_RADIUS = 6.dp
private const val PLUS_ARM = 12.577f / 29f
private const val PLUS_STROKE = 3.354f / 29f
private const val PLUS_SHIFT = 0.0769f / 29f

/**
 * The orange 55 dp square with a white plus.
 *
 * @param onClick Called when tapped.
 * @param modifier Modifier for the button.
 * @param plus Size of the plus sign: 21 dp beside the search bar, 22 dp on the Spaces pages.
 * @param square Where the web page puts the square inside the laid-out box, in px, or null when it fills the box.
 *   The page places it at fractional pixels; its edges land on the nearest pixel, as the browser paints them, and
 *   the plus keeps the exact centre.
 */
@Composable
fun AddButton(onClick: () -> Unit, modifier: Modifier = Modifier, plus: Dp = 21.dp, square: Rect? = null) {
    val accent = MymindTheme.colors.accent
    Canvas(
        modifier
            .size(SEARCH_BAR_HEIGHT)
            .semantics { contentDescription = "Add" }
            .clickable(interactionSource = null, indication = null, role = Role.Button, onClick = onClick),
    ) {
        val exact = square ?: Rect(Offset.Zero, size)
        val edges = Rect(exact.left.snap(), exact.top.snap(), exact.right.snap(), exact.bottom.snap())
        drawRoundRect(accent, edges.topLeft, edges.size, CornerRadius(BAR_RADIUS.toPx()))
        val plusPx = plus.toPx()
        val centre = exact.center + Offset(plusPx * PLUS_SHIFT, plusPx * PLUS_SHIFT)
        val arm = plusPx * PLUS_ARM
        val stroke = plusPx * PLUS_STROKE
        drawLine(Color.White, Offset(centre.x - arm, centre.y), Offset(centre.x + arm, centre.y), stroke, StrokeCap.Round)
        drawLine(Color.White, Offset(centre.x, centre.y - arm), Offset(centre.x, centre.y + arm), stroke, StrokeCap.Round)
    }
}

/** The nearest whole pixel, halves rounding up as in the browser. */
private fun Float.snap(): Float = roundToInt().toFloat()

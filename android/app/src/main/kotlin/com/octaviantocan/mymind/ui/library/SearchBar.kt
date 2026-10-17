package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlin.math.roundToInt

/** Height of the search bar and of the + button beside it. */
val SEARCH_BAR_HEIGHT = 55.dp

/** Space between the top of the page and the search bar, and around the grid. */
val PAGE_GUTTER = 10.dp

private val BAR_RADIUS = 6.dp
private val ADD_GAP = 5.dp
private val BAR_SHADOW_LIGHT = Shadow(radius = 43.dp, color = Color(0xFF363A40), offset = DpOffset(8.dp, 17.dp), alpha = 0.15f)
private val BAR_SHADOW_DARK = Shadow(radius = 43.dp, color = Color.Black, offset = DpOffset(8.dp, 17.dp), alpha = 0.5f)

/** Half the length of a plus arm as a part of the plus size (`plus.svg`, 29 units). */
private const val PLUS_ARM = 12.577f / 29f

/** Stroke width of the plus as a part of the plus size. */
private const val PLUS_STROKE = 3.354f / 29f

/** How far the plus's centre sits right of and below the middle of `plus.svg`, as a part of the plus size. */
private const val PLUS_SHIFT = 0.0769f / 29f

/**
 * The search bar at the top of the Everything tab: the hamburger that opens settings, the centred placeholder and
 * the orange + button. Focused, it fills the row and shows a clear button instead.
 *
 * @param query Text in the field.
 * @param focused True while the field has focus; the bar then fills the row.
 * @param onQueryChange Called with the new text.
 * @param onFocusChange Called when the field gains or loses focus.
 * @param onClear Called when the clear button is tapped.
 * @param onOpenSettings Called when the hamburger is tapped.
 * @param onAdd Called when the + button is tapped.
 * @param focusRequester Moves focus into the field.
 * @param modifier Modifier for the row.
 */
@Composable
fun SearchBar(
    query: String,
    focused: Boolean,
    onQueryChange: (String) -> Unit,
    onFocusChange: (Boolean) -> Unit,
    onClear: () -> Unit,
    onOpenSettings: () -> Unit,
    onAdd: () -> Unit,
    focusRequester: FocusRequester,
    modifier: Modifier = Modifier,
) {
    val page = MymindTheme.page
    val faint = searchFaint()
    val textStyle = TextStyle(color = page.text, fontSize = 19.sp, letterSpacing = (-0.03).em, fontFamily = MymindTheme.fonts.nunito)
    Row(modifier.fillMaxWidth().padding(start = PAGE_GUTTER, end = PAGE_GUTTER, top = PAGE_GUTTER)) {
        // Drawn above the + button, whose left part lies in the bar's shadow as on the web page.
        Box(
            Modifier
                .weight(1f)
                .zIndex(1f)
                .height(SEARCH_BAR_HEIGHT)
                .dropShadow(RoundedCornerShape(BAR_RADIUS), if (MymindTheme.colors.dark) BAR_SHADOW_DARK else BAR_SHADOW_LIGHT)
                .clip(RoundedCornerShape(BAR_RADIUS))
                .background(page.surface),
            contentAlignment = Alignment.CenterStart,
        ) {
            BasicTextField(
                value = query,
                onValueChange = onQueryChange,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = if (focused) 15.dp else 20.dp, end = if (focused) 50.dp else 0.dp)
                    .focusRequester(focusRequester)
                    .onFocusChanged { onFocusChange(it.isFocused) },
                textStyle = textStyle,
                singleLine = true,
                cursorBrush = SolidColor(Color.White),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = KeyboardActions(),
                decorationBox = { field ->
                    Box(contentAlignment = Alignment.CenterStart) {
                        if (query.isEmpty() && !focused) {
                            Text("Search my mind...", Modifier.fillMaxWidth(), style = textStyle.copy(color = faint, textAlign = TextAlign.Center))
                        }
                        field()
                    }
                },
            )
            if (!focused) Hamburger(faint, onOpenSettings, Modifier.padding(start = 15.dp))
            if (focused) {
                val haptic = LocalHapticFeedback.current
                ClearButton(
                    {
                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                        onClear()
                    },
                    Modifier.align(Alignment.CenterEnd).padding(end = 11.dp),
                )
            }
        }
        if (!focused) {
            Spacer(Modifier.width(ADD_GAP))
            AddButton(onAdd)
        }
    }
}

/** Colour of the hamburger and the placeholder: 60% of the muted grey. */
@Composable
private fun searchFaint(): Color =
    if (MymindTheme.colors.dark) Color(0xFF626C7A).copy(alpha = 0.6f) else Color(0xFF748297).copy(alpha = 0.6f)

/** The 25 x 17 dp three-line icon that opens settings, with a 15 dp touch margin. */
@Composable
private fun Hamburger(color: Color, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Canvas(
        modifier
            .size(width = 25.dp, height = 17.dp)
            .semantics { contentDescription = "Settings" }
            .clickable(interactionSource = null, indication = null, role = Role.Button, onClick = onClick),
    ) {
        val thickness = 2.3.dp.toPx()
        listOf(0f, 0.5f, 1f).forEach { at ->
            val y = thickness / 2 + at * (size.height - thickness)
            drawLine(color, Offset(thickness / 2, y), Offset(size.width - thickness / 2, y), thickness, StrokeCap.Round)
        }
    }
}

/** The grey round button with a cross that empties the focused field. */
@Composable
private fun ClearButton(onClick: () -> Unit, modifier: Modifier = Modifier) {
    val dark = MymindTheme.colors.dark
    val fill = if (dark) Color(0xFF343941) else Color(0xFFE3E8EE)
    val ink = if (dark) Color(0xFFA7B4C6) else Color(0xFF748297)
    Canvas(
        modifier
            .size(25.dp)
            .clip(CircleShape)
            .semantics { contentDescription = "Clear" }
            .clickable(role = Role.Button, onClick = onClick),
    ) {
        drawCircle(fill)
        val arm = 4.5.dp.toPx()
        val stroke = 1.6.dp.toPx()
        drawLine(ink, Offset(center.x - arm, center.y - arm), Offset(center.x + arm, center.y + arm), stroke, StrokeCap.Round)
        drawLine(ink, Offset(center.x - arm, center.y + arm), Offset(center.x + arm, center.y - arm), stroke, StrokeCap.Round)
    }
}

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

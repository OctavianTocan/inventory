package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

/** Neutral gallery surfaces; card-specific accents remain on each card and its action sheet. */
@Immutable
data class LibraryColors(
    val background: Color,
    val raised: Color,
    val selected: Color,
    val text: Color,
    val muted: Color,
    val accent: Color,
    val shadow: Color,
)

private val DarkLibraryColors = LibraryColors(
    background = Color(0xFF10100F), raised = Color(0xFF1F1F1D), selected = Color(0xFF4C432F),
    text = Color(0xFFF4F2EC), muted = Color(0xFFB5B2A8), accent = Color(0xFFE2BC78), shadow = Color.Black,
)
private val LightLibraryColors = LibraryColors(
    background = Color(0xFFF8F7F3), raised = Color.White, selected = Color(0xFFEADFCA),
    text = Color(0xFF262520), muted = Color(0xFF716E64), accent = Color(0xFF745B28), shadow = Color(0xFF262520),
)

/** Shared type and surfaces for the gallery header, tiles, and floating navigation. */
object LibraryStyle {
    val colors: LibraryColors
        @Composable @ReadOnlyComposable get() = if (MymindTheme.colors.dark) DarkLibraryColors else LightLibraryColors

    val font = AppFontFamily
    val label = TextStyle(fontFamily = font, fontSize = 13.sp, lineHeight = 18.sp)
    val body = TextStyle(fontFamily = font, fontSize = 15.sp, lineHeight = 21.sp)
    val title = TextStyle(fontFamily = font, fontWeight = FontWeight.SemiBold, fontSize = 20.sp, lineHeight = 26.sp)
}

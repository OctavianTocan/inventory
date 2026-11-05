package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

/** Pully's paper and floating control colors, following the card's light or dark appearance. */
@Immutable
class CardSheetColors(val paper: Color, val raised: Color, val text: Color, val grabber: Color, val scrim: Color, val active: Color, val shadow: Color)

/** ThyMind's own sheet colours in the light look. */
internal val LightCardSheetColors = CardSheetColors(Color(0xFFF4F2EF), Color.White, Color(0xFF0E333A), Color(0xFFD9D9D6), Color(0x610E333A), Color(0x1A0E333A), Color(0xFF0E333A))

/** ThyMind's own sheet colours in the dark look. */
internal val DarkCardSheetColors = CardSheetColors(Color(0xFF1B1B1B), Color(0xFF1D1D1D), Color(0xFFE3E3E3), Color(0xFF3A3A3A), Color(0x99000000), Color(0x1FFFFFFF), Color.Black)

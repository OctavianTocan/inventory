package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight

/** Android's system sans-serif (Roboto on standard Android), shared by all app text. */
val AndroidFontFamily = FontFamily.SansSerif

/** App text roles; callers select weight with [FontWeight]. */
@Immutable
class MymindFonts(
    val heading: FontFamily,
    val body: FontFamily,
)

/** Fonts used by [MymindTheme]. */
val MymindDefaultFonts = MymindFonts(heading = AndroidFontFamily, body = AndroidFontFamily)

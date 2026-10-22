package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import com.octaviantocan.mymind.R

/**
 * Google Sans Flex, the typeface of Google's own phone apps, shared by all app text. The bundled file is a Latin subset
 * whose only variable axis is weight, from light to bold (`assets/font-licenses/GoogleSansFlex-SOURCE.md`).
 */
val AppFontFamily = FontFamily(
    listOf(FontWeight.Light, FontWeight.Normal, FontWeight.Medium, FontWeight.SemiBold, FontWeight.Bold).map { weight ->
        Font(R.font.google_sans_flex, weight, variationSettings = FontVariation.Settings(FontVariation.weight(weight.weight)))
    },
)

/** App text roles; callers select weight with [FontWeight]. */
@Immutable
class MymindFonts(
    val heading: FontFamily,
    val body: FontFamily,
)

/** Fonts used by [MymindTheme]. */
val MymindDefaultFonts = MymindFonts(heading = AppFontFamily, body = AppFontFamily)

package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import com.octaviantocan.mymind.R

/** Geist's real variable weights, shared by app text, Material components, and the gallery. */
val GeistFontFamily = FontFamily(
    (100..900 step 100).map { weight ->
        Font(
            R.font.geist,
            weight = FontWeight(weight),
            variationSettings = FontVariation.Settings(FontVariation.weight(weight)),
        )
    },
)

/** App text roles; callers select weight with [FontWeight]. Both roles use bundled Geist. */
@Immutable
class MymindFonts(
    val heading: FontFamily,
    val body: FontFamily,
)

/** Fonts used by [MymindTheme]. */
val MymindDefaultFonts = MymindFonts(heading = GeistFontFamily, body = GeistFontFamily)

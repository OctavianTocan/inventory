package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import com.octaviantocan.mymind.R

/**
 * The typefaces the app draws with. Nunito is one variable font; each Nunito family fixes its weight axis
 * and is used with the default font weight. The title face is Instrument Serif, still exposed as `louize`
 * so existing call sites keep that slot.
 *
 * @property louize Instrument Serif, used for large titles.
 * @property nunito Nunito at weight 400, the default text face.
 * @property nunitoMedium Nunito at weight 500.
 * @property nunitoSemiBold Nunito at weight 600.
 * @property nunitoBold Nunito at weight 700.
 */
@Immutable
class MymindFonts(
    val louize: FontFamily,
    val nunito: FontFamily,
    val nunitoMedium: FontFamily,
    val nunitoSemiBold: FontFamily,
    val nunitoBold: FontFamily,
)

private fun nunitoAt(weight: Int) = FontFamily(
    Font(R.font.nunito, variationSettings = FontVariation.Settings(FontVariation.weight(weight))),
)

/** Fonts used by [MymindTheme]. */
val MymindDefaultFonts = MymindFonts(
    louize = FontFamily(Font(R.font.instrument_serif_regular)),
    nunito = nunitoAt(400),
    nunitoMedium = nunitoAt(500),
    nunitoSemiBold = nunitoAt(600),
    nunitoBold = nunitoAt(700),
)

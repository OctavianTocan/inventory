package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

/**
 * Colours of the library and card pages, from the web front-end's stylesheets (`app.css` light, `dark.css` dark).
 *
 * @property background Page behind the grid (`--surface-color-2`).
 * @property surface Fill of card tiles, the search bar and chips (`mymind-card --surface-color`).
 * @property document Background of an opened card (`--document-surface-color`).
 * @property text Main text (`--color`).
 * @property text2 Secondary text, such as card text in the grid (`--color-2`).
 * @property heading Headlines on article and repository tiles.
 * @property faint Post authors and other quiet labels on tiles.
 * @property muted Captions under tiles, placeholders and space names (`#748297`).
 * @property icon Platform logos on tiles (`--card-color`).
 * @property link Links inside posts (`#4080DF`).
 * @property border Hairlines and stack edges (`--border-color`).
 */
@Immutable
class PageColors(
    val background: Color,
    val surface: Color,
    val document: Color,
    val text: Color,
    val text2: Color,
    val heading: Color,
    val faint: Color,
    val muted: Color,
    val icon: Color,
    val link: Color,
    val border: Color,
)

/** Page colours of the dark appearance. */
val DarkPageColors = PageColors(
    background = Color(0xFF14161E),
    surface = Color(0xFF1D1E28),
    document = Color(0xFF0A0B0F),
    text = Color(0xFFFFFFFF),
    text2 = Color(0xFFA7B4C6),
    heading = Color(0xFFA7B4C6),
    faint = Color(0xFF626C7A),
    muted = Color(0xFF748297),
    icon = Color(0xFF5C697B),
    link = Color(0xFF4080DF),
    border = Color(0xFF33384E),
)

/** Page colours of the light appearance. */
val LightPageColors = PageColors(
    background = Color(0xFFF0F2F5),
    surface = Color(0xFFFFFFFF),
    document = Color(0xFFFFFFFF),
    text = Color(0xFF000000),
    text2 = Color(0xFF000000),
    heading = Color(0xFF22232F),
    faint = Color(0xFF909CAE),
    muted = Color(0xFF748297),
    icon = Color(0xFF909CAE),
    link = Color(0xFF4080DF),
    border = Color(0xFFB8C3D3),
)

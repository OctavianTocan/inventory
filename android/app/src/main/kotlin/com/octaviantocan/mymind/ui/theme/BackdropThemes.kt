package com.octaviantocan.mymind.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.luminance

/**
 * ThyMind's own themes that paint the gallery with a gradient, and with doodles over it in the manner of a chat app's
 * wallpaper. Their other colours are drawn from the gradient, its ink and its accent the way the OpenCode themes are
 * drawn from theirs, so sheets, tiles and pages read the same as under any theme.
 */
internal val BackdropThemes: List<ColorTheme> = listOf(
    backdropTheme("sunrise", "Sunrise",
        light = Look(listOf(Color(0xFFFFE3C8), Color(0xFFFFCCD8), Color(0xFFE6D6FF)), ink = Color(0xFF3A2430), accent = Color(0xFFE0533D)),
        dark = Look(listOf(Color(0xFF2E1C2A), Color(0xFF3A1E30), Color(0xFF1F1B3C)), ink = Color(0xFFF6E6EC), accent = Color(0xFFFF8A70))),
    backdropTheme("ocean", "Ocean",
        light = Look(listOf(Color(0xFFD5F3F2), Color(0xFFD3E4FB)), ink = Color(0xFF0F2E3A), accent = Color(0xFF0E8C94)),
        dark = Look(listOf(Color(0xFF06303A), Color(0xFF0B1C3C)), ink = Color(0xFFE2F4F7), accent = Color(0xFF4FD1D9))),
    backdropTheme("lavender", "Lavender",
        light = Look(listOf(Color(0xFFF1E8FF), Color(0xFFE3EEFF)), ink = Color(0xFF261F3D), accent = Color(0xFF7A5AF8)),
        dark = Look(listOf(Color(0xFF211936), Color(0xFF15223D)), ink = Color(0xFFEDE8FB), accent = Color(0xFFB59CFF))),
    backdropTheme("aurora", "Aurora",
        light = Look(listOf(Color(0xFF0A2126), Color(0xFF0F2C3C), Color(0xFF261B42)), ink = Color(0xFFE6F5F0), accent = Color(0xFF6EF2B4))),
    backdropTheme("notebook", "Notebook",
        light = Look(listOf(Color(0xFFF7F1E3)), ink = Color(0xFF3A3328), accent = Color(0xFFC1602A), doodles = Doodles.Notebook),
        dark = Look(listOf(Color(0xFF1F1C17)), ink = Color(0xFFEDE5D2), accent = Color(0xFFF0A266), doodles = Doodles.Notebook)),
    backdropTheme("garden", "Garden",
        light = Look(listOf(Color(0xFFE5F2DD), Color(0xFFF6F0DA)), ink = Color(0xFF213A26), accent = Color(0xFF3E8E4E), doodles = Doodles.Garden),
        dark = Look(listOf(Color(0xFF13241A), Color(0xFF1F2A1A)), ink = Color(0xFFE0EFD9), accent = Color(0xFF7DD08B), doodles = Doodles.Garden)),
    backdropTheme("night-sky", "Night Sky",
        light = Look(listOf(Color(0xFF0E1433), Color(0xFF1D1547)), ink = Color(0xFFEEEAFB), accent = Color(0xFFF2C14E), doodles = Doodles.NightSky)),
    backdropTheme("postcard", "Postcard",
        light = Look(listOf(Color(0xFFDCEBFF), Color(0xFFFFE0EA)), ink = Color(0xFF22283D), accent = Color(0xFF3D6BE0), doodles = Doodles.Postcard),
        dark = Look(listOf(Color(0xFF14203C), Color(0xFF2E1A2E)), ink = Color(0xFFE9ECFA), accent = Color(0xFF8FB0FF), doodles = Doodles.Postcard)),
)

/**
 * One look of a backdrop theme, from which all its colours are drawn.
 *
 * @property gradient The gradient behind the gallery, top left to bottom right.
 * @property ink The main text colour; the doodles are drawn in it, faintly.
 * @property accent The theme's own colour.
 * @property doodles The drawings over the gradient, or null for none.
 */
internal class Look(val gradient: List<Color>, val ink: Color, val accent: Color, val doodles: DoodleSet? = null)

/** A theme from its looks; with no [dark] look, [light] is used for both, as a one-look OpenCode theme is. */
private fun backdropTheme(id: String, name: String, light: Look, dark: Look? = null): ColorTheme {
    val first = palette(light)
    return ColorTheme(id, name, first, dark?.let(::palette))
}

/** Every ThyMind colour of [look]. */
private fun palette(look: Look): ThemePalette {
    val roles = roles(look)
    val doodleInk = look.ink.copy(alpha = if (roles.dark) 0.12f else 0.14f)
    return ThemePalette.of(roles, Backdrop(look.gradient, look.doodles, doodleInk))
}

/** [look]'s roles: the screen is the gradient's middle, so edges, bars and fades sit close to it. */
private fun roles(look: Look): ThemeRoles {
    val base = middle(look.gradient)
    val text = look.ink
    val dark = base.luminance() < text.luminance()
    val paper = if (dark) lerp(base, text, 0.06f) else lerp(base, Color.White, 0.55f)
    val wash = lerp(base, look.accent, if (dark) 0.2f else 0.12f)
    return ThemeRoles(
        dark = dark,
        base = base,
        text = text,
        muted = lerp(text, base, 0.42f),
        faint = lerp(text, base, 0.6f),
        border = lerp(base, text, 0.14f),
        raised = if (dark) lerp(base, text, 0.09f) else lerp(base, Color.White, 0.75f),
        sunken = lerp(base, text, 0.07f),
        paper = paper,
        accent = look.accent,
        fill = readable(look.accent, Color.White, 3f, Color.Black),
        error = if (dark) Color(0xFFFF7A7F) else Color(0xFFC9393F),
        success = if (dark) Color(0xFF5FD39A) else Color(0xFF1C8A5A),
        warning = if (dark) Color(0xFFF2A14B) else Color(0xFFC46F12),
        link = look.accent,
        accentWash = wash,
        accentWashInk = readable(look.accent, wash, 4.5f, text),
        scrim = if (dark) Color.Black.copy(alpha = 0.6f) else text.copy(alpha = 0.38f),
        grabber = lerp(paper, text, 0.15f),
        shadow = if (dark) Color.Black else text,
    )
}

/** The colour halfway along [gradient]. */
private fun middle(gradient: List<Color>): Color = when (gradient.size) {
    1 -> gradient[0]
    2 -> lerp(gradient[0], gradient[1], 0.5f)
    else -> gradient[gradient.size / 2]
}

/** WCAG 2 contrast ratio of two opaque colours. */
internal fun contrast(x: Color, y: Color): Float {
    val (hi, lo) = listOf(x.luminance(), y.luminance()).sortedDescending()
    return (hi + 0.05f) / (lo + 0.05f)
}

/** [color] stepped toward [toward] until it reads against [against] at [ratio], or as far as it goes. */
private fun readable(color: Color, against: Color, ratio: Float, toward: Color): Color {
    var t = 0f
    while (t <= 1f) {
        val next = lerp(color, toward, t)
        if (contrast(next, against) >= ratio) return next
        t += 0.02f
    }
    return toward
}

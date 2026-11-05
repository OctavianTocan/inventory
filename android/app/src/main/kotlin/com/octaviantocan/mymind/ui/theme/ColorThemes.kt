package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp

/**
 * One look of a colour theme in the few roles Pully's themes are made of, from which every ThyMind palette is drawn
 * ([ThemePalette.of]). The OpenCode themes are generated into these by `android/scripts/gen-opencode-themes.mjs`.
 *
 * @property dark True when the look really is dark, whatever look was asked for (a one-look theme).
 * @property base Screen behind everything.
 * @property text Main ink.
 * @property muted Secondary ink.
 * @property faint Quiet labels.
 * @property border Hairlines.
 * @property raised Tiles, cards and the rows of a sheet.
 * @property sunken Wells and the track of a switch that is off.
 * @property paper Sheets.
 * @property accent The theme's own colour: selection, ticks, links in the changelog.
 * @property fill The accent deepened until white ink reads on it.
 * @property error Destructive actions.
 * @property success Success.
 * @property warning Warnings.
 * @property link Links inside posts.
 * @property accentWash A faint wash of the accent on [base].
 * @property accentWashInk The accent as ink on [accentWash].
 * @property scrim Shade behind a sheet.
 * @property grabber A sheet's grab handle.
 * @property shadow Colour of shadows.
 */
@Immutable
class ThemeRoles(
    val dark: Boolean,
    val base: Color,
    val text: Color,
    val muted: Color,
    val faint: Color,
    val border: Color,
    val raised: Color,
    val sunken: Color,
    val paper: Color,
    val accent: Color,
    val fill: Color,
    val error: Color,
    val success: Color,
    val warning: Color,
    val link: Color,
    val accentWash: Color,
    val accentWashInk: Color,
    val scrim: Color,
    val grabber: Color,
    val shadow: Color,
)

/**
 * Colours of sheets that are not the sheet's paper, rows and ink ([CardSheetColors]).
 *
 * @property muted Secondary ink, such as the changelog's dates.
 * @property border Hairlines between the changelog's versions.
 * @property accent The theme's colour on a sheet.
 * @property wash A faint wash of the accent, under the changelog's New badge.
 * @property washInk The accent as ink on [wash].
 * @property danger Ink of a destructive row.
 * @property switchOff Track of a switch that is off.
 */
@Immutable
class SheetInk(
    val muted: Color,
    val border: Color,
    val accent: Color,
    val wash: Color,
    val washInk: Color,
    val danger: Color,
    val switchOff: Color,
)

/**
 * Every colour ThyMind draws in one look of one theme.
 *
 * @property colors The native chrome's colours.
 * @property page The library and card pages' colours.
 * @property library The gallery's surfaces.
 * @property sheet Pully's sheet paper and controls.
 * @property sheetInk Other colours on sheets.
 * @property dots The theme in four colours for the picker: its screen, accent, success and error.
 */
@Immutable
class ThemePalette(
    val colors: MymindColors,
    val page: PageColors,
    val library: LibraryColors,
    val sheet: CardSheetColors,
    val sheetInk: SheetInk,
    val dots: List<Color>,
) {
    /** True when this look is dark. */
    val dark: Boolean get() = colors.dark

    companion object {
        /**
         * Draws every ThyMind palette from a theme's roles, the way Pully maps its roles onto its own screens: one
         * screen colour behind the gallery, the pages and the chrome, the theme's accent wherever ThyMind's orange or
         * Pully's teal would be. The splash keeps ThyMind's own colours, as Pully's launch screen does.
         *
         * @param r One look of a theme.
         * @return That look as ThyMind's palettes.
         */
        fun of(r: ThemeRoles): ThemePalette {
            val splash = if (r.dark) MymindDarkColors else MymindLightColors
            val secondary = lerp(r.text, r.muted, 0.5f)
            val colors = MymindColors(
                dark = r.dark, page = r.base, tabBarFade = r.base, tabSelected = r.accent, tabUnselected = r.muted,
                muted = r.muted, raised = r.raised, accent = r.accent,
                onboardingBackground = splash.onboardingBackground, splashIconTint = splash.splashIconTint,
                danger = r.error, settingsSheet = r.base, settingsLabel = r.muted, settingsValue = r.text,
                settingsRowLabel = r.muted, settingsTrack = r.sunken, settingsHint = r.muted, settingsGreeting = r.text,
                switchUncheckedTrack = r.sunken, switchUncheckedBorder = r.muted, switchUncheckedThumb = r.muted,
                switchCheckedTrack = r.accent.copy(alpha = 0.2f), switchCheckedBorder = r.muted, switchCheckedThumb = r.muted,
                dangerLabel = lerp(r.error, Color.Black, 0.6f), dangerButton = r.error, queueItem = r.raised,
            )
            val page = PageColors(
                background = r.base, surface = r.raised,
                document = if (r.dark) lerp(r.base, Color.Black, 0.3f) else r.raised,
                text = r.text, text2 = secondary, heading = secondary, faint = r.faint, muted = r.muted, icon = r.faint,
                link = r.link, border = r.border,
            )
            val library = LibraryColors(
                background = r.base, raised = r.raised, selected = lerp(r.raised, r.accent, 0.25f), text = r.text,
                muted = r.muted, accent = r.accent, shadow = r.shadow,
            )
            val sheet = CardSheetColors(r.paper, r.raised, r.text, r.grabber, r.scrim,
                r.text.copy(alpha = if (r.dark) 0.12f else 0.1f), r.shadow)
            val ink = SheetInk(r.muted, r.border, r.accent, r.accentWash, r.accentWashInk, r.error, r.sunken)
            return ThemePalette(colors, page, library, sheet, ink, listOf(r.base, r.accent, r.success, r.error))
        }
    }
}

/**
 * A colour theme, as Pully's (K18): a whole palette for each look. The appearance still picks the look; the theme
 * picks the palette it is drawn in. [dark] is null for a theme with one look, which is then used for both, so a
 * dark-only theme stays dark when the light look is asked for ([ThemePalette.dark] says which look it really is).
 *
 * @property id What the phone stores: `thymind`, `pully`, or the OpenCode theme's own name (`tokyonight`).
 * @property name The name shown in the picker, never translated.
 */
@Immutable
class ColorTheme(val id: String, val name: String, private val light: ThemePalette, private val dark: ThemePalette? = null) {
    /** A theme from its roles; null [dark] uses [light] for both looks. */
    constructor(id: String, name: String, light: ThemeRoles, dark: ThemeRoles?) :
        this(id, name, ThemePalette.of(light), dark?.let(ThemePalette::of))

    /**
     * @param dark True when the dark look is asked for.
     * @return The palette for that look.
     */
    fun palette(dark: Boolean): ThemePalette = if (dark) this.dark ?: light else light
}

/** Pully's own palette (Pully `ui/design/Tokens.kt` at 9a44869) in ThyMind's roles. */
private val PullyLight = ThemeRoles(
    dark = false, base = Color(0xFFFDFCF9), text = Color(0xFF0E333A), muted = Color(0xFF6E8285), faint = Color(0xFF93A2A4),
    border = Color(0xFFE3E5E3), raised = Color(0xFFFFFFFF), sunken = Color(0xFFF1F0EC), paper = Color(0xFFF4F2EF),
    accent = Color(0xFF24929C), fill = Color(0xFF1B7580), error = Color(0xFFC94F55), success = Color(0xFF1C8A5A),
    warning = Color(0xFFD9822B), link = Color(0xFF3B82F6), accentWash = Color(0xFFE6F1F2), accentWashInk = Color(0xFF165E66),
    scrim = Color(0x610E333A), grabber = Color(0xFFD9D9D6), shadow = Color(0xFF0E333A),
)
private val PullyDark = ThemeRoles(
    dark = true, base = Color(0xFF161616), text = Color(0xFFE3E3E3), muted = Color(0xFF9D9D9D), faint = Color(0xFF707070),
    border = Color(0xFF2B2B2B), raised = Color(0xFF1D1D1D), sunken = Color(0xFF212121), paper = Color(0xFF1B1B1B),
    accent = Color(0xFF4FB7C1), fill = Color(0xFF1B7580), error = Color(0xFFE8777C), success = Color(0xFF3CC98F),
    warning = Color(0xFFF2A14B), link = Color(0xFF60A5FA), accentWash = Color(0xFF14363B), accentWashInk = Color(0xFF9AD3D6),
    scrim = Color(0x99000000), grabber = Color(0xFF3A3A3A), shadow = Color(0xFF000000),
)

private val ThyMindLight = ThemePalette(
    MymindLightColors, LightPageColors, LightLibraryColors, LightCardSheetColors,
    SheetInk(Color(0xFF6E8285), Color(0xFFECEBE7), Color(0xFF24929C), Color(0xFFE6F1F2), Color(0xFF165E66),
        Color(0xFFB4233A), Color(0xFFE3E5E3)),
    listOf(MymindLightColors.page, MymindLightColors.accent, Color(0xFF1C8A5A), MymindLightColors.danger),
)
private val ThyMindDark = ThemePalette(
    MymindDarkColors, DarkPageColors, DarkLibraryColors, DarkCardSheetColors,
    SheetInk(Color(0xFF9D9D9D), Color(0xFF242424), Color(0xFF4FB7C1), Color(0xFF14363B), Color(0xFF9AD3D6),
        Color(0xFFFF98AA), Color(0xFF3A3A3A)),
    listOf(MymindDarkColors.page, MymindDarkColors.accent, Color(0xFF3CC98F), MymindDarkColors.danger),
)

/**
 * Every theme the app can be drawn in: ThyMind's own colours first, the default, then Pully's own palette, then the
 * themes Pully converted from OpenCode's ([OpenCodeThemes]).
 */
object ColorThemes {
    /** The id of ThyMind's own colours. */
    const val DEFAULT = "thymind"

    /** ThyMind's own colours, exactly as the app drew before themes. */
    val ThyMind = ColorTheme(DEFAULT, "ThyMind", ThyMindLight, ThyMindDark)

    /** Pully's own palette. */
    val Pully = ColorTheme("pully", "Pully", PullyLight, PullyDark)

    /** Every theme, in the picker's order. */
    val all: List<ColorTheme> = listOf(ThyMind, Pully) + OpenCodeThemes

    /**
     * @param id A stored theme id, or null.
     * @return The theme stored as [id], or ThyMind's when there is none by that name (a theme that was dropped since).
     */
    fun of(id: String?): ColorTheme = all.firstOrNull { it.id == id } ?: ThyMind
}

/** The theme the app is drawn in. */
val LocalColorTheme = staticCompositionLocalOf { ColorThemes.ThyMind }

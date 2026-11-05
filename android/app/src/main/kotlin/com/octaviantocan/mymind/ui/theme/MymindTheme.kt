package com.octaviantocan.mymind.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.text.TextStyle

private val LocalMymindColors = staticCompositionLocalOf<MymindColors> { error("MymindTheme is missing") }
private val LocalMymindFonts = staticCompositionLocalOf<MymindFonts> { error("MymindTheme is missing") }
private val LocalThemePalette = staticCompositionLocalOf<ThemePalette> { error("MymindTheme is missing") }
private val LocalChromeColors = staticCompositionLocalOf<MymindColors?> { null }

/** Access to the colours and fonts of the enclosing [MymindTheme]. */
object MymindTheme {
    /** Colours of the current appearance. */
    val colors: MymindColors
        @Composable @ReadOnlyComposable get() = LocalMymindColors.current

    /** Fonts of the app. */
    val fonts: MymindFonts
        @Composable @ReadOnlyComposable get() = LocalMymindFonts.current

    /** Colours of the library and card pages in the current appearance. */
    val page: PageColors
        @Composable @ReadOnlyComposable get() = LocalThemePalette.current.page

    /** Every colour of the current theme in the current appearance. */
    val palette: ThemePalette
        @Composable @ReadOnlyComposable get() = LocalThemePalette.current

    /** The colour theme the app is drawn in, for a part that needs its other look. */
    val theme: ColorTheme
        @Composable @ReadOnlyComposable get() = LocalColorTheme.current

    /** Colours of the native chrome, which follows the system even inside a [PageTheme]. */
    val chrome: MymindColors
        @Composable @ReadOnlyComposable get() = LocalChromeColors.current ?: LocalMymindColors.current
}

/** Appearance the account picked for library and card pages; native chrome always follows the system. */
enum class PageScheme {
    Light,
    Dark,
    System;

    /**
     * @param systemDark True when the system is in dark mode.
     * @return True when the pages draw dark.
     */
    fun isDark(systemDark: Boolean): Boolean = when (this) {
        Light -> false
        Dark -> true
        System -> systemDark
    }

    companion object {
        /**
         * @param value The account's `colorScheme` setting, or null when the server did not send one.
         * @return The matching scheme; anything but `light` or `dark` follows the system, as the web front-end does.
         */
        fun of(value: String?): PageScheme = when (value) {
            "light" -> Light
            "dark" -> Dark
            else -> System
        }
    }
}

/**
 * Applies the appearance to library and card pages inside a [MymindTheme] that follows the system: the one picked on
 * this phone in Profile, else the account's own.
 *
 * @param scheme Appearance the account picked.
 * @param content The page.
 */
@Composable
fun PageTheme(scheme: PageScheme, content: @Composable () -> Unit) {
    val chrome = MymindTheme.chrome
    val picked = LocalThemePrefs.current?.appearance ?: scheme
    MymindTheme(dark = picked.isDark(chrome.dark)) {
        CompositionLocalProvider(LocalChromeColors provides chrome, content = content)
    }
}

/**
 * Applies the app's colours and fonts, and a Material 3 theme built from them for the stock components. The colours
 * are [theme]'s palette for the look [dark] asks for; a one-look theme draws its one look either way, and
 * [MymindColors.dark] then says which look is really on screen.
 *
 * @param dark True for the dark appearance; follows the system setting by default.
 * @param theme The colour theme; the enclosing one by default, ThyMind's own outside any.
 * @param content Content drawn with the theme.
 */
@Composable
fun MymindTheme(dark: Boolean = isSystemInDarkTheme(), theme: ColorTheme = LocalColorTheme.current, content: @Composable () -> Unit) {
    val palette = theme.palette(dark)
    val colors = palette.colors
    val fonts = MymindDefaultFonts
    val scheme = if (colors.dark) {
        darkColorScheme(primary = colors.accent, background = colors.page, surface = colors.page, error = colors.danger)
    } else {
        lightColorScheme(primary = colors.accent, background = colors.page, surface = colors.page, error = colors.danger)
    }
    CompositionLocalProvider(
        LocalColorTheme provides theme,
        LocalMymindColors provides colors,
        LocalMymindFonts provides fonts,
        LocalThemePalette provides palette,
    ) {
        MaterialTheme(colorScheme = scheme, typography = appTypography(fonts), content = content)
    }
}

private fun appTypography(fonts: MymindFonts): Typography {
    val base = Typography()
    fun TextStyle.appFont() = copy(fontFamily = fonts.body)
    return Typography(
        displayLarge = base.displayLarge.appFont(),
        displayMedium = base.displayMedium.appFont(),
        displaySmall = base.displaySmall.appFont(),
        headlineLarge = base.headlineLarge.appFont(),
        headlineMedium = base.headlineMedium.appFont(),
        headlineSmall = base.headlineSmall.appFont(),
        titleLarge = base.titleLarge.appFont(),
        titleMedium = base.titleMedium.appFont(),
        titleSmall = base.titleSmall.appFont(),
        bodyLarge = base.bodyLarge.appFont(),
        bodyMedium = base.bodyMedium.appFont(),
        bodySmall = base.bodySmall.appFont(),
        labelLarge = base.labelLarge.appFont(),
        labelMedium = base.labelMedium.appFont(),
        labelSmall = base.labelSmall.appFont(),
    )
}

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
private val LocalPageColors = staticCompositionLocalOf<PageColors> { error("MymindTheme is missing") }
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
        @Composable @ReadOnlyComposable get() = LocalPageColors.current

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
 * Applies the account's appearance to library and card pages inside a [MymindTheme] that follows the system.
 *
 * @param scheme Appearance the account picked.
 * @param content The page.
 */
@Composable
fun PageTheme(scheme: PageScheme, content: @Composable () -> Unit) {
    val chrome = MymindTheme.chrome
    MymindTheme(dark = scheme.isDark(chrome.dark)) {
        CompositionLocalProvider(LocalChromeColors provides chrome, content = content)
    }
}

/**
 * Applies the app's colours and fonts, and a Material 3 theme built from them for the stock components.
 *
 * @param dark True for the dark appearance; follows the system setting by default.
 * @param content Content drawn with the theme.
 */
@Composable
fun MymindTheme(dark: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    val colors = if (dark) MymindDarkColors else MymindLightColors
    val fonts = MymindDefaultFonts
    val scheme = if (dark) {
        darkColorScheme(primary = colors.accent, background = colors.page, surface = colors.page)
    } else {
        lightColorScheme(primary = colors.accent, background = colors.page, surface = colors.page)
    }
    CompositionLocalProvider(
        LocalMymindColors provides colors,
        LocalMymindFonts provides fonts,
        LocalPageColors provides if (dark) DarkPageColors else LightPageColors,
    ) {
        MaterialTheme(colorScheme = scheme, typography = geistTypography(fonts), content = content)
    }
}

private fun geistTypography(fonts: MymindFonts): Typography {
    val base = Typography()
    fun TextStyle.geist() = copy(fontFamily = fonts.body)
    return Typography(
        displayLarge = base.displayLarge.geist(),
        displayMedium = base.displayMedium.geist(),
        displaySmall = base.displaySmall.geist(),
        headlineLarge = base.headlineLarge.geist(),
        headlineMedium = base.headlineMedium.geist(),
        headlineSmall = base.headlineSmall.geist(),
        titleLarge = base.titleLarge.geist(),
        titleMedium = base.titleMedium.geist(),
        titleSmall = base.titleSmall.geist(),
        bodyLarge = base.bodyLarge.geist(),
        bodyMedium = base.bodyMedium.geist(),
        bodySmall = base.bodySmall.geist(),
        labelLarge = base.labelLarge.geist(),
        labelMedium = base.labelMedium.geist(),
        labelSmall = base.labelSmall.geist(),
    )
}

package com.octaviantocan.mymind.ui.theme

import android.content.Context
import android.content.SharedPreferences
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf

/**
 * The colour theme and the appearance picked on this phone. They belong to the phone, not the account, so they stay
 * through a sign-out, and every screen that reads them redraws the moment they change.
 */
class ThemePrefs(private val prefs: SharedPreferences) {
    constructor(context: Context) : this(context.applicationContext.getSharedPreferences(NAME, Context.MODE_PRIVATE))

    /** The stored theme id; ThyMind's own until one is picked. */
    var id: String by mutableStateOf(prefs.getString(KEY, null) ?: ColorThemes.DEFAULT)
        private set

    /**
     * Light, Dark or System as picked in Profile, for the whole app: the native chrome and the pages. Null until one is
     * picked, when the chrome follows the system and the pages the account's own appearance, as before the choice.
     */
    var appearance: PageScheme? by mutableStateOf(prefs.getString(APPEARANCE, null)?.let(PageScheme::of))
        private set

    /**
     * Keeps [scheme] as the phone's appearance.
     *
     * @param scheme The picked appearance.
     */
    fun chooseAppearance(scheme: PageScheme) {
        appearance = scheme
        prefs.edit().putString(APPEARANCE, scheme.name.lowercase()).apply()
    }

    /**
     * @param systemDark True when the system is in dark mode.
     * @return True when the native chrome draws dark: the picked appearance, else the system.
     */
    fun chromeDark(systemDark: Boolean): Boolean = appearance?.isDark(systemDark) ?: systemDark

    /** The theme [id] names, or ThyMind's own when it names none any more. */
    val theme: ColorTheme get() = ColorThemes.of(id)

    /**
     * Keeps [theme] as the phone's theme and redraws the app in it.
     *
     * @param theme The picked theme.
     */
    fun choose(theme: ColorTheme) {
        id = theme.id
        prefs.edit().putString(KEY, theme.id).apply()
    }

    private companion object {
        const val NAME = "theme"
        const val KEY = "colorTheme"
        const val APPEARANCE = "appearance"
    }
}

/** The phone's theme setting for the pages that change it, or null where none can (previews, screenshot fixtures). */
val LocalThemePrefs = staticCompositionLocalOf<ThemePrefs?> { null }

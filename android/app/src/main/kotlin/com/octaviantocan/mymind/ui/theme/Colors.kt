package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

/**
 * Colours of the app in one appearance, with the values the original app draws.
 *
 * @property dark True for the dark appearance.
 * @property page Background behind every screen.
 * @property tabBarFade Colour the tab bar's bottom gradient fades into.
 * @property tabSelected Label colour of the selected tab.
 * @property tabUnselected Label colour of the other tabs.
 * @property muted Colour of secondary text on screens without a tab bar.
 * @property raised Fill of raised panels and pills.
 * @property accent Brand orange used for selection and calls to action.
 * @property onboardingBackground Background of the splash and onboarding screens.
 * @property splashIconTint Tint of the logo on the splash screen.
 * @property danger Red of destructive actions; also the fill of the settings danger zone.
 * @property settingsSheet Fill of the settings drawer.
 * @property settingsLabel Colour of the small labels in the settings drawer.
 * @property settingsValue Colour of the values and row titles in the settings drawer.
 * @property settingsRowLabel Colour of the action rows in the settings drawer.
 * @property settingsTrack Track colour of the switches in the settings drawer.
 * @property settingsHint Colour of hints and the footer in the settings drawer.
 * @property settingsGreeting Colour of the first line of the settings greeting.
 * @property switchUncheckedTrack Track of a switch that is off.
 * @property switchUncheckedBorder Border of a switch that is off.
 * @property switchUncheckedThumb Thumb of a switch that is off; also tints the trailing icons of settings rows.
 * @property switchCheckedTrack Track of a switch that is on.
 * @property switchCheckedBorder Border of a switch that is on.
 * @property switchCheckedThumb Thumb of a switch that is on.
 * @property dangerLabel Colour of the danger zone caption.
 * @property dangerButton Fill of the account deletion button.
 */
@Immutable
class MymindColors(
    val dark: Boolean,
    val page: Color,
    val tabBarFade: Color,
    val tabSelected: Color,
    val tabUnselected: Color,
    val muted: Color,
    val raised: Color,
    val accent: Color,
    val onboardingBackground: Color,
    val splashIconTint: Color,
    val danger: Color,
    val settingsSheet: Color,
    val settingsLabel: Color,
    val settingsValue: Color,
    val settingsRowLabel: Color,
    val settingsTrack: Color,
    val settingsHint: Color,
    val settingsGreeting: Color,
    val switchUncheckedTrack: Color,
    val switchUncheckedBorder: Color,
    val switchUncheckedThumb: Color,
    val switchCheckedTrack: Color,
    val switchCheckedBorder: Color,
    val switchCheckedThumb: Color,
    val dangerLabel: Color,
    val dangerButton: Color,
)

/** Colours of the dark appearance. */
val MymindDarkColors = MymindColors(
    dark = true,
    page = Color(0xFF14161E),
    tabBarFade = Color(0xFF14161E),
    tabSelected = Color(0xFFFF5924),
    tabUnselected = Color(0xFF626C7A),
    muted = Color(0xFF626C7A),
    raised = Color(0xFF1D1E28),
    accent = Color(0xFFFF5924),
    onboardingBackground = Color(0xFFEEB7C9),
    splashIconTint = Color(0xFF000000),
    danger = Color(0xFFF90019),
    settingsSheet = Color(0xFF14161E),
    settingsLabel = Color(0xFFA7B4C6).copy(alpha = 0.7f),
    settingsValue = Color(0xFFFFFFFF),
    settingsRowLabel = Color(0xFF8493A9),
    settingsTrack = Color(0xFF000000).copy(alpha = 0.4f),
    settingsHint = Color(0xFF748297),
    settingsGreeting = Color(0xFFFFFFFF),
    switchUncheckedTrack = Color(0xFF000000).copy(alpha = 0.2f),
    switchUncheckedBorder = Color(0xFF8493A9),
    switchUncheckedThumb = Color(0xFF8493A9),
    switchCheckedTrack = Color(0xFFD4D4D4).copy(alpha = 0.2f),
    switchCheckedBorder = Color(0xFF8493A9),
    switchCheckedThumb = Color(0xFF8493A9),
    dangerLabel = Color(0xFF67000A),
    dangerButton = Color(0xFFC00013),
)

/** Colours of the light appearance. */
val MymindLightColors = MymindColors(
    dark = false,
    page = Color(0xFFF0F2F5),
    tabBarFade = Color(0xFFF0F2F5),
    tabSelected = Color(0xFFFF5924),
    tabUnselected = Color(0xFF748297),
    muted = Color(0xFF748297),
    raised = Color(0xFFFFFFFF),
    accent = Color(0xFFFF5924),
    onboardingBackground = Color(0xFFEEB7C9),
    splashIconTint = Color(0xFFFFFFFF),
    danger = Color(0xFFF90019),
    settingsSheet = Color(0xFFF0F2F5),
    settingsLabel = Color(0xFFFFFFFF).copy(alpha = 0.8f),
    settingsValue = Color(0xFF000000).copy(alpha = 0.7f),
    settingsRowLabel = Color(0xFF4C5F7C),
    settingsTrack = Color(0xFFFFFFFF).copy(alpha = 0.5f),
    settingsHint = Color(0xFF748297),
    settingsGreeting = Color(0xFF2C1A20),
    switchUncheckedTrack = Color(0xFFF0F2F5).copy(alpha = 0.2f),
    switchUncheckedBorder = Color(0xFFAEBBCF),
    switchUncheckedThumb = Color(0xFFA7B4C7),
    switchCheckedTrack = Color(0xFFFFFFFF),
    switchCheckedBorder = Color(0xFFAEBBCF),
    switchCheckedThumb = Color(0xFFA7B4C7),
    dangerLabel = Color(0xFF67000A),
    dangerButton = Color(0xFFC00013),
)

package com.octaviantocan.mymind.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.text.font.FontWeight
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.LocalCardSheetColors
import com.octaviantocan.mymind.ui.card.cardSurfaceShadow
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.theme.PageScheme
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.menuColors
import com.octaviantocan.mymind.ui.theme.ColorTheme
import com.octaviantocan.mymind.ui.theme.ColorThemes
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.theme.ThemePrefs
import com.octaviantocan.mymind.ui.theme.ThemeWipe
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** What the theme search field says while it is empty. */
const val FIND_A_THEME = "Find a theme"

/**
 * Pully's theme page (Pully `ui/settings/ThemePage.kt` at 9a44869): a search over every colour theme, ThyMind first,
 * each with dots of its screen, accent, success and error in the look on screen, and a tick on the one in use. A tap
 * applies it at once through the wipe and stays here, so themes can be tried one after another.
 *
 * @param prefs The phone's theme setting.
 */
@Composable
internal fun ColumnScope.ThemePage(prefs: ThemePrefs) {
    val dark = MymindTheme.colors.dark
    val motion = rememberSheetMotionEnabled()
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    val shown = ColorThemes.all.filter { it.name.contains(query.trim(), ignoreCase = true) }
    CardMenuGroup(Modifier.padding(top = 20.dp)) { ThemeSearch(query) { query = it } }
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        if (shown.isEmpty()) SheetRow(null, "No theme matches", null)
        shown.forEachIndexed { index, theme ->
            if (index > 0) SheetRowDivider(start = 16.dp)
            SheetRow(null, theme.name, { scope.applyColorTheme(prefs, theme, motion) }, role = Role.RadioButton) {
                ThemeDots(theme.palette(dark).dots)
                Tick(theme.id == prefs.id)
            }
        }
    }
}

/**
 * Applies [theme] through the wipe, from the top right when it sits further down the list than the theme on screen and
 * from the top left when it sits higher, as Pully does. Picking the theme in use again changes nothing.
 *
 * @param prefs The phone's theme setting.
 * @param theme The picked theme.
 * @param motion False when system animations are off; the theme then changes at once.
 */
internal fun CoroutineScope.applyColorTheme(prefs: ThemePrefs, theme: ColorTheme, motion: Boolean) {
    val current = prefs.theme
    if (current.id == theme.id) return
    val from = ColorThemes.all.indexOf(current)
    val to = ColorThemes.all.indexOf(theme)
    launch { ThemeWipe.run(fromRight = to > from, motion = motion, changes = true) { prefs.choose(theme) } }
}

/** A theme in four dots: its screen, its accent, success and error. The screen's dot is ringed so it shows on any row. */
@Composable
internal fun ThemeDots(dots: List<Color>, modifier: Modifier = Modifier) {
    val ring = menuColors().rowText.copy(alpha = 0.2f)
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
        dots.forEachIndexed { index, color ->
            Box(Modifier.size(14.dp).background(color, CircleShape)
                .then(if (index == 0) Modifier.border(1.dp, ring, CircleShape) else Modifier))
        }
    }
}

/** The field that narrows the list by name. */
@Composable
private fun ThemeSearch(query: String, onChange: (String) -> Unit) {
    val colors = menuColors()
    BasicTextField(
        value = query,
        onValueChange = onChange,
        singleLine = true,
        textStyle = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont),
        cursorBrush = SolidColor(colors.rowText),
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
        modifier = Modifier.fillMaxWidth().semantics { contentDescription = FIND_A_THEME },
        decorationBox = { field ->
            Box(Modifier.heightIn(min = 52.dp).padding(horizontal = 16.dp, vertical = 14.dp), contentAlignment = Alignment.CenterStart) {
                if (query.isEmpty()) Text(FIND_A_THEME, style = TextStyle(color = colors.rowText.copy(alpha = 0.5f), fontSize = 17.sp, fontFamily = CardMenuFont))
                field()
            }
        },
    )
}

/**
 * What the Appearance group shows and does.
 *
 * @property selected The appearance in use: the one picked on the phone, else the account's own.
 * @property onSelect Picks an appearance.
 * @property themeName The colour theme in use.
 * @property onTheme Opens the Theme page.
 */
internal class AppearanceRows(val selected: PageScheme, val onSelect: (PageScheme) -> Unit, val themeName: String, val onTheme: () -> Unit)

/**
 * The Appearance group for [prefs]: picking Light, Dark or System applies to the whole app at once through the wipe,
 * from the top right for Dark and the top left otherwise, as Pully's does. A pick that changes no colour on screen
 * (System while the phone is already in that look) changes the setting without a wipe.
 *
 * @param prefs The phone's theme setting.
 * @param pageScheme The account's own appearance, which the pages follow until one is picked.
 * @param onTheme Opens the Theme page.
 * @return The group's state and actions.
 */
@Composable
internal fun rememberAppearanceRows(prefs: ThemePrefs, pageScheme: PageScheme, onTheme: () -> Unit): AppearanceRows {
    val systemDark = isSystemInDarkTheme()
    val motion = rememberSheetMotionEnabled()
    val scope = rememberCoroutineScope()
    val selected = prefs.appearance ?: pageScheme
    return AppearanceRows(selected, { picked ->
        if (picked != prefs.appearance) {
            val theme = prefs.theme
            val chromeNow = prefs.chromeDark(systemDark)
            val pageNow = selected.isDark(chromeNow)
            val next = picked.isDark(systemDark)
            val changes = theme.palette(next) !== theme.palette(chromeNow) || theme.palette(next) !== theme.palette(pageNow)
            scope.launch { ThemeWipe.run(fromRight = picked == PageScheme.Dark, motion = motion, changes = changes) { prefs.chooseAppearance(picked) } }
        }
    }, prefs.theme.name, onTheme)
}

/** Pully's System, Light and Dark capsule tabs, the one in use raised. */
@Composable
internal fun AppearanceTabs(selected: PageScheme, onSelect: (PageScheme) -> Unit) {
    val colors = menuColors()
    val sheet = LocalCardSheetColors.current
    val haptic = rememberSheetHaptics()
    Row(Modifier.fillMaxWidth().padding(top = 12.dp).cardSurfaceShadow(CircleShape, note = true).clip(CircleShape).background(colors.row).padding(4.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        listOf(PageScheme.System, PageScheme.Light, PageScheme.Dark).forEach { scheme ->
            val on = scheme == selected
            Box(Modifier.weight(1f).heightIn(min = 44.dp).clip(CircleShape).background(if (on) sheet.active else Color.Transparent)
                .semantics { this.selected = on }
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = !on, role = Role.Tab) { haptic.tick(); onSelect(scheme) },
                contentAlignment = Alignment.Center) {
                Text(scheme.name, style = TextStyle(fontFamily = CardMenuFont, fontSize = 16.sp, lineHeight = 22.sp,
                    fontWeight = FontWeight.Medium, color = colors.rowText))
            }
        }
    }
}

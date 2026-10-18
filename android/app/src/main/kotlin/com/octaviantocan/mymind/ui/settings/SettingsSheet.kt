package com.octaviantocan.mymind.ui.settings

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.Image
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val GREETING = "You have a\nbeautiful mind,\n"
private const val NIGHT_ZOOM = 1.15f
private const val DAY_ZOOM = 1.5f
private const val HINT_FADE_PX = 100f
private val SIDE = 32.dp

/**
 * Body of the settings drawer: greeting, account, icon picker, scroll hint, then the rows, the danger zone and the
 * footer further down. The first screenful fills the window; the rest scrolls into view.
 *
 * @param account Email address of the signed-in account, or null while it is unknown.
 * @param version Version text shown in the footer.
 * @param onSignOut Called when the user taps the sign out row.
 * @param modifier Modifier for the body.
 * @param plan Name of the subscription plan, or null to leave the plan lines out.
 * @param greetingName Name appended to the greeting.
 * @param pictures Artwork of the drawer.
 * @param scrollState Scroll position of the body.
 * @param updates Update rows to add after the account rows.
 */
@Composable
fun SettingsSheet(
    account: String?,
    version: String,
    onSignOut: () -> Unit,
    modifier: Modifier = Modifier,
    plan: String? = null,
    greetingName: String = "",
    pictures: SettingsPictures = defaultSettingsPictures(),
    scrollState: ScrollState = rememberScrollState(),
    updates: SettingsUpdates = SettingsUpdates(),
) {
    val dark = MymindTheme.colors.dark
    val zoom = animateFloatAsState(if (dark) NIGHT_ZOOM else DAY_ZOOM)
    val backdrop = rememberGlassBackdrop(zoom)
    Box(modifier.fillMaxSize()) {
        Box(Modifier.fillMaxSize().glassSource(backdrop).background(MymindTheme.colors.settingsSheet))
        Column(Modifier.fillMaxSize().verticalScroll(scrollState)) {
            FirstScreenful(account, plan, greetingName, pictures, scrollState.value)
            Spacer(Modifier.height(20.dp))
            Rows(backdrop, pictures, onSignOut, updates)
            DangerZone(onClick = {}, modifier = Modifier.padding(horizontal = SIDE))
            Spacer(Modifier.height(16.dp))
            SettingsFooter(version, Modifier.padding(horizontal = SIDE))
            Spacer(Modifier.height(16.dp))
        }
    }
}

@Composable
private fun FirstScreenful(account: String?, plan: String?, greetingName: String, pictures: SettingsPictures, scrolled: Int) {
    val insets = WindowInsets.systemBars.asPaddingValues()
    val top = insets.calculateTopPadding()
    val bottom = insets.calculateBottomPadding()
    val windowHeight = with(LocalDensity.current) { LocalWindowInfo.current.containerSize.height.toDp() }
    Column(Modifier.height(windowHeight - top - bottom).padding(top = top, bottom = bottom)) {
        Greeting(greetingName, Modifier.padding(horizontal = SIDE))
        Spacer(Modifier.height(32.dp))
        if (account != null) {
            LabelledValue("YOUR ACCOUNT", account, Modifier.padding(horizontal = SIDE))
            Spacer(Modifier.height(20.dp))
        }
        if (plan != null) {
            LabelledValue("YOUR PLAN", plan, Modifier.padding(horizontal = SIDE))
            Spacer(Modifier.height(20.dp))
        }
        Spacer(Modifier.weight(1f))
        ScrollHint(pictures, scrolled)
    }
}

@Composable
private fun Rows(backdrop: GlassBackdrop, pictures: SettingsPictures, onSignOut: () -> Unit, updates: SettingsUpdates) {
    val side = Modifier.padding(horizontal = SIDE)
    GlassRow("Quick note notification", backdrop, onClick = {}, modifier = side) {
        SettingsSwitch(checked = false, onCheckedChange = {})
    }
    Spacer(Modifier.height(8.dp))
    GlassRow("Get help or send feedback", backdrop, onClick = {}, modifier = side) { RowIcon(pictures.help) }
    Spacer(Modifier.height(8.dp))
    GlassRow("Sign out of mymind", backdrop, onClick = onSignOut, modifier = side) { RowIcon(pictures.signOut) }
    Spacer(Modifier.height(8.dp))
    if (updates.visible) {
        GlassRow(updates.versionLabel, backdrop, onClick = {}, modifier = side)
        Spacer(Modifier.height(8.dp))
        GlassRow("Check for updates", backdrop, onClick = updates.onCheck, modifier = side) { RowValue(updates.status) }
        Spacer(Modifier.height(8.dp))
        if (updates.showAllow) {
            GlassRow("Allow mymind to install updates", backdrop, onClick = updates.onAllow, modifier = side)
            Spacer(Modifier.height(8.dp))
        }
        GlassRow("Changelog", backdrop, onClick = updates.onChangelog, modifier = side)
        Spacer(Modifier.height(8.dp))
    }
}

@Composable
private fun Greeting(name: String, modifier: Modifier) {
    val colors = MymindTheme.colors
    val text = buildAnnotatedString {
        withStyle(SpanStyle(color = colors.settingsGreeting)) { append(GREETING) }
        // An empty last line lays out 1 px taller than one with text, so a missing name keeps a zero-width glyph.
        withStyle(SpanStyle(color = Color.White)) { append(name.ifEmpty { "\u200B" }) }
    }
    BasicText(
        text,
        modifier,
        style = TextStyle(fontSize = 48.sp, fontFamily = MymindTheme.fonts.louize, letterSpacing = (-0.05).em, lineHeight = 0.95.em),
    )
}

@Composable
private fun LabelledValue(label: String, value: String, modifier: Modifier) {
    val colors = MymindTheme.colors
    val nunito = MymindTheme.fonts.nunito
    BasicText(label, modifier, style = TextStyle(color = colors.settingsLabel, fontSize = 13.sp, fontFamily = nunito, letterSpacing = 0.06.em))
    BasicText(
        value,
        modifier,
        style = TextStyle(color = colors.settingsValue, fontSize = 20.sp, fontFamily = nunito, letterSpacing = (-0.01).em, lineHeight = 1.4.em),
    )
}

@Composable
private fun ScrollHint(pictures: SettingsPictures, scrolled: Int) {
    val target = if (scrolled <= 0) 1f else (1f - scrolled / HINT_FADE_PX).coerceIn(0f, 1f)
    val alpha by animateFloatAsState(target)
    Row(
        Modifier.alpha(alpha).fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(pictures.hand, contentDescription = null)
        BasicText(
            "Swipe down for more options",
            style = TextStyle(color = MymindTheme.colors.settingsHint, fontSize = 13.sp, fontFamily = MymindTheme.fonts.nunito, lineHeight = 18.sp),
        )
    }
}

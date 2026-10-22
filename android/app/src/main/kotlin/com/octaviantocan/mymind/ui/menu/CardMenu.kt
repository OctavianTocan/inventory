package com.octaviantocan.mymind.ui.menu

import androidx.annotation.DrawableRes
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.GeistFontFamily
import com.octaviantocan.mymind.ui.card.*
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Actions supported by the card menu; ordering and availability come from cardMenuItems. */
enum class CardMenuAction(val label: String, @param:DrawableRes val icon: Int, val danger: Boolean = false) {
    SameVibe("Same Vibe", R.drawable.samevibeicon),
    ReaderMode("Reader Mode", R.drawable.readermodeicon),
    Download("Download", R.drawable.downloadicon),
    ViewSource("View original source", R.drawable.visitsourceicon),
    Copy("Copy to clipboard", R.drawable.copytoclipboardicon),
    AddTags("Add tags", R.drawable.addtagsicon),
    AddToSpace("Add to space", R.drawable.addtospaceicon),
    RemoveFromSpace("Remove from space", R.drawable.removefromspaceicon),
    AddToTopOfMind("Top of Mind", R.drawable.topofmindicon),
    RemoveFromTopOfMind("Top of Mind", R.drawable.removetopofmindicon),
    DeleteCard("Delete card", R.drawable.deleteicon, danger = true),
}

/** Surface and action colors shared by the card menu and its subpages. */
@Immutable
class MenuColors(
    val scrim: Color,
    val sheet: Color,
    val row: Color,
    val rowText: Color,
    val dangerRow: Color,
    val dangerText: Color,
    val handle: Color,
)

private val Scrim = Color(0xFF3B4050).copy(alpha = 0.8f)
private val Danger = Color(0xFFFF002E)
private val Handle = Color(0xFF49454F)

/** Legacy update-card colors; contextual card sheets use Pully's palette below. */
val DarkMenuColors = MenuColors(Scrim, Color(0xFF14161E), Color(0xFF1C1E26), Color(0xFF748297), Danger.copy(alpha = 0.1f), Danger, Handle)
val LightMenuColors = MenuColors(Scrim, Color.White, Color(0xFFF4F6F9), Color(0xFF353643), Danger.copy(alpha = 0.1f), Danger, Handle)

internal val LocalCardMenuAccent = staticCompositionLocalOf<Color?> { null }
internal val CardMenuFont = GeistFontFamily

/** Pully's paper and floating controls, tinted with the same accent as the visible card. */
internal fun cardMenuSheetColors(accent: Color?, dark: Boolean): CardSheetColors {
    val base = if (dark) DarkCardSheetColors else LightCardSheetColors
    if (accent == null) return base
    return CardSheetColors(
        paper = lerp(base.paper, accent, 0.24f),
        raised = lerp(base.raised, accent, 0.12f),
        text = base.text,
        grabber = lerp(base.grabber, accent, 0.5f),
        scrim = base.scrim,
        active = base.active,
        shadow = base.shadow,
    )
}

@Composable
internal fun menuColors(): MenuColors {
    val dark = MymindTheme.colors.dark
    val sheet = cardMenuSheetColors(LocalCardMenuAccent.current, dark)
    return MenuColors(sheet.scrim, sheet.paper, sheet.raised, sheet.text,
        sheet.raised, if (dark) Color(0xFFFF98AA) else Color(0xFFB4233A), sheet.grabber)
}

/** Shipped artwork for each card action. */
@Immutable
class CardMenuIcons(val icon: @Composable (CardMenuAction) -> Painter)

@Composable
fun defaultCardMenuIcons() = CardMenuIcons(icon = { painterResource(it.icon) })

/** Pully's grouped action rows; the sheet owns the grabber, header and scrolling viewport. */
@Composable
fun CardMenuContent(
    items: List<CardMenuItem>,
    onSelect: (CardMenuItem) -> Unit,
    modifier: Modifier = Modifier,
    icons: CardMenuIcons = defaultCardMenuIcons(),
    enabled: Boolean = true,
) {
    val colors = menuColors()
    Column(modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 16.dp, bottom = 30.dp)) {
        Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(colors.row)) {
            items.forEachIndexed { index, item ->
                if (index > 0) HorizontalDivider(Modifier.padding(start = 52.dp), color = colors.rowText.copy(alpha = 0.1f), thickness = 1.dp)
                CardMenuRow(item, colors, icons, enabled) { onSelect(item) }
            }
        }
    }
}

/** Floating wrapper retained for update notices, separate from contextual card sheets. */
@Composable
fun MenuCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val colors = if (MymindTheme.colors.dark) DarkMenuColors else LightMenuColors
    Column(
        modifier.fillMaxWidth()
            .windowInsetsPadding(WindowInsets.ime.union(WindowInsets.navigationBars))
            .padding(16.dp)
            .background(colors.sheet, RoundedCornerShape(22.dp))
            .animateContentSize(),
        content = content,
    )
}

/** A Pully row: one shared surface, an inset divider and a label that can wrap. */
@Composable
private fun CardMenuRow(item: CardMenuItem, colors: MenuColors, icons: CardMenuIcons, enabled: Boolean, onClick: () -> Unit) {
    val tint = if (item.action.danger) colors.dangerText else colors.rowText
    val haptic = rememberSheetHaptics()
    Row(
        Modifier.fillMaxWidth().heightIn(min = 52.dp)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = enabled, role = Role.Button) { haptic.tick(); onClick() }
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(icons.icon(item.action), contentDescription = null, modifier = Modifier.size(22.dp), colorFilter = ColorFilter.tint(tint))
        Text(item.action.label, Modifier.weight(1f), style = TextStyle(color = tint, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont))
    }
}

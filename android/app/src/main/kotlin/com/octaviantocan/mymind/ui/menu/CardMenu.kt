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
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.AppFontFamily
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
internal val CardMenuFont = AppFontFamily

/** Neutral Pully paper and controls; the card accent is confined to the top hint and grabber. */
internal fun cardMenuSheetColors(accent: Color?, dark: Boolean): CardSheetColors {
    val base = if (dark) DarkCardSheetColors else LightCardSheetColors
    if (accent == null) return base
    return CardSheetColors(
        paper = base.paper,
        raised = base.raised,
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

/** Pully's quick tiles and titled groups; the sheet owns its fixed header and viewport. */
@Composable
fun CardMenuContent(
    items: List<CardMenuItem>,
    onSelect: (CardMenuItem) -> Unit,
    modifier: Modifier = Modifier,
    icons: CardMenuIcons = defaultCardMenuIcons(),
    enabled: Boolean = true,
) {
    val colors = menuColors()
    val quick = items.filter { it.action in QUICK_ACTIONS }
    val organize = items.filter { it.action in ORGANIZE_ACTIONS }
    val discover = items.filter { it.action !in QUICK_ACTIONS && it.action !in ORGANIZE_ACTIONS && !it.action.danger }
    val manage = items.filter { it.action.danger }
    Column(modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 8.dp, bottom = 30.dp)) {
        if (quick.isNotEmpty()) {
            CardMenuHeading("Quick actions", first = true)
            quick.chunked(if (quick.size > 3) 2 else 3).forEach { row ->
                Row(Modifier.fillMaxWidth().padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    row.forEach { item -> CardQuickAction(item, colors, icons, enabled, Modifier.weight(1f)) { onSelect(item) } }
                }
            }
        }
        listOf("Organize" to organize, "Discover" to discover, "Manage card" to manage).forEach { (heading, rows) ->
            if (rows.isNotEmpty()) {
                CardMenuHeading(heading, first = quick.isEmpty() && heading == "Organize")
                CardMenuGroup(Modifier.padding(top = 12.dp)) {
                    rows.forEachIndexed { index, item ->
                        if (index > 0) SheetRowDivider()
                        CardMenuRow(item, colors, icons, enabled) { onSelect(item) }
                    }
                }
            }
        }
    }
}

private val QUICK_ACTIONS = setOf(CardMenuAction.ViewSource, CardMenuAction.Copy, CardMenuAction.Download,
    CardMenuAction.AddToTopOfMind, CardMenuAction.RemoveFromTopOfMind)
private val ORGANIZE_ACTIONS = setOf(CardMenuAction.AddTags, CardMenuAction.AddToSpace, CardMenuAction.RemoveFromSpace)

/** The kit's small, muted, non-folding section heading, outside the raised group. */
@Composable
internal fun CardMenuHeading(label: String, first: Boolean = false) {
    Text(label, Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = if (first) 12.dp else 24.dp, bottom = 2.dp)
        .semantics { heading() },
        style = TextStyle(fontFamily = CardMenuFont, fontSize = 14.sp, lineHeight = 20.sp,
            fontWeight = FontWeight.Medium, color = menuColors().rowText.copy(alpha = 0.65f)))
}

/** The same 16dp raised card and offset note shadow used by Pully's sheets. */
@Composable
internal fun CardMenuGroup(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val shape = RoundedCornerShape(16.dp)
    Column(modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(menuColors().row), content = content)
}

@Composable
private fun CardQuickAction(item: CardMenuItem, colors: MenuColors, icons: CardMenuIcons, enabled: Boolean, modifier: Modifier, onClick: () -> Unit) {
    val label = when (item.action) {
        CardMenuAction.ViewSource -> "Source"
        CardMenuAction.Copy -> "Copy"
        CardMenuAction.RemoveFromTopOfMind -> "Unpin"
        else -> item.action.label
    }
    SheetTile(icons.icon(item.action), label, onClick, modifier, description = item.action.label, enabled = enabled)
}

/**
 * One of Pully's quick-action tiles: a raised 16dp card with the icon over a short label.
 *
 * @param icon Artwork, tinted with the row ink.
 * @param label Short label under the icon.
 * @param onClick Called on a tap, after the sheet's tick.
 * @param modifier Modifier for the tile; callers in a row give each tile the same weight.
 * @param description Spoken name, when the label alone is too short.
 * @param enabled False ignores taps.
 */
@Composable
internal fun SheetTile(icon: Painter, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, description: String = label, enabled: Boolean = true) {
    val colors = menuColors()
    val shape = RoundedCornerShape(16.dp)
    val haptic = rememberSheetHaptics()
    Column(modifier.heightIn(min = 72.dp).cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row)
        .semantics { contentDescription = description }
        .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = enabled, role = Role.Button) { haptic.tick(); onClick() }
        .padding(horizontal = 8.dp, vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterVertically)) {
        Image(icon, null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(colors.rowText))
        Text(label, style = TextStyle(fontFamily = CardMenuFont, fontSize = 14.sp, lineHeight = 20.sp, color = colors.rowText),
            textAlign = androidx.compose.ui.text.style.TextAlign.Center)
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
    val detail = when (item.action) {
        CardMenuAction.AddTags -> "Label and find this card"
        CardMenuAction.AddToSpace -> "Keep related cards together"
        else -> if (item.locked) "Not included in your plan" else null
    }
    val opensPage = item.action == CardMenuAction.AddTags || item.action == CardMenuAction.AddToSpace
    SheetRow(icons.icon(item.action), item.action.label, onClick, detail = detail, danger = item.action.danger, enabled = enabled,
        trailing = if (opensPage) { { SheetRowChevron(colors) } } else null)
}

/**
 * One row of a raised Pully group: the icon, the label with an optional muted detail, then optional trailing content.
 *
 * @param icon Artwork, tinted like the label, or null for a row of text alone.
 * @param label The row's name.
 * @param onClick Called on a tap, after the sheet's tick; null for a row that only shows something.
 * @param detail Muted second line, or null.
 * @param danger True paints the icon and label in the destructive color.
 * @param enabled False ignores taps.
 * @param iconTint The icon's own colour, such as a brand mark's, instead of the label's.
 * @param role What a tap does, for accessibility: a button, or a radio button on a list of choices.
 * @param trailing Drawn at the end, such as a value or a chevron.
 */
@Composable
internal fun SheetRow(
    icon: Painter?,
    label: String,
    onClick: (() -> Unit)?,
    detail: String? = null,
    danger: Boolean = false,
    enabled: Boolean = true,
    iconTint: Color? = null,
    role: Role = Role.Button,
    trailing: (@Composable RowScope.() -> Unit)? = null,
) {
    val colors = menuColors()
    val tint = if (danger) colors.dangerText else colors.rowText
    val haptic = rememberSheetHaptics()
    Row(
        Modifier.fillMaxWidth().heightIn(min = 52.dp)
            .then(if (onClick == null) Modifier else Modifier.clickable(remember { MutableInteractionSource() }, CardSheetIndication,
                enabled = enabled, role = role) { haptic.tick(); onClick() })
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null) Image(icon, contentDescription = null, modifier = Modifier.size(22.dp), colorFilter = ColorFilter.tint(iconTint ?: tint))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(label, style = TextStyle(color = tint, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont))
            if (detail != null) Text(detail, style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 14.sp, lineHeight = 20.sp, fontFamily = CardMenuFont))
        }
        trailing?.invoke(this)
    }
}

/** The muted chevron of a row that opens a page. */
@Composable
internal fun SheetRowChevron(colors: MenuColors = menuColors()) {
    Image(painterResource(R.drawable.ic_caret_right), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.rowText.copy(alpha = 0.55f)))
}

/**
 * The inset divider between rows of one raised group.
 *
 * @param start Where it starts: past the icon (52dp), or at the text of rows without icons (16dp).
 */
@Composable
internal fun SheetRowDivider(start: Dp = 52.dp) {
    HorizontalDivider(Modifier.padding(start = start), color = menuColors().rowText.copy(alpha = 0.1f), thickness = 1.dp)
}

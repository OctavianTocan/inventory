package com.octaviantocan.mymind.ui.menu

import androidx.annotation.DrawableRes
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.text.style.TextOverflow
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
    Pin("Pin", R.drawable.topofmindicon),
    Unpin("Pin", R.drawable.removetopofmindicon),
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

/**
 * Choose one: the card's big choices (Add to space, Same Vibe, Reader Mode), then its quick actions as tiles two by
 * two, then Delete card alone in its own group, held rather than tapped. The sheet owns its fixed header and viewport.
 *
 * @param items The card's actions, from [cardMenuItems].
 * @param onSelect Called with the chosen action; Delete card only once its hold completes.
 * @param modifier Modifier for the content.
 * @param icons Artwork for each action.
 * @param enabled False ignores taps.
 * @param spaces Add to space in place, with the card's spaces, instead of a choice that opens the spaces page; null
 *   keeps the choice.
 * @param onDeleteHold How far Delete card's hold has come, from 0 to 1.
 */
@Composable
fun CardMenuContent(
    items: List<CardMenuItem>,
    onSelect: (CardMenuItem) -> Unit,
    modifier: Modifier = Modifier,
    icons: CardMenuIcons = defaultCardMenuIcons(),
    enabled: Boolean = true,
    spaces: (@Composable () -> Unit)? = null,
    onDeleteHold: (Float) -> Unit = {},
) {
    val colors = menuColors()
    val choices = items.filter { it.action in CHOICES }.sortedBy { CHOICES.indexOf(it.action) }
    val tiles = items.filter { it.action in TILES }.sortedBy { TILES.indexOf(it.action) }
    val delete = items.firstOrNull { it.action == CardMenuAction.DeleteCard }
    Column(modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 8.dp, bottom = 30.dp)) {
        if (choices.isNotEmpty()) Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            choices.forEach { item ->
                if (spaces != null && item.action in SPACE_ACTIONS) spaces()
                else CardChoice(item, icons, enabled) { onSelect(item) }
            }
        }
        if (tiles.isNotEmpty()) Column(Modifier.padding(top = if (choices.isEmpty()) 0.dp else 20.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            tiles.chunked(2).forEach { row ->
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    row.forEach { item -> CardQuickAction(item, colors, icons, enabled, Modifier.weight(1f)) { onSelect(item) } }
                    if (row.size == 1) Spacer(Modifier.weight(1f))
                }
            }
        }
        if (delete != null) CardMenuGroup(Modifier.padding(top = 20.dp)) {
            HoldToDeleteRow(onDelete = { if (enabled) onSelect(delete) }, onProgress = onDeleteHold)
        }
    }
}

private val SPACE_ACTIONS = setOf(CardMenuAction.AddToSpace, CardMenuAction.RemoveFromSpace)
private val CHOICES = listOf(CardMenuAction.AddToSpace, CardMenuAction.RemoveFromSpace, CardMenuAction.SameVibe, CardMenuAction.ReaderMode)
private val TILES = listOf(CardMenuAction.ViewSource, CardMenuAction.Copy, CardMenuAction.Download, CardMenuAction.Pin,
    CardMenuAction.Unpin, CardMenuAction.AddTags)

/** The kit's small, muted, non-folding section heading, outside the raised group. */
@Composable
internal fun CardMenuHeading(label: String, first: Boolean = false) {
    Text(label, Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = if (first) 12.dp else 24.dp, bottom = 2.dp)
        .semantics { heading() },
        style = TextStyle(fontFamily = CardMenuFont, fontSize = 15.sp, lineHeight = 21.sp,
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
    val pinned = item.action == CardMenuAction.Unpin
    val label = when (item.action) {
        CardMenuAction.ViewSource -> "Open source"
        CardMenuAction.Copy -> "Copy"
        else -> item.action.label
    }
    SheetTile(icons.icon(item.action), label, onClick, modifier, description = if (pinned) "Pinned" else item.action.label,
        enabled = enabled, on = pinned)
}

/** One of the big choices: what it does over a muted line on what that means. */
@Composable
private fun CardChoice(item: CardMenuItem, icons: CardMenuIcons, enabled: Boolean, onClick: () -> Unit) {
    val detail = when {
        item.locked -> "Not included in your plan"
        else -> when (item.action) {
            CardMenuAction.AddToSpace -> "Keep it with cards like it"
            CardMenuAction.RemoveFromSpace -> "Take it out of this space"
            CardMenuAction.SameVibe -> "Cards that feel like this one"
            CardMenuAction.ReaderMode -> "Just the words"
            else -> null
        }
    }
    SheetChoice(icons.icon(item.action), item.action.label, detail, onClick, enabled = enabled)
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
 * @param on True fills the tile with ink, for a setting that is on, such as a pinned card's Pin.
 */
@Composable
internal fun SheetTile(icon: Painter, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, description: String = label, enabled: Boolean = true,
    on: Boolean = false) {
    val colors = menuColors()
    val shape = RoundedCornerShape(16.dp)
    val haptic = rememberSheetHaptics()
    val fill by animateColorAsState(if (on) colors.rowText else colors.row, tween(200), label = "tileFill")
    val ink by animateColorAsState(if (on) colors.sheet else colors.rowText, tween(200), label = "tileInk")
    Column(modifier.heightIn(min = 72.dp).cardSurfaceShadow(shape, note = true).clip(shape).background(fill)
        .semantics { contentDescription = description }
        .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = enabled, role = Role.Button) { haptic.tick(); onClick() }
        .padding(horizontal = 8.dp, vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterVertically)) {
        Image(icon, null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(ink))
        Text(label, style = TextStyle(fontFamily = CardMenuFont, fontSize = 15.sp, lineHeight = 21.sp, color = ink),
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
            Text(label, style = TextStyle(color = tint, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont))
            if (detail != null) Text(detail, style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont))
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

/**
 * Pully's switch, drawn at the end of a row whose whole surface toggles it: a 52 by 32dp track with a white thumb.
 *
 * @param checked True slides the thumb to the end over an ink track.
 */
@Composable
internal fun SheetSwitch(checked: Boolean) {
    val colors = menuColors()
    val dark = MymindTheme.colors.dark
    val track by animateColorAsState(if (checked) colors.rowText else if (dark) Color(0xFF3A3A3A) else Color(0xFFE3E5E3), tween(200), label = "switchTrack")
    val thumb by animateDpAsState(if (checked) 23.dp else 3.dp, tween(200), label = "switchThumb")
    Box(Modifier.size(52.dp, 32.dp).clip(CircleShape).background(track)) {
        Box(Modifier.offset(x = thumb, y = 3.dp).size(26.dp).cardSurfaceShadow(CircleShape).clip(CircleShape)
            .background(if (checked && dark) colors.sheet else Color.White))
    }
}

/**
 * One of a sheet's large choices: a raised 16dp card with the icon in a round wash, the label over a muted detail,
 * and an optional note at the end.
 *
 * @param icon Artwork, tinted with the row ink.
 * @param label What choosing it does.
 * @param detail Muted second line, or null.
 * @param onClick Called on a tap, after the sheet's tick.
 * @param modifier Modifier for the card.
 * @param note Short muted word at the end, such as "Most used", or null.
 * @param enabled False ignores taps.
 */
@Composable
internal fun SheetChoice(icon: Painter, label: String, detail: String?, onClick: () -> Unit, modifier: Modifier = Modifier, note: String? = null, enabled: Boolean = true) {
    val colors = menuColors()
    val shape = RoundedCornerShape(16.dp)
    val haptic = rememberSheetHaptics()
    Row(
        modifier.fillMaxWidth().heightIn(min = 76.dp).cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = enabled, role = Role.Button) { haptic.tick(); onClick() }
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(44.dp).clip(CircleShape).background(colors.rowText.copy(alpha = 0.1f)), contentAlignment = Alignment.Center) {
            Image(icon, null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(colors.rowText))
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(label, style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont))
            if (detail != null) Text(detail, style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont),
                maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
        if (note != null) Text(note, style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont), maxLines = 1)
    }
}

/**
 * Pully's quiet 44dp pill for a sheet's lesser action, raised on the sheet's paper.
 *
 * @param icon Artwork before the label, or null.
 * @param label What it does.
 * @param onClick Called on a tap, after the sheet's tick.
 * @param modifier Modifier for the pill.
 */
@Composable
internal fun SheetPill(icon: Painter?, label: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    Row(
        modifier.height(44.dp).cardSurfaceShadow(CircleShape).clip(CircleShape).background(colors.row)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { haptic.tick(); onClick() }
            .padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null) Image(icon, null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.rowText))
        Text(label, style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium, fontFamily = CardMenuFont), maxLines = 1)
    }
}

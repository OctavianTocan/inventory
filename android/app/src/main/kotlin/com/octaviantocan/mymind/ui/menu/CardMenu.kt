package com.octaviantocan.mymind.ui.menu

import androidx.annotation.DrawableRes
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * The actions a card menu row can carry, with the label and icon the original app shows.
 *
 * @property label Row text.
 * @property icon Drawable drawn at its own size and colours before the label.
 * @property danger True for rows drawn in the red danger colours.
 */
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

/**
 * Colours of the card menu in one system appearance.
 *
 * @property scrim Colour laid over the screen behind the open menu.
 * @property sheet Fill of the menu card.
 * @property row Fill of a normal row.
 * @property rowText Label colour of a normal row.
 * @property dangerRow Fill of the delete row.
 * @property dangerText Label colour of the delete row.
 * @property handle Colour of the drag handle.
 */
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

/** Card menu colours when the system is dark. */
val DarkMenuColors = MenuColors(Scrim, Color(0xFF14161E), Color(0xFF1C1E26), Color(0xFF748297), Danger.copy(alpha = 0.1f), Danger, Handle)

/** Card menu colours when the system is light. */
val LightMenuColors = MenuColors(Scrim, Color.White, Color(0xFFF4F6F9), Color(0xFF353643), Danger.copy(alpha = 0.1f), Danger, Handle)

/**
 * Pictures of the menu rows.
 *
 * @property icon Picture of an action's row.
 * @property lock Lock drawn at the end of locked rows.
 */
@Immutable
class CardMenuIcons(val icon: @Composable (CardMenuAction) -> Painter, val lock: Painter)

/** @return The pictures that ship with the app resources; the lock follows the system appearance. */
@Composable
fun defaultCardMenuIcons() = CardMenuIcons(
    icon = { painterResource(it.icon) },
    lock = painterResource(if (MymindTheme.colors.dark) R.drawable.upgrade_dark else R.drawable.upgrade_light),
)

/**
 * The card menu sheet: a Material 3 modal bottom sheet without its own scrim or surface, holding [CardMenuContent],
 * or a submenu in the same card. The caller draws the scrim and blur behind it from [sheetState].
 *
 * @param items Rows to show.
 * @param onSelect Called with the tapped row; the caller closes the sheet or switches to a submenu.
 * @param onDismiss Called when the sheet was swiped away or Back was pressed.
 * @param sheetState State of the sheet.
 * @param icons Pictures of the rows.
 * @param submenu Page shown instead of the rows, or null for the rows.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CardMenuSheet(
    items: List<CardMenuItem>,
    onSelect: (CardMenuItem) -> Unit,
    onDismiss: () -> Unit,
    sheetState: SheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
    icons: CardMenuIcons = defaultCardMenuIcons(),
    submenu: (@Composable () -> Unit)? = null,
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        modifier = Modifier.fillMaxWidth(),
        sheetState = sheetState,
        containerColor = Color.Transparent,
        contentColor = Color.Unspecified,
        tonalElevation = 0.dp,
        scrimColor = Color.Transparent,
        dragHandle = null,
        contentWindowInsets = { WindowInsets(0, 0, 0, 0) },
    ) {
        if (submenu == null) CardMenuContent(items, onSelect, icons = icons) else MenuCard { submenu() }
    }
}

/**
 * The visible menu: a rounded card 16 dp from the screen edges and the navigation bar, with a drag handle and one
 * 52 dp row per action.
 *
 * @param items Rows to show.
 * @param onSelect Called with the tapped row.
 * @param modifier Modifier for the outer column.
 * @param icons Pictures of the rows.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CardMenuContent(
    items: List<CardMenuItem>,
    onSelect: (CardMenuItem) -> Unit,
    modifier: Modifier = Modifier,
    icons: CardMenuIcons = defaultCardMenuIcons(),
) {
    val colors = if (MymindTheme.colors.dark) DarkMenuColors else LightMenuColors
    MenuCard(modifier) {
        Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            BottomSheetDefaults.DragHandle(color = colors.handle)
        }
        LazyColumn(
            contentPadding = PaddingValues(12.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            items(items, key = { it.action.name }) { item -> CardMenuRow(item, colors, icons) { onSelect(item) } }
        }
    }
}

/**
 * The rounded card that holds the menu or a submenu, 16 dp from the screen edges and above the navigation bar or the
 * keyboard.
 *
 * @param modifier Modifier for the outer column.
 * @param content The menu rows or a submenu page.
 */
@Composable
fun MenuCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val colors = if (MymindTheme.colors.dark) DarkMenuColors else LightMenuColors
    Column(
        modifier
            .fillMaxWidth()
            .windowInsetsPadding(WindowInsets.ime.union(WindowInsets.navigationBars))
            .padding(16.dp)
            .background(colors.sheet, RoundedCornerShape(22.dp))
            .animateContentSize(),
        content = content,
    )
}

/** One 52 dp row: icon, label and, when locked, the lock. */
@Composable
private fun CardMenuRow(item: CardMenuItem, colors: MenuColors, icons: CardMenuIcons, onClick: () -> Unit) {
    val danger = item.action.danger
    val shape = RoundedCornerShape(16.dp)
    Row(
        Modifier
            .fillMaxWidth()
            .height(52.dp)
            .background(if (danger) colors.dangerRow else colors.row, shape)
            .clip(shape)
            .clickable(onClick = onClick)
            .padding(horizontal = 24.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(icons.icon(item.action), contentDescription = null)
        Text(
            item.action.label,
            Modifier.weight(1f),
            style = TextStyle(color = if (danger) colors.dangerText else colors.rowText, fontSize = 17.sp, fontFamily = MymindTheme.fonts.nunito),
        )
        if (item.locked) Image(icons.lock, contentDescription = "")
    }
}

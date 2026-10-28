package com.octaviantocan.mymind.ui.card

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.paneTitle
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.util.lerp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.library.CardThumbnail
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.pinned
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.CardMenuHeading
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowChevron
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.SheetSwitch
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * The name's card: tapping the reader's name grows its pill into a card of the card's details over a light scrim, with
 * Copy link, Top of Mind and its spaces, then the cards opened before it. Tapping its name, the scrim or Back hides it.
 *
 * @param visible True while it shows; it grows in and shrinks back into the pill.
 * @param card The card the reader shows.
 * @param details Its spaces' names and the recently opened cards.
 * @param actions What its rows do; a row whose action is null is left out.
 * @param onClose Hides it.
 */
@Composable
internal fun ReaderNameCard(visible: Boolean, card: Card, details: ReaderDetails, actions: ReaderActions, onClose: () -> Unit) {
    val motion = rememberSheetMotionEnabled()
    val shown by animateFloatAsState(if (visible) 1f else 0f,
        tween(if (!motion) 0 else if (visible) GROW_MS else SHRINK_MS, easing = CardSheetMotion.SmoothOut), label = "nameCard")
    if (!visible && shown == 0f) return
    BackHandler(enabled = visible, onBack = onClose)
    val colors = LocalCardSheetColors.current
    val then = { action: () -> Unit -> { onClose(); action() } }
    Box(Modifier.fillMaxSize()) {
        Box(Modifier.fillMaxSize().graphicsLayer { alpha = shown }.background(colors.text.copy(alpha = 0.2f))
            .clickable(remember { MutableInteractionSource() }, indication = null, enabled = visible, role = Role.Button,
                onClickLabel = "Hide card details", onClick = onClose))
        val shape = RoundedCornerShape(24.dp)
        Column(
            Modifier.windowInsetsPadding(WindowInsets.safeDrawing).padding(start = 16.dp, end = 16.dp, top = 20.dp, bottom = 16.dp)
                .graphicsLayer {
                    transformOrigin = TransformOrigin(0.5f, 0f)
                    scaleX = lerp(0.66f, 1f, shown)
                    scaleY = lerp(0.08f, 1f, shown)
                    alpha = shown
                }
                .fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(colors.paper)
                .semantics { paneTitle = "About this card" }
                .verticalScroll(rememberScrollState())
                .padding(start = 12.dp, end = 12.dp, bottom = 16.dp),
        ) {
            Row(Modifier.fillMaxWidth().height(44.dp).clickable(remember { MutableInteractionSource() }, CardSheetIndication,
                    role = Role.Button, onClickLabel = "Hide card details", onClick = onClose),
                horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically) {
                Text(cardHeading(card), Modifier.weight(1f, fill = false).padding(start = 16.dp), style = readerTitleStyle(colors),
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
                Image(painterResource(R.drawable.ic_caret_down), null, Modifier.padding(end = 16.dp).size(16.dp).rotate(180f),
                    colorFilter = ColorFilter.tint(colors.text.copy(alpha = 0.65f)))
            }
            Byline(card)
            val link = sourceUrl(card)
            val rows = buildList<@Composable () -> Unit> {
                if (link != null) actions.onCopyLink?.let { copy ->
                    add { SheetRow(painterResource(R.drawable.ic_link), "Copy link", then(copy), detail = siteOf(card)) }
                }
                actions.onTogglePin?.let { toggle ->
                    add {
                        SheetRow(painterResource(R.drawable.topofmindicon), "Top of Mind", toggle,
                            detail = if (card.pinned) "Pinned" else "Keep it above everything else", role = Role.Switch,
                            trailing = { SheetSwitch(card.pinned) })
                    }
                }
                actions.onSpaces?.let { spaces ->
                    add {
                        SheetRow(painterResource(R.drawable.addtospaceicon), "Spaces", then(spaces),
                            detail = details.spaces.joinToString(", ").ifEmpty { "Not in a space yet" }, trailing = { SheetRowChevron() })
                    }
                }
            }
            if (rows.isNotEmpty()) {
                CardMenuHeading("This card")
                CardMenuGroup(Modifier.padding(top = 12.dp)) {
                    rows.forEachIndexed { index, row -> if (index > 0) SheetRowDivider(); row() }
                }
            }
            val open = actions.onOpenRecent
            val recent = if (open != null) details.recent.filter { it.id != card.id }.take(RECENT_LIMIT) else emptyList()
            if (recent.isNotEmpty()) {
                CardMenuHeading("Opened recently")
                CardMenuGroup(Modifier.padding(top = 12.dp)) {
                    recent.forEachIndexed { index, other ->
                        if (index > 0) SheetRowDivider(start = 74.dp)
                        RecentRow(other) { onClose(); open?.invoke(other) }
                    }
                    actions.onEverything?.let { everything ->
                        SheetRowDivider()
                        SheetRow(painterResource(R.drawable.ic_sheet_back), "Back to Everything", then(everything))
                    }
                }
            }
        }
    }
}

/** The card's picture beside who made it, where it is from, when it was saved and how long it takes to read. */
@Composable
private fun Byline(card: Card) {
    val colors = LocalCardSheetColors.current
    val first = bylineOf(card)
    val second = listOfNotNull(savedLabel(card.created), readingTime(card)).joinToString(" · ").ifEmpty { null }
    if (first == null && second == null) return
    Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 4.dp, top = 8.dp), horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically) {
        Thumb(card, 56)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            first?.let { Text(it, style = TextStyle(color = colors.text, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = MymindTheme.fonts.body)) }
            second?.let { Text(it, style = TextStyle(color = colors.text.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = MymindTheme.fonts.body)) }
        }
    }
}

/** A recently opened card: its picture, name and where it is from, or when it was saved. */
@Composable
private fun RecentRow(card: Card, onClick: () -> Unit) {
    val colors = LocalCardSheetColors.current
    val haptic = rememberSheetHaptics()
    Row(Modifier.fillMaxWidth().heightIn(min = 64.dp)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClickLabel = "Open") { haptic.tick(); onClick() }
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
        Thumb(card, 44)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(cardHeading(card), style = TextStyle(color = colors.text, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = MymindTheme.fonts.body),
                maxLines = 1, overflow = TextOverflow.Ellipsis)
            (bylineOf(card) ?: savedLabel(card.created))?.let {
                Text(it, style = TextStyle(color = colors.text.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = MymindTheme.fonts.body),
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

/** A rounded square of the card's picture, or of its words' icon when it has none. */
@Composable
private fun Thumb(card: Card, side: Int) {
    val colors = LocalCardSheetColors.current
    val shape = RoundedCornerShape((side / 4).dp)
    if (galleryPicture(card) != null) CardThumbnail(card, Modifier.size(side.dp).clip(shape))
    else Box(Modifier.size(side.dp).clip(shape).background(colors.active), contentAlignment = Alignment.Center) {
        Image(painterResource(R.drawable.ic_text), null, Modifier.size((side / 2).dp), colorFilter = ColorFilter.tint(colors.text.copy(alpha = 0.65f)))
    }
}

private const val GROW_MS = 300
private const val SHRINK_MS = 200
private const val RECENT_LIMIT = 3

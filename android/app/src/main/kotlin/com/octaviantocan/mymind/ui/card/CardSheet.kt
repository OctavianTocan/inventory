package com.octaviantocan.mymind.ui.card

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.library.CardPicture
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.noteText
import com.octaviantocan.mymind.ui.library.tileKind
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Gap between the status bar and the top of an opened card. */
val CARD_SHEET_TOP_GAP = 24.dp

private val SHEET_RADIUS = 10.dp
private val BODY_SIDE = 30.dp

/**
 * An opened card: a sheet over the library and the tab bar, starting 24 dp under the status bar, with the card
 * header over the card's content. The header's backdrop takes the native chrome colour; at rest its icons contrast
 * with the card page instead.
 *
 * @param card Card to show.
 * @param onClose Called when the caret is tapped.
 * @param onMenu Called when the menu button is tapped.
 * @param modifier Modifier for the full-screen box holding the sheet.
 * @param headerState Scroll tracking that decides whether the header shows.
 * @param scrollState Scroll position of the card's content.
 * @param headerIcons Pictures of the header buttons.
 * @param onRemoveTag Called with a tag whose chip was tapped; null draws the tags as one line of text.
 */
@Composable
fun CardSheet(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    modifier: Modifier = Modifier,
    headerState: CardHeaderState = remember(card.id) { CardHeaderState() },
    scrollState: ScrollState = rememberScrollState(),
    headerIcons: CardHeaderIcons = defaultCardHeaderIcons(),
    onRemoveTag: ((Tag) -> Unit)? = null,
) {
    val page = MymindTheme.page
    val density = LocalDensity.current.density
    LaunchedEffect(scrollState, headerState, density) {
        snapshotFlow { (scrollState.value / density).toInt() }.collect(headerState::onScroll)
    }
    Box(
        modifier
            .fillMaxSize()
            .statusBarsPadding()
            .padding(top = CARD_SHEET_TOP_GAP)
            .clip(RoundedCornerShape(topStart = SHEET_RADIUS, topEnd = SHEET_RADIUS))
            .background(page.document),
    ) {
        Column(Modifier.fillMaxSize().verticalScroll(scrollState).navigationBarsPadding()) {
            Spacer(Modifier.height(CARD_HEADER_HEIGHT))
            CardBody(card, onRemoveTag)
        }
        CardHeader(
            title = cardHeading(card),
            color = MymindTheme.chrome.page,
            state = headerState,
            onClose = onClose,
            page = page.document,
            onMenu = onMenu,
            icons = headerIcons,
        )
    }
}

/** The card's tags as chips; tapping a chip's cross takes the tag off. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagChips(tags: List<Tag>, style: TextStyle, onRemove: (Tag) -> Unit) {
    val haptic = LocalHapticFeedback.current
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        tags.forEach { tag ->
            Row(
                Modifier
                    .clip(RoundedCornerShape(50))
                    .background(MymindTheme.page.surface)
                    .padding(start = 14.dp, end = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(tag.name.orEmpty(), Modifier.padding(vertical = 6.dp), style = style)
                Text(
                    "\u00D7",
                    Modifier
                        .semantics { contentDescription = "Remove tag ${tag.name.orEmpty()}" }
                        .clickable(role = Role.Button) {
                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                            onRemove(tag)
                        }
                        .padding(horizontal = 8.dp, vertical = 6.dp),
                    style = style,
                )
            }
        }
    }
}

/** The card's own content: picture, centred title, text and tags. */
@Composable
private fun CardBody(card: Card, onRemoveTag: ((Tag) -> Unit)?) {
    val page = MymindTheme.page
    val fonts = MymindTheme.fonts
    val body = TextStyle(color = page.text, fontSize = 19.sp, lineHeight = 1.5.em, fontFamily = FontFamily.Serif)
    Column(Modifier.fillMaxWidth().padding(horizontal = BODY_SIDE, vertical = 30.dp)) {
        val kind = tileKind(card)
        if (kind == TileKind.Media || kind == TileKind.Video) {
            CardPicture(card, play = kind == TileKind.Video)
            Spacer(Modifier.height(24.dp))
        }
        if (kind != TileKind.Note) {
            Text(
                cardHeading(card),
                Modifier.fillMaxWidth(),
                style = TextStyle(color = page.text, fontSize = 31.sp, lineHeight = 1.06.em, fontFamily = fonts.louize, textAlign = TextAlign.Center),
            )
            Spacer(Modifier.height(30.dp))
        }
        val text = if (kind == TileKind.Note) noteText(card) else listOfNotNull(card.description, card.note).joinToString("\n\n")
        if (text.isNotBlank()) Text(text, style = body)
        val tags = card.tags.orEmpty().filter { !it.name.isNullOrEmpty() }
        if (tags.isNotEmpty()) {
            Spacer(Modifier.height(24.dp))
            val style = TextStyle(color = page.muted, fontSize = 15.sp, fontFamily = fonts.nunitoSemiBold)
            if (onRemoveTag == null) {
                Text(tags.joinToString("  ") { it.name.orEmpty() }, style = style)
            } else {
                TagChips(tags, style, onRemoveTag)
            }
        }
    }
}

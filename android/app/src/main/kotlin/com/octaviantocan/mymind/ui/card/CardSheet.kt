package com.octaviantocan.mymind.ui.card

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.interaction.MutableInteractionSource
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.layout.layout
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.collapse
import androidx.compose.ui.semantics.dismiss
import androidx.compose.ui.semantics.expand
import androidx.compose.ui.semantics.paneTitle
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
import kotlin.math.roundToInt

/** Gap between the status bar and the top of an opened card. */
val CARD_SHEET_TOP_GAP = 24.dp

private val SHEET_RADIUS = 24.dp
private val GRABBER_HEIGHT = 24.dp
private val BODY_SIDE = 30.dp

/**
 * A card slides up over the library to half height, expands near the status bar, and follows a finger down to
 * dismiss. Content scrolls under the header; a drag that already scrolled content never pulls the sheet down.
 *
 * @param card Card to show.
 * @param onClose Called after the sheet has animated out, from the caret, back, scrim or a downward swipe.
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
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    val position = remember { CardSheetPosition() }
    val latestClose by rememberUpdatedState(onClose)
    val requestDismiss = { position.dismiss(scope) { latestClose() } }
    val speed = with(density) { 200.dp.toPx() }
    val connection = remember(position, scope, speed) { CardSheetScroll(position, scope, speed, requestDismiss) }
    val drag = rememberDraggableState { position.dragBy(it) }
    BackHandler { requestDismiss() }
    LaunchedEffect(scrollState, headerState, density) {
        snapshotFlow { (scrollState.value / density.density).toInt() }.collect(headerState::onScroll)
    }
    Box(modifier.fillMaxSize()) {
        Box(
            Modifier.fillMaxSize()
                .graphicsLayer { alpha = position.scrim }
                .background(Color.Black.copy(alpha = 0.4f))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClickLabel = "Close card",
                    onClick = requestDismiss,
                ),
        )
        Box(Modifier.fillMaxSize().statusBarsPadding().padding(top = CARD_SHEET_TOP_GAP)) {
            Box(
                Modifier.fillMaxSize()
                    .onSizeChanged { position.measured(it.height.toFloat(), scope) }
                    .graphicsLayer {
                        translationY = position.offset
                        alpha = if (position.height > 0f) 1f else 0f
                    }
                    .clip(RoundedCornerShape(topStart = SHEET_RADIUS, topEnd = SHEET_RADIUS))
                    .background(page.document)
                    .semantics {
                        paneTitle = "Card details"
                        dismiss { requestDismiss(); true }
                        expand { position.expand(scope, true); true }
                        collapse { position.expand(scope, false); true }
                    }
                    .nestedScroll(connection)
                    .draggable(drag, Orientation.Vertical, enabled = !position.closing, onDragStopped = { velocity ->
                        position.settle(scope, velocity, speed, requestDismiss)
                    }),
            ) {
                Column(
                    Modifier.fillMaxWidth().layout { measurable, constraints ->
                        val shown = (position.height - minOf(position.offset, position.height / 2f))
                            .roundToInt().coerceIn(0, constraints.maxHeight)
                        val content = measurable.measure(constraints.copy(minHeight = shown, maxHeight = shown))
                        layout(content.width, content.height) { content.place(0, 0) }
                    }.verticalScroll(scrollState).navigationBarsPadding(),
                ) {
                    Spacer(Modifier.height(GRABBER_HEIGHT + CARD_HEADER_HEIGHT))
                    CardBody(card, onRemoveTag)
                }
                Column {
                    Box(
                        Modifier.fillMaxWidth().height(GRABBER_HEIGHT)
                            .clickable(onClickLabel = "Expand card", role = Role.Button) { position.expand(scope, true) },
                        contentAlignment = Alignment.Center,
                    ) {
                        Box(Modifier.size(36.dp, 4.dp).clip(RoundedCornerShape(2.dp)).background(page.muted))
                    }
                    CardHeader(
                        title = cardHeading(card),
                        color = MymindTheme.chrome.page,
                        state = headerState,
                        onClose = requestDismiss,
                        page = page.document,
                        onMenu = onMenu,
                        icons = headerIcons,
                    )
                }
            }
        }
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

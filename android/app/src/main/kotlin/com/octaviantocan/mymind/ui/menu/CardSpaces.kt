package com.octaviantocan.mymind.ui.menu

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.cardSurfaceShadow
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.ui.library.CardSpaceCreationState
import com.octaviantocan.mymind.ui.library.CardThumbnail
import com.octaviantocan.mymind.ui.library.SPACE_COLORS
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.parseHexColor
import java.net.URI

/**
 * The space Add to space suggests for a card.
 *
 * @property space The space.
 * @property alike How many cards like this one are in it.
 */
data class SpaceSuggestion(val space: Space, val alike: Int)

/**
 * What the space choices show for one card.
 *
 * @property spaces Spaces in the user's order.
 * @property selected Ids of the spaces the card is in.
 * @property latest A space's first cards by space id, for its mosaic and peek.
 * @property suggestion The space suggested for the card, or null.
 */
data class SpaceChoices(
    val spaces: List<Space>,
    val selected: Set<String>,
    val latest: Map<String, List<Card>> = emptyMap(),
    val suggestion: SpaceSuggestion? = null,
)

/**
 * @param card The card being placed.
 * @param library Cards on hand, such as the gallery's loaded pages.
 * @param spaces Spaces in the user's order.
 * @return The space holding the most other cards that share a tag or a site with [card], leaving out the spaces it is in
 *   and smart spaces; a tie goes to the earlier space. Null when no other card is alike.
 */
fun suggestSpace(card: Card, library: List<Card>, spaces: List<Space>): SpaceSuggestion? {
    val tags = tagKeys(card)
    val site = siteKey(card)
    if (tags.isEmpty() && site == null) return null
    val counts = HashMap<String, Int>()
    library.forEach { other ->
        if (other.id == card.id) return@forEach
        val alike = (site != null && siteKey(other) == site) || tagKeys(other).any { it in tags }
        if (alike) other.spaces.orEmpty().toSet().forEach { counts[it] = (counts[it] ?: 0) + 1 }
    }
    val own = card.spaces.orEmpty().toSet()
    return spaces.filter { it.id !in own && it.smart != true }
        .mapNotNull { space -> counts[space.id]?.let { SpaceSuggestion(space, it) } }
        .maxByOrNull { it.alike }
}

/**
 * @param card The card being placed.
 * @param spaces Spaces in the user's order.
 * @param suggestion The suggested space, or null.
 * @param limit How many to return.
 * @return The spaces Choose one shows in place: the suggested one, then the ones the card is in, then the rest in the
 *   user's order.
 */
fun inlineSpaces(card: Card, spaces: List<Space>, suggestion: SpaceSuggestion?, limit: Int = INLINE_SPACES): List<Space> {
    val own = card.spaces.orEmpty().toSet()
    return (listOfNotNull(suggestion?.space) + spaces.filter { it.id in own } + spaces).distinctBy { it.id }.take(limit)
}

private fun tagKeys(card: Card) = card.tags.orEmpty().mapNotNullTo(HashSet()) { it.name?.trim()?.lowercase()?.takeIf(String::isNotEmpty) }

private fun siteKey(card: Card): String? = (card.domain?.takeIf { it.isNotBlank() } ?: card.url?.let { runCatching { URI(it).host }.getOrNull() })
    ?.lowercase()?.removePrefix("www.")

private fun spaceNames(choices: SpaceChoices): String? =
    choices.spaces.filter { it.id in choices.selected }.mapNotNull { it.name }.ifEmpty { null }?.let { "In " + it.joinToString(", ") }

/** "1 card" or "12 cards"; null when the count is unknown. */
private fun cardCount(space: Space): String? = space.count?.let { if (it == 1) "1 card" else "$it cards" }

/**
 * Add to space inside Choose one: the card's spaces, then the suggested space and two others as small mosaics that put
 * the card in or take it out, then All spaces.
 *
 * @param choices The spaces and the card's place in them.
 * @param card The card being placed.
 * @param onToggle Puts the card in a space or takes it out.
 * @param onAll Opens every space, or the New space form when there are none.
 */
@Composable
internal fun InlineSpaces(choices: SpaceChoices, card: Card, onToggle: (Space) -> Unit, onAll: () -> Unit) {
    val colors = menuColors()
    val shape = RoundedCornerShape(16.dp)
    // Fixed while the sheet is open, so a space does not move under the finger that just tapped it.
    val shown = remember(card.id, choices.spaces, choices.suggestion) { inlineSpaces(card, choices.spaces, choices.suggestion) }
    Column(Modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row)
        .border(2.dp, colors.rowText, shape).padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 14.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clip(CircleShape).background(colors.rowText.copy(alpha = 0.1f)), contentAlignment = Alignment.Center) {
                Image(painterResource(R.drawable.addtospaceicon), null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(colors.rowText))
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("Add to space", style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont))
                Text(spaceNames(choices) ?: "Not in a space yet", style = mutedStyle(colors), maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
        Row(Modifier.fillMaxWidth().padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.Top) {
            shown.forEach { space ->
                MiniSpace(space, choices.latest[space.id].orEmpty(), space.id in choices.selected, space.id == choices.suggestion?.space?.id,
                    Modifier.weight(1f)) { onToggle(space) }
            }
            MiniButton(if (choices.spaces.isEmpty()) "New space" else "All spaces",
                if (choices.spaces.isEmpty()) R.drawable.ic_library_add else R.drawable.ic_caret_right, Modifier.weight(1f), onAll)
            repeat(INLINE_SPACES - shown.size) { Spacer(Modifier.weight(1f)) }
        }
    }
}

/** One space in Add to space: its mosaic over its ring and name, with a badge when it is the suggested one. */
@Composable
private fun MiniSpace(space: Space, latest: List<Card>, selected: Boolean, suggested: Boolean, modifier: Modifier, onClick: () -> Unit) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    val name = space.name.orEmpty()
    Column(
        modifier.clip(RoundedCornerShape(12.dp))
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Checkbox) { haptic.tick(); onClick() }
            .semantics(mergeDescendants = true) {
                contentDescription = name
                stateDescription = if (selected) "In this space" else "Not in this space"
                this.selected = selected
            }
            .padding(vertical = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        // The ring sits on the mosaic's corner so the name has the whole width; four places share a phone's width.
        Box {
            SpaceMosaic(latest, 56.dp)
            SpaceRing(parseHexColor(space.color) ?: colors.rowText, selected, 20.dp,
                Modifier.align(Alignment.TopEnd).offset(x = 6.dp, y = (-6).dp), backdrop = colors.row)
        }
        Text(name, style = TextStyle(color = colors.rowText, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont, textAlign = TextAlign.Center),
            maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (suggested) Text("Suggested", style = mutedStyle(colors).copy(fontSize = 13.sp, lineHeight = 18.sp, fontWeight = FontWeight.Medium,
            textAlign = TextAlign.Center), maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/** The last place in Add to space: an outlined square that opens every space. */
@Composable
private fun MiniButton(label: String, icon: Int, modifier: Modifier, onClick: () -> Unit) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    val square = RoundedCornerShape(14.dp)
    Column(
        modifier.clip(RoundedCornerShape(12.dp))
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { haptic.tick(); onClick() }
            .padding(vertical = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(Modifier.size(56.dp).border(1.5.dp, colors.rowText.copy(alpha = 0.22f), square), contentAlignment = Alignment.Center) {
            Image(painterResource(icon), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(colors.rowText))
        }
        Text(label, style = TextStyle(color = colors.rowText, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont), maxLines = 1,
            overflow = TextOverflow.Ellipsis)
    }
}

/**
 * A space's first four cards in a two by two grid; a card without a picture, or a missing one, is a quiet square.
 *
 * @param cards The space's first cards.
 * @param size Side of the square.
 */
@Composable
internal fun SpaceMosaic(cards: List<Card>, size: Dp, modifier: Modifier = Modifier) {
    val colors = menuColors()
    Column(modifier.size(size).clip(RoundedCornerShape(14.dp)).background(colors.sheet), verticalArrangement = Arrangement.spacedBy(2.dp)) {
        repeat(2) { row ->
            Row(Modifier.fillMaxWidth().weight(1f), horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                repeat(2) { column ->
                    val card = cards.getOrNull(row * 2 + column)
                    val cell = Modifier.weight(1f).fillMaxHeight()
                    if (card != null && galleryPicture(card) != null) CardThumbnail(card, cell)
                    else Box(cell.background(colors.rowText.copy(alpha = 0.08f)))
                }
            }
        }
    }
}

/**
 * A space's ring in its own colour, filled with a check when the card is in the space; it bumps when that changes.
 *
 * @param color The space's colour.
 * @param selected True when the card is in the space.
 * @param size Outer side of the ring.
 * @param modifier Modifier for the ring's place.
 * @param backdrop The ring's inside when the card is not in the space, and an outline around it, for a ring over a picture;
 *   transparent leaves both out.
 */
@Composable
internal fun SpaceRing(color: Color, selected: Boolean, size: Dp, modifier: Modifier = Modifier, backdrop: Color = Color.Transparent) {
    val fill by animateColorAsState(if (selected) color else backdrop, label = "ringFill")
    val check by animateFloatAsState(if (selected) 1f else 0f, label = "ringCheck")
    val scale = remember { Animatable(1f) }
    var first by remember { mutableStateOf(true) }
    LaunchedEffect(selected) {
        if (first) {
            first = false
            return@LaunchedEffect
        }
        val bump = spring<Float>(dampingRatio = 0.75f, stiffness = 400f)
        scale.animateTo(if (selected) 1.1f else 0.9f, bump)
        scale.animateTo(1f, bump)
    }
    val outline = if (backdrop == Color.Transparent) Modifier else Modifier.background(backdrop, CircleShape).padding(2.dp)
    Box(modifier.size(size).scale(scale.value).then(outline).clip(CircleShape).background(fill).border(2.dp, color, CircleShape),
        contentAlignment = Alignment.Center) {
        Image(painterResource(R.drawable.spaces_checkmark), null, Modifier.size(size * 0.62f).alpha(check), colorFilter = ColorFilter.tint(Color.White))
    }
}

/**
 * Every space as a save slot, the suggested one first: a tap puts the card in or takes it out, holding one shows its
 * latest cards until the finger lifts, and New space makes a space in place and adds the card to it.
 *
 * @param choices The spaces and the card's place in them.
 * @param creation The model's pending creation and its last result.
 * @param onToggle Puts the card in a space or takes it out.
 * @param onCreate Makes a space with a name and colour and adds the card to it.
 * @param onAcknowledge Clears a finished creation, once its form has closed or when it opens again.
 */
@Composable
fun AllSpacesPage(
    choices: SpaceChoices,
    creation: CardSpaceCreationState,
    onToggle: (Space) -> Unit,
    onCreate: (name: String, color: String) -> Unit,
    onAcknowledge: () -> Unit = {},
) {
    var peek by remember { mutableStateOf<String?>(null) }
    var forming by rememberSaveable { mutableStateOf(choices.spaces.isEmpty() || creation.pending) }
    val suggested = choices.suggestion?.space?.let { space -> choices.spaces.firstOrNull { it.id == space.id } ?: space }
    LaunchedEffect(creation.result) {
        if (creation.result is ApiResult.Ok) {
            forming = false
            onAcknowledge()
        }
    }
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 8.dp, bottom = 30.dp)) {
        val slot = @Composable { space: Space, detail: String? ->
            SpaceSlot(space, choices.latest[space.id].orEmpty(), space.id in choices.selected, detail, peek == space.id,
                onToggle = { onToggle(space) }, onPeek = { peek = if (it) space.id else null })
        }
        if (suggested != null) {
            CardMenuHeading("Suggested", first = true)
            Column(Modifier.padding(top = 12.dp)) {
                val alike = choices.suggestion?.alike ?: 0
                slot(suggested, if (alike == 1) "1 card like this one is here" else "$alike cards like this one are here")
            }
        }
        val rest = choices.spaces.filter { it.id != suggested?.id }
        if (rest.isNotEmpty()) CardMenuHeading("Your spaces", first = suggested == null)
        Column(Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            rest.forEach { space -> slot(space, cardCount(space)) }
            if (forming) NewSpaceForm(creation, onCreate)
            else NewSpaceSlot {
                onAcknowledge()
                forming = true
            }
        }
        if (choices.spaces.isNotEmpty()) Text("Hold a space to look inside", Modifier.fillMaxWidth().padding(top = 16.dp),
            style = mutedStyle(menuColors()).copy(textAlign = TextAlign.Center))
    }
}

/** One space as a raised save slot; held, it opens to show its latest cards. */
@Composable
private fun SpaceSlot(
    space: Space,
    latest: List<Card>,
    selected: Boolean,
    detail: String?,
    peeking: Boolean,
    onToggle: () -> Unit,
    onPeek: (Boolean) -> Unit,
) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    val shape = RoundedCornerShape(16.dp)
    val toggle by rememberUpdatedState(onToggle)
    val peekNow by rememberUpdatedState(onPeek)
    val edge by animateColorAsState(if (selected || peeking) colors.rowText else Color.Transparent, tween(200), label = "slotEdge")
    val name = space.name.orEmpty()
    Column(
        Modifier.fillMaxWidth().heightIn(min = 88.dp).cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row)
            .border(2.dp, edge, shape)
            .pointerInput(space.id) {
                detectTapGestures(
                    onPress = {
                        tryAwaitRelease()
                        peekNow(false)
                    },
                    onLongPress = {
                        haptic.tick()
                        peekNow(true)
                    },
                    onTap = {
                        haptic.tick()
                        toggle()
                    },
                )
            }
            .semantics(mergeDescendants = true) {
                role = Role.Checkbox
                contentDescription = name
                stateDescription = if (selected) "In this space" else "Not in this space"
                this.selected = selected
                onClick(label = if (selected) "Take the card out" else "Put the card in") { toggle(); true }
            }
            .padding(start = 12.dp, end = 16.dp, top = 12.dp, bottom = 12.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
            SpaceMosaic(latest, 64.dp)
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(name, style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont),
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (detail != null) Text(detail, style = mutedStyle(colors))
            }
            SpaceRing(parseHexColor(space.color) ?: colors.rowText, selected, 20.dp)
        }
        AnimatedVisibility(peeking, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
            Column(Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Latest in $name · let go to close", style = mutedStyle(colors))
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    val pictures = latest.filter { galleryPicture(it) != null }.take(PEEK_PICTURES)
                    pictures.forEach { CardThumbnail(it, Modifier.weight(1f).aspectRatio(1f).clip(RoundedCornerShape(12.dp))) }
                    repeat(PEEK_PICTURES - pictures.size) { Spacer(Modifier.weight(1f)) }
                }
            }
        }
    }
}

/** The outlined slot that opens the New space form. */
@Composable
private fun NewSpaceSlot(onClick: () -> Unit) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    val shape = RoundedCornerShape(16.dp)
    Row(
        Modifier.fillMaxWidth().heightIn(min = 88.dp).clip(shape).border(1.5.dp, colors.rowText.copy(alpha = 0.22f), shape)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { haptic.tick(); onClick() }
            .padding(start = 12.dp, end = 16.dp, top = 12.dp, bottom = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(64.dp).clip(RoundedCornerShape(14.dp)).background(colors.rowText.copy(alpha = 0.08f)), contentAlignment = Alignment.Center) {
            Image(painterResource(R.drawable.ic_library_add), null, Modifier.size(24.dp), colorFilter = ColorFilter.tint(colors.rowText))
        }
        Text("New space", style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont))
    }
}

/** The New space form in place: a name, a colour and Create and add. A failure keeps what was typed for another try. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun NewSpaceForm(creation: CardSpaceCreationState, onCreate: (name: String, color: String) -> Unit) {
    val colors = menuColors()
    val shape = RoundedCornerShape(16.dp)
    var name by rememberSaveable { mutableStateOf("") }
    var color by rememberSaveable { mutableStateOf(SPACE_COLORS.first()) }
    val busy = creation.pending
    val ready = name.isNotBlank() && !busy
    val error = (creation.result as? ApiResult.Failed)?.let { describe(it.failure) }
    val submit = { if (ready) onCreate(name.trim(), color) }
    Column(Modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row).border(2.dp, colors.rowText, shape)
        .padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Text("New space", style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium, fontFamily = CardMenuFont))
        BasicTextField(
            value = name,
            onValueChange = { name = it },
            enabled = !busy,
            singleLine = true,
            textStyle = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont),
            cursorBrush = SolidColor(colors.rowText),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { submit() }),
            modifier = Modifier.fillMaxWidth().semantics { contentDescription = "Space name" }
                .clip(RoundedCornerShape(12.dp)).background(colors.sheet).padding(horizontal = 14.dp, vertical = 13.dp),
            decorationBox = { field ->
                Box {
                    if (name.isEmpty()) Text("Name it", style = mutedStyle(colors).copy(fontSize = 17.sp, lineHeight = 22.sp))
                    field()
                }
            },
        )
        FlowRow(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            SPACE_COLORS.forEach { choice ->
                Box(Modifier.size(40.dp).clip(CircleShape)
                    .clickable(enabled = !busy, role = Role.RadioButton) { color = choice }
                    .semantics { contentDescription = "Space color $choice"; selected = color == choice },
                    contentAlignment = Alignment.Center) {
                    SpaceRing(parseHexColor(choice) ?: colors.rowText, color == choice, 28.dp)
                }
            }
        }
        if (error != null) Text(error, style = mutedStyle(colors).copy(color = colors.dangerText))
        Row(
            Modifier.fillMaxWidth().height(52.dp).clip(CircleShape).background(colors.rowText.copy(alpha = if (ready) 1f else 0.4f))
                .clickable(enabled = ready, role = Role.Button) { submit() },
            horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (!busy) Image(painterResource(R.drawable.ic_library_add), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.sheet))
            Text(if (busy) "Creating…" else "Create and add", style = TextStyle(color = colors.sheet, fontSize = 17.sp, lineHeight = 22.sp,
                fontWeight = FontWeight.Medium, fontFamily = CardMenuFont))
        }
    }
}

private fun mutedStyle(colors: MenuColors) = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont)

private const val INLINE_SPACES = 3
private const val PEEK_PICTURES = 5

package com.octaviantocan.mymind.ui.share

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.share.ShareDetails
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.card.cardSurfaceShadow
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.library.CardThumbnail
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.parseHexColor
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.MenuColors
import com.octaviantocan.mymind.ui.menu.SpaceRing
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.menu.addTag
import com.octaviantocan.mymind.ui.menu.menuColors
import java.net.URI
import java.util.Locale

/** How far saving what was shared has come. */
enum class SavePhase {
    /** Still saving. */
    Saving,

    /** Everything that could be saved is saved. */
    Saved,

    /** Kept on the phone while it is offline; it goes up once it is online. */
    Kept,

    /** Nothing could be saved. */
    Failed,
}

/**
 * A file that was shared.
 *
 * @property uri Its address.
 * @property name Its name.
 * @property mime Its media type.
 * @property size Its length in bytes.
 */
data class SharedFile(val uri: String, val name: String, val mime: String, val size: Long)

/**
 * Everything the share sheet shows and lets the user add.
 *
 * @param items What was shared.
 * @param phase How far saving has come.
 */
@Stable
class ShareSheetState(items: List<ShareItem> = emptyList(), phase: SavePhase = SavePhase.Saving) {
    /** What was shared. */
    var items by mutableStateOf(items)

    /** The shared files, described. */
    var files by mutableStateOf<List<SharedFile>>(emptyList())

    /** The saved cards as the library has them; they fill in as ThyMind reads them. */
    var cards by mutableStateOf<List<Card>>(emptyList())
    var phase by mutableStateOf(phase)

    /** How many items are saved. */
    var saved by mutableIntStateOf(0)

    /** How many items could not be saved. */
    var failed by mutableIntStateOf(0)

    /** Why something could not be saved, or null. */
    var problem by mutableStateOf<String?>(null)
    var spaces by mutableStateOf<List<Space>>(emptyList())
    var allTags by mutableStateOf<List<String>>(emptyList())
    var recentTags by mutableStateOf<List<String>>(emptyList())

    /** The space suggested for what was saved, or null. */
    var suggestion by mutableStateOf<SpaceSuggestion?>(null)

    /** What is in the field. */
    var typed by mutableStateOf("")
    var tags by mutableStateOf<List<String>>(emptyList())
    var note by mutableStateOf("")

    /** ThyMind's own tags the user took off what was saved. */
    var untagged by mutableStateOf<Set<String>>(emptySet())

    /** True while ThyMind is still finding tags for what was saved, which holds the sheet up a little longer. */
    var findingTags by mutableStateOf(false)

    /** ThyMind's own tags on what was saved, once it has found them, without the ones taken off; none for several things. */
    val foundTags: List<String>
        get() = if (items.size != 1) emptyList() else thymindTags(cards.firstOrNull()).filterNot { name -> untagged.any { it.equals(name, ignoreCase = true) } }

    /** Ids of the chosen spaces, in the order they were chosen. */
    var spaceIds by mutableStateOf<List<String>>(emptyList())

    /** True once ThyMind folded what was saved into a card saved before (share-sheet-says-when-merged). */
    var merged by mutableStateOf(false)

    /** True once the sheet has been touched, which keeps it open until it is closed. */
    var touched by mutableStateOf(false)

    /** The details to put on the saved cards. */
    fun details() = ShareDetails(tags, note.trim(), spaceIds.toSet(), untagged)

    /** The rows under the field, for what is typed now. */
    fun suggestions(): List<ShareSuggestion> {
        val onCards = cards.flatMap { card -> card.tags.orEmpty().mapNotNull { it.name } }.filterNot { name -> untagged.any { it.equals(name, ignoreCase = true) } }
        return shareSuggestions(typed, spaces, spaceIds.toSet(), allTags, recentTags, tags + onCards, suggestion)
    }

    /**
     * Adds what a row offers and empties the field.
     *
     * @param row The row tapped, or the first one on Enter.
     */
    fun add(row: ShareSuggestion) {
        when (row) {
            is ShareSuggestion.AddToSpace -> if (row.space.id !in spaceIds) spaceIds = spaceIds + row.space.id
            is ShareSuggestion.AddTag -> {
                val back = untagged.filterTo(HashSet()) { it.equals(row.tag, ignoreCase = true) }
                if (back.isNotEmpty()) untagged = untagged - back else tags = addTag(tags, row.tag)
            }
            is ShareSuggestion.UseAsNote -> note = row.text
        }
        typed = ""
    }
}

/**
 * @param card A saved card, or null.
 * @return The names of the tags ThyMind put on [card] itself, in their order.
 */
fun thymindTags(card: Card?): List<String> =
    card?.tags.orEmpty().filter { (it.flags ?: 0) and AI_TAG != 0 }.mapNotNull { it.name?.takeIf(String::isNotBlank) }.distinctBy { it.lowercase(Locale.ROOT) }

/**
 * The two lines of the saved row.
 *
 * @property title What was shared, by its name once ThyMind knows it.
 * @property detail Where it is from and how big it is, what ThyMind is doing with it, or why it was not saved.
 */
data class SavedLines(val title: String, val detail: String)

/**
 * @param state The sheet.
 * @return The saved row's lines for what was shared and how far saving has come.
 */
fun savedLines(state: ShareSheetState): SavedLines {
    val problem = (state.problem ?: NOT_SAVED).takeIf { state.phase == SavePhase.Failed }
    val count = state.items.size
    if (count > 1) {
        val detail = problem ?: when {
            state.phase != SavePhase.Saving && state.failed > 0 -> "${state.saved} saved, ${state.failed} could not be"
            else -> state.files.takeIf { it.size == count }?.let { files -> sizeText(files.sumOf { it.size }) } ?: "From the app you shared from"
        }
        return SavedLines("$count ${pluralNoun(state.files.map { it.mime }, count)}", detail)
    }
    val card = state.cards.firstOrNull()
    return when (val item = state.items.firstOrNull()) {
        is ShareItem.Text -> if (LINK.matches(item.text)) {
            val read = card?.title?.takeIf { it.isNotBlank() && card.type != PLACEHOLDER }
            val site = card?.siteName?.takeIf { it.isNotBlank() } ?: card?.domain ?: hostOf(item.text)
            SavedLines(read ?: shortLink(item.text), problem ?: when {
                read != null -> listOfNotNull(site, card.author?.takeIf { it.isNotBlank() }).distinct().joinToString(" · ")
                state.phase == SavePhase.Kept -> "ThyMind reads it once you’re online"
                state.phase == SavePhase.Saved -> "Getting the page…"
                else -> site.orEmpty()
            })
        } else {
            SavedLines(item.text.lineSequence().firstOrNull { it.isNotBlank() }?.trim().orEmpty(), problem ?: "Words you shared")
        }
        is ShareItem.Stream -> {
            val file = state.files.firstOrNull()
            val title = card?.title?.takeIf { it.isNotBlank() } ?: file?.name ?: "A file"
            SavedLines(title, problem ?: listOfNotNull(kindOf(file?.mime), file?.size?.let(::sizeText)).joinToString(" · "))
        }
        null -> SavedLines("", problem.orEmpty())
    }
}

/**
 * @param state The sheet.
 * @return The header's words: saving, saved, kept on the phone, or nothing saved.
 */
fun statusText(state: ShareSheetState): String = when (state.phase) {
    SavePhase.Saving -> if (state.items.size > 1) "Saving ${minOf(state.saved + state.failed + 1, state.items.size)} of ${state.items.size}" else "Saving to your mind"
    SavePhase.Saved -> if (state.merged) MERGED else "Saved to your mind"
    SavePhase.Kept -> "Kept on this phone"
    SavePhase.Failed -> NOT_SAVED
}

/**
 * What the share target shows once something was shared (share-field-takes-everything): the saved row, what has been
 * added, one field for a space, a tag or a note with its suggestions, and Done. Left untouched once saved it fades
 * away after [SAVED_FADE_MS], or [PROBLEM_FADE_MS] when nothing was saved (share-sheet-fades-by-itself).
 *
 * @param state What the sheet shows.
 * @param onClose Called once the sheet has gone, by itself or closed; the details are still to be put on the cards.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ShareSheet(state: ShareSheetState, onClose: () -> Unit) {
    val failed = state.phase == SavePhase.Failed
    ShareFrame(
        statusText(state),
        counting = state.phase != SavePhase.Saving && !state.touched && (failed || !state.findingTags),
        waitMs = if (failed) PROBLEM_FADE_MS else SAVED_FADE_MS,
        onTouch = { state.touched = true },
        onClosed = onClose,
        titleContent = { StatusTitle(state) },
    ) { frame ->
        val colors = menuColors()
        Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 16.dp, bottom = 24.dp)) {
            SavedRow(state, colors)
            if (!failed) {
                Added(state, colors)
                val rows = state.suggestions()
                CommandField(state, colors) { rows.firstOrNull()?.let(state::add) }
                if (rows.isNotEmpty()) Suggestions(rows, typed = state.typed.isNotBlank(), colors) { state.add(it) }
            }
            Spacer(Modifier.height(20.dp))
            ShareDoneButton(if (failed) "Okay" else "Done", frame::left, frame.close)
        }
    }
}

/** The header: a spinner, a tick or the offline mark, then the words. */
@Composable
private fun StatusTitle(state: ShareSheetState) {
    val colors = menuColors()
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically) {
        when (state.phase) {
            SavePhase.Saving -> CircularProgressIndicator(Modifier.size(18.dp), color = colors.rowText, strokeWidth = 2.dp, trackColor = colors.rowText.copy(alpha = 0.1f))
            SavePhase.Saved -> Tick(colors, 22.dp)
            SavePhase.Kept -> Image(painterResource(R.drawable.offline_notice), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(colors.rowText))
            SavePhase.Failed -> Unit
        }
        Text(statusText(state), maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun Tick(colors: MenuColors, size: Dp) {
    Box(Modifier.size(size).clip(CircleShape).background(colors.rowText), contentAlignment = Alignment.Center) {
        Image(painterResource(R.drawable.spaces_checkmark), null, Modifier.size(size * 0.62f), colorFilter = ColorFilter.tint(colors.row))
    }
}

/** What was shared, as a raised row: its picture, its two lines and how far saving it has come. */
@Composable
private fun SavedRow(state: ShareSheetState, colors: MenuColors) {
    val shape = RoundedCornerShape(16.dp)
    val lines = savedLines(state)
    Row(
        Modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row)
            .padding(start = 10.dp, top = 10.dp, bottom = 10.dp, end = 14.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SavedPicture(state, colors, Modifier.size(56.dp).clip(RoundedCornerShape(12.dp)))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(lines.title, style = labelStyle(colors), maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(lines.detail, style = mutedStyle(colors), maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
    }
}

/** The saved row's picture: the card's own once it has one, else the shared photo, else a mark for what it is. */
@Composable
private fun SavedPicture(state: ShareSheetState, colors: MenuColors, modifier: Modifier) {
    val quiet = colors.rowText.copy(alpha = 0.07f)
    val items = state.items
    if (items.size > 1) {
        Column(modifier.background(colors.sheet), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            repeat(2) { row ->
                Row(Modifier.fillMaxWidth().weight(1f), horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                    repeat(2) { column ->
                        val at = row * 2 + column
                        Box(Modifier.weight(1f).fillMaxHeight().background(quiet)) {
                            if (at < items.size) OnePicture(items[at], state.cards.getOrNull(at), state.files.getOrNull(at), colors, Modifier.fillMaxSize(), mark = false)
                        }
                    }
                }
            }
        }
    } else {
        Box(modifier.background(quiet), contentAlignment = Alignment.Center) {
            items.firstOrNull()?.let { OnePicture(it, state.cards.firstOrNull(), state.files.firstOrNull(), colors, Modifier.fillMaxSize(), mark = true) }
        }
    }
}

@Composable
private fun OnePicture(item: ShareItem, card: Card?, file: SharedFile?, colors: MenuColors, modifier: Modifier, mark: Boolean) {
    when {
        card != null && galleryPicture(card) != null -> CardThumbnail(card, modifier)
        item is ShareItem.Stream && file?.mime?.startsWith("image/") != false ->
            AsyncImage(model = item.uri, contentDescription = null, contentScale = ContentScale.Crop, modifier = modifier)
        mark -> Box(modifier, contentAlignment = Alignment.Center) {
            val icon = when {
                item is ShareItem.Text -> if (LINK.matches(item.text)) R.drawable.ic_link else R.drawable.ic_quote
                file?.mime?.startsWith("video/") == true -> R.drawable.ic_video
                file?.mime == PDF -> R.drawable.ic_pdf
                else -> R.drawable.ic_file
            }
            Image(painterResource(icon), null, Modifier.size(24.dp), colorFilter = ColorFilter.tint(colors.rowText.copy(alpha = 0.65f)))
        }
    }
}

/**
 * The spaces and tags on what was saved, ThyMind's own tags first, and the note; each comes off with its cross. While
 * ThyMind is still finding tags a quiet Finding tags chip holds their place.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun Added(state: ShareSheetState, colors: MenuColors) {
    val spaces = state.spaceIds.mapNotNull { id -> state.spaces.firstOrNull { it.id == id } }
    val found = state.foundTags
    val finding = state.findingTags && found.isEmpty()
    if (spaces.isNotEmpty() || state.tags.isNotEmpty() || found.isNotEmpty() || finding) {
        FlowRow(Modifier.fillMaxWidth().padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            spaces.forEach { space ->
                AddedChip(space.name.orEmpty(), "Take out of ${space.name.orEmpty()}", colors, lead = {
                    Box(Modifier.size(12.dp).clip(CircleShape).background(parseHexColor(space.color) ?: colors.rowText))
                }) { state.spaceIds = state.spaceIds - space.id }
            }
            found.forEach { tag ->
                AddedChip(tag, "Take off $tag", colors, lead = {
                    Image(painterResource(R.drawable.ic_library_search_spark), null, Modifier.size(16.dp), colorFilter = ColorFilter.tint(colors.rowText))
                }) { state.untagged = state.untagged + tag }
            }
            if (finding) FindingChip(colors)
            state.tags.forEach { tag ->
                AddedChip(tag, "Take off $tag", colors, lead = {
                    Image(painterResource(R.drawable.tags), null, Modifier.size(16.dp), colorFilter = ColorFilter.tint(colors.rowText))
                }) { state.tags = state.tags - tag }
            }
        }
    }
    if (state.note.isNotBlank()) {
        val shape = RoundedCornerShape(16.dp)
        Row(
            Modifier.fillMaxWidth().padding(top = 10.dp).cardSurfaceShadow(shape).clip(shape).background(colors.row)
                .padding(start = 14.dp, top = 12.dp, bottom = 12.dp, end = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Image(painterResource(R.drawable.notes), null, Modifier.padding(top = 1.dp).size(18.dp), colorFilter = ColorFilter.tint(colors.rowText))
            Text(state.note, Modifier.weight(1f), style = mutedStyle(colors).copy(color = colors.rowText), maxLines = 4, overflow = TextOverflow.Ellipsis)
            Cross("Take off the note", colors) { state.note = "" }
        }
    }
}

@Composable
private fun AddedChip(label: String, removeLabel: String, colors: MenuColors, lead: @Composable () -> Unit, onRemove: () -> Unit) {
    val haptic = rememberSheetHaptics()
    Row(
        Modifier.height(36.dp).cardSurfaceShadow(CircleShape).clip(CircleShape).background(colors.row)
            .clickable(role = Role.Button, onClickLabel = removeLabel) { haptic.tick(); onRemove() }
            .padding(start = 12.dp, end = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        lead()
        Text(label, style = mutedStyle(colors).copy(color = colors.rowText), maxLines = 1, overflow = TextOverflow.Ellipsis)
        Image(painterResource(R.drawable.ic_sheet_close), null, Modifier.size(16.dp), colorFilter = ColorFilter.tint(colors.rowText.copy(alpha = 0.7f)))
    }
}

/** A chip that breathes while ThyMind finds the tags for what was saved. */
@Composable
private fun FindingChip(colors: MenuColors) {
    val breath by rememberInfiniteTransition(label = "finding tags").animateFloat(
        initialValue = 0.45f,
        targetValue = 0.9f,
        animationSpec = infiniteRepeatable(tween(FINDING_BREATH_MS), RepeatMode.Reverse),
        label = "finding tags alpha",
    )
    Row(
        Modifier.height(36.dp).clip(CircleShape).background(colors.row.copy(alpha = 0.6f)).graphicsLayer { alpha = breath }
            .semantics { contentDescription = FINDING_TAGS }.padding(horizontal = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(painterResource(R.drawable.ic_library_search_spark), null, Modifier.size(16.dp), colorFilter = ColorFilter.tint(colors.rowText))
        Text(FINDING_TAGS, style = mutedStyle(colors).copy(color = colors.rowText), maxLines = 1)
    }
}

@Composable
private fun Cross(label: String, colors: MenuColors, modifier: Modifier = Modifier, onClick: () -> Unit) {
    val haptic = rememberSheetHaptics()
    Box(modifier.size(28.dp).clip(CircleShape).clickable(role = Role.Button, onClickLabel = label) { haptic.tick(); onClick() }
        .semantics { contentDescription = label }, contentAlignment = Alignment.Center) {
        Image(painterResource(R.drawable.ic_sheet_close), null, Modifier.size(16.dp), colorFilter = ColorFilter.tint(colors.rowText.copy(alpha = 0.7f)))
    }
}

/** The one field: a space, a tag or a note, added with Enter or the arrow. */
@Composable
private fun CommandField(state: ShareSheetState, colors: MenuColors, onEnter: () -> Unit) {
    val ready = state.typed.isNotBlank()
    BasicTextField(
        value = state.typed,
        onValueChange = { state.typed = it },
        singleLine = true,
        textStyle = labelStyle(colors),
        cursorBrush = SolidColor(colors.rowText),
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { if (ready) onEnter() }),
        modifier = Modifier.fillMaxWidth().padding(top = 12.dp).semantics { contentDescription = FIELD_HINT },
        decorationBox = { field ->
            Row(
                Modifier.fillMaxWidth().heightIn(min = 52.dp).cardSurfaceShadow(CircleShape).clip(CircleShape).background(colors.row)
                    .padding(start = 18.dp, end = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.weight(1f).padding(vertical = 14.dp)) {
                    if (state.typed.isEmpty()) Text(FIELD_HINT, style = labelStyle(colors).copy(color = colors.rowText.copy(alpha = 0.65f)), maxLines = 1)
                    field()
                }
                Spacer(Modifier.width(10.dp))
                Box(
                    Modifier.size(40.dp).clip(CircleShape).background(colors.rowText.copy(alpha = if (ready) 1f else 0.25f))
                        .clickable(enabled = ready, role = Role.Button, onClickLabel = "Add") { onEnter() }
                        .semantics { contentDescription = "Add" },
                    contentAlignment = Alignment.Center,
                ) {
                    Image(painterResource(R.drawable.ic_arrow_up), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(colors.row))
                }
            }
        },
    )
}

/** The rows under the field, in one raised group; the first says Enter adds it once something is typed. */
@Composable
private fun Suggestions(rows: List<ShareSuggestion>, typed: Boolean, colors: MenuColors, onPick: (ShareSuggestion) -> Unit) {
    val shape = RoundedCornerShape(16.dp)
    val haptic = rememberSheetHaptics()
    Column(Modifier.fillMaxWidth().padding(top = 12.dp).cardSurfaceShadow(shape, note = true).clip(shape).background(colors.row)) {
        rows.forEachIndexed { index, row ->
            if (index > 0) Box(Modifier.padding(start = 52.dp).fillMaxWidth().height(1.dp).background(colors.rowText.copy(alpha = 0.06f)))
            Row(
                Modifier.fillMaxWidth().heightIn(min = 52.dp).clickable(role = Role.Button) { haptic.tick(); onPick(row) }.padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(14.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.width(22.dp), contentAlignment = Alignment.Center) {
                    when (row) {
                        is ShareSuggestion.AddToSpace -> SpaceRing(parseHexColor(row.space.color) ?: colors.rowText, selected = false, size = 20.dp)
                        is ShareSuggestion.AddTag -> Image(painterResource(R.drawable.tags), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(colors.rowText))
                        is ShareSuggestion.UseAsNote -> Image(painterResource(R.drawable.notes), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(colors.rowText))
                    }
                }
                val (label, detail) = rowLines(row)
                Column(Modifier.weight(1f).padding(vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(label, style = labelStyle(colors), maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text(detail, style = mutedStyle(colors), maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                if (index == 0 && typed) Text("Enter", style = mutedStyle(colors))
            }
        }
    }
}

/**
 * @param row A row under the field.
 * @return Its label and its muted line.
 */
fun rowLines(row: ShareSuggestion): Pair<String, String> = when (row) {
    is ShareSuggestion.AddToSpace -> "Add to ${row.space.name.orEmpty()}" to row.detail
    is ShareSuggestion.AddTag -> when {
        row.fresh -> "Tag it “${row.tag}”" to "A new tag"
        row.recent -> row.tag to "Recent tag"
        else -> row.tag to "Tag"
    }
    is ShareSuggestion.UseAsNote -> "Use as your note" to row.text
}

private fun labelStyle(colors: MenuColors) = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont)

private fun mutedStyle(colors: MenuColors) = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont)

/** "theatlantic.com/magazine/walking" for a link, without its scheme, `www.` or a closing slash. */
private fun shortLink(link: String) = link.substringAfter("://").removePrefix("www.").trimEnd('/')

private fun hostOf(link: String): String? = runCatching { URI(link).host }.getOrNull()?.removePrefix("www.")

private fun kindOf(mime: String?): String? = when {
    mime == null -> null
    mime.startsWith("image/") -> "Photo"
    mime.startsWith("video/") -> "Video"
    mime == PDF -> "PDF"
    else -> "File"
}

private fun pluralNoun(mimes: List<String>, count: Int): String = when {
    mimes.size < count -> "things"
    mimes.all { it.startsWith("image/") } -> "photos"
    mimes.all { it.startsWith("video/") } -> "videos"
    mimes.all { it == PDF } -> "PDFs"
    else -> "files"
}

/** "640 KB", "2.1 MB" or "31 MB". */
internal fun sizeText(bytes: Long): String {
    val mb = bytes / 1_048_576.0
    return when {
        mb < 1 -> "${maxOf(1L, bytes / 1024)} KB"
        mb < 10 -> String.format(Locale.ROOT, "%.1f MB", mb)
        else -> "${mb.toLong()} MB"
    }
}

private val LINK = Regex("""^https?://\S+$""")
private const val PLACEHOLDER = "Placeholder"
private const val PDF = "application/pdf"
private const val NOT_SAVED = "Nothing was saved"
private const val MERGED = "Merged with an earlier save"
private const val FIELD_HINT = "Add a space, a tag or a note"
private const val FINDING_TAGS = "Finding tags…"
private const val FINDING_BREATH_MS = 700

/** A tag's flag for one ThyMind added by itself. */
private const val AI_TAG = 2

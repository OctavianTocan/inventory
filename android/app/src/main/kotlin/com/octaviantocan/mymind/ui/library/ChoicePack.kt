package com.octaviantocan.mymind.ui.library

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.theme.AppFontFamily
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** How long a batch's Undo stays, longer than a delete's, because more changed. */
const val UNDO_MANY_MS = 8_000L

/** The dark of the backpack's badge, the app's ink, so the gold spark reads in both themes. */
private val BadgeInk = Color(0xFF0E333A)
private val PackGold = Color(0xFFF2C14E)

/** The colours of the backpack's sheets and bars, from the theme's own sheet colours. */
private class PackColors(val scrim: Color, val paper: Color, val tile: Color, val ink: Color, val soft: Color, val muted: Color, val handle: Color)

@Composable
private fun packColors(): PackColors {
    val sheet = MymindTheme.theme.palette(MymindTheme.colors.dark).sheet
    return PackColors(sheet.scrim, sheet.paper, sheet.raised, sheet.text, sheet.text.copy(alpha = 0.08f), sheet.text.copy(alpha = 0.64f), sheet.grabber)
}

private fun packText(size: Int, weight: FontWeight = FontWeight.Normal, color: Color) =
    TextStyle(fontFamily = AppFontFamily, fontSize = size.sp, lineHeight = (size + 6).sp, fontWeight = weight, color = color)

/**
 * The backpack's bar while finds are chosen: how many, then what to do with them. It sits over the bottom bar.
 *
 * @param count How many finds are chosen.
 * @param onTag Opens the tag sheet.
 * @param onSpace Opens the space sheet.
 * @param onClose Empties the backpack.
 * @param modifier Places the bar.
 */
@Composable
fun ChoicePackBar(count: Int, onTag: () -> Unit, onSpace: () -> Unit, onClose: () -> Unit, modifier: Modifier = Modifier) {
    val c = packColors()
    val paper = LibraryStyle.colors.raised
    val ink = LibraryStyle.colors.text
    Row(
        modifier.heightIn(min = 68.dp).clip(CircleShape).background(paper).padding(start = 12.dp, end = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(40.dp).clip(CircleShape).background(BadgeInk), contentAlignment = Alignment.Center) {
            Canvas(Modifier.size(16.dp)) { drawPath(sparkPath(size.width), PackGold) }
        }
        Column(Modifier.weight(1f).semantics(mergeDescendants = true) { liveRegion = LiveRegionMode.Polite }) {
            Text(findsLabel(count), style = packText(15, FontWeight.SemiBold, ink), maxLines = 1)
            Text("in your backpack", style = packText(12, color = LibraryStyle.colors.muted), maxLines = 1)
        }
        PackPill("Tag", strong = true, onClick = onTag)
        PackPill("Space", strong = false, onClick = onSpace)
        Box(
            Modifier.size(44.dp).clip(CircleShape).background(c.soft)
                .clickable(role = Role.Button, onClickLabel = "Empty the backpack", onClick = onClose)
                .semantics { contentDescription = "Empty the backpack" },
            contentAlignment = Alignment.Center,
        ) {
            Canvas(Modifier.size(14.dp)) {
                val stroke = 2.dp.toPx()
                drawLine(c.ink, Offset(0f, 0f), Offset(size.width, size.height), stroke)
                drawLine(c.ink, Offset(size.width, 0f), Offset(0f, size.height), stroke)
            }
        }
    }
}

@Composable
private fun PackPill(label: String, strong: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val c = packColors()
    Box(
        modifier.heightIn(min = 44.dp).clip(CircleShape).background(if (strong) c.ink else c.soft)
            .clickable(role = Role.Button, onClick = onClick).padding(horizontal = 16.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = packText(14, FontWeight.Medium, if (strong) c.paper else c.ink))
    }
}

/** "1 find" or "N finds". */
internal fun findsLabel(count: Int) = if (count == 1) "1 find" else "$count finds"

/**
 * A sheet rising over a dim scrim, the app's paper with a grabber, closed by a tap outside or Back.
 *
 * @param onDismiss Closes the sheet.
 * @param content What the sheet holds.
 */
@Composable
private fun PackSheet(onDismiss: () -> Unit, content: @Composable () -> Unit) {
    val c = packColors()
    BackHandler(onBack = onDismiss)
    Box(Modifier.fillMaxSize()) {
        Box(Modifier.fillMaxSize().background(c.scrim)
            .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null, onClickLabel = "Close", onClick = onDismiss))
        Column(
            Modifier.align(Alignment.BottomCenter).fillMaxWidth().clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
                .background(c.paper).clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) {}
                .navigationBarsPadding().padding(start = 16.dp, end = 16.dp, top = 10.dp, bottom = 20.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(Modifier.align(Alignment.CenterHorizontally).size(width = 36.dp, height = 4.dp).clip(CircleShape).background(c.handle))
            content()
        }
    }
}

/**
 * Puts tags on many finds, or takes them off. Each tag shows whether all, some or none of the chosen finds have it: a
 * tap puts it on all of them, and taking tags off is its own mode, so a stray tap never removes one.
 *
 * @param cards The chosen finds.
 * @param known Tag names to offer besides those the finds have: recent first, then every tag.
 * @param onDone Called with the tags to put on and the tags to take off; both may be empty.
 * @param onDismiss Closes the sheet without changing anything.
 */
@Composable
fun ChoiceTagSheet(cards: List<Card>, known: List<String>, onDone: (put: List<String>, takeOff: List<String>) -> Unit, onDismiss: () -> Unit) {
    val c = packColors()
    var takingOff by remember { mutableStateOf(false) }
    val marked = remember { mutableStateListOf<String>() }
    val typed = remember { mutableStateListOf<String>() }
    var draft by remember { mutableStateOf("") }
    val counts = remember(cards) { tagCounts(cards) }
    val total = cards.size
    val names = remember(counts, known, typed.toList(), draft, takingOff) {
        val pool = if (takingOff) counts.keys.toList()
        else (typed.map(String::lowercase) + counts.entries.sortedByDescending { it.value }.map { it.key } + known.map { it.trim().lowercase() })
        val query = draft.trim().lowercase()
        pool.filter { it.isNotEmpty() && (query.isEmpty() || query in it) }.distinct().take(60)
    }
    val addDraft = {
        val name = draft.trim().lowercase()
        if (name.isNotEmpty()) {
            if (name !in typed) typed.add(0, name)
            if (name !in marked) marked.add(name)
            takingOff = false
            draft = ""
        }
    }
    PackSheet(onDismiss) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text((if (takingOff) "Take tags off " else "Tag ") + findsLabel(total), Modifier.weight(1f), style = packText(20, FontWeight.SemiBold, c.ink))
            PackPill("Done", strong = true, onClick = {
                val chosen = marked.toList()
                if (takingOff) onDone(emptyList(), chosen) else onDone(chosen, emptyList())
            })
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            ModeChip("Put on", !takingOff) { if (takingOff) { takingOff = false; marked.clear() } }
            ModeChip("Take off", takingOff) { if (!takingOff) { takingOff = true; marked.clear() } }
        }
        Row(
            Modifier.fillMaxWidth().heightIn(min = 48.dp).clip(CircleShape).background(c.tile).padding(start = 16.dp, end = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(Modifier.weight(1f)) {
                if (draft.isEmpty()) Text(if (takingOff) "Search their tags" else "Search or add a tag", style = packText(15, color = c.muted))
                BasicTextField(draft, { draft = it }, Modifier.fillMaxWidth(), singleLine = true, textStyle = packText(15, color = c.ink),
                    cursorBrush = SolidColor(c.ink),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done), keyboardActions = KeyboardActions(onDone = { if (!takingOff) addDraft() }))
            }
            if (!takingOff && draft.isNotBlank()) PackPill("Add", strong = false, onClick = addDraft)
        }
        LazyVerticalGrid(GridCells.Fixed(3), Modifier.fillMaxWidth().heightIn(max = 340.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            items(names, key = { it }) { name ->
                val have = counts[name] ?: 0
                val on = name in marked
                TagTile(name, have, total, on, takingOff) { if (on) marked.remove(name) else marked.add(name) }
            }
        }
        Text(
            if (takingOff) "A tap marks a tag to come off every chosen find that has it."
            else "A half-filled dot means some of these finds have the tag. A tap puts it on all of them.",
            style = packText(13, color = c.muted),
        )
    }
}

@Composable
private fun ModeChip(label: String, selected: Boolean, onClick: () -> Unit) {
    val c = packColors()
    Box(
        Modifier.heightIn(min = 40.dp).clip(CircleShape).background(if (selected) c.ink else c.soft)
            .clickable(role = Role.Tab, onClick = onClick).semantics { stateDescription = if (selected) "Selected" else "" }
            .padding(horizontal = 16.dp),
        contentAlignment = Alignment.Center,
    ) { Text(label, style = packText(14, FontWeight.Medium, if (selected) c.paper else c.ink)) }
}

@Composable
private fun TagTile(name: String, have: Int, total: Int, marked: Boolean, takingOff: Boolean, onTap: () -> Unit) {
    val c = packColors()
    val share = tagShare(have, total)
    val full = if (takingOff) share == TagShare.All && !marked else marked || share == TagShare.All
    val half = !full && !marked && share == TagShare.Some
    val note = when {
        takingOff && marked -> "off $have"
        !takingOff && marked && share != TagShare.All -> "all $total"
        share == TagShare.Some -> "$have of $total"
        else -> ""
    }
    val said = when {
        takingOff && marked -> "comes off $have"
        marked -> "goes on all $total"
        share == TagShare.All -> "on all $total"
        share == TagShare.Some -> "on $have of $total"
        else -> "on none"
    }
    Column(
        Modifier.heightIn(min = 76.dp).clip(RoundedCornerShape(16.dp)).background(c.tile)
            .then(if (marked) Modifier.border(2.dp, c.ink, RoundedCornerShape(16.dp)) else Modifier)
            .clickable(role = Role.Checkbox, onClick = onTap)
            .clearAndSetSemantics { contentDescription = "#$name, $said" }
            .padding(10.dp),
        verticalArrangement = Arrangement.SpaceBetween,
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End, verticalAlignment = Alignment.CenterVertically) {
            if (note.isNotEmpty()) Text(note, style = packText(11, color = c.muted), maxLines = 1)
            Spacer(Modifier.width(4.dp))
            Canvas(Modifier.size(18.dp)) {
                val r = size.minDimension / 2f
                val stroke = 2.dp.toPx()
                when {
                    full -> drawCircle(c.ink, r)
                    half -> {
                        drawArc(c.ink, 90f, 180f, true, size = Size(size.width, size.height))
                        drawCircle(c.ink, r - stroke / 2f, style = Stroke(stroke))
                    }
                    else -> drawCircle(if (takingOff && marked) c.ink else c.handle, r - stroke / 2f, style = Stroke(stroke))
                }
            }
        }
        Spacer(Modifier.height(10.dp))
        Text("#$name", style = packText(14, color = c.ink).copy(textDecoration = if (takingOff && marked) TextDecoration.LineThrough else null),
            maxLines = 2, overflow = TextOverflow.Ellipsis)
    }
}

/**
 * Puts many finds into one space, picked from the list; finds already in it stay where they are.
 *
 * @param count How many finds are chosen.
 * @param spaces The spaces to offer.
 * @param onPick Called with the space picked.
 * @param onDismiss Closes the sheet without changing anything.
 */
@Composable
fun ChoiceSpaceSheet(count: Int, spaces: List<Space>, onPick: (Space) -> Unit, onDismiss: () -> Unit) {
    val c = packColors()
    PackSheet(onDismiss) {
        Text("Put ${findsLabel(count)} in a space", style = packText(20, FontWeight.SemiBold, c.ink))
        if (spaces.isEmpty()) Text("You have no spaces yet. Make one from the Spaces page.", style = packText(15, color = c.muted))
        LazyColumn(Modifier.fillMaxWidth().heightIn(max = 380.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            items(spaces.filter { it.smart != true }, key = { it.id }) { space ->
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 52.dp).clip(RoundedCornerShape(16.dp)).background(c.tile)
                        .clickable(role = Role.Button) { onPick(space) }.padding(horizontal = 16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Box(Modifier.size(14.dp).clip(CircleShape).background(spaceDot(space.color, c.muted)))
                    Text(space.name ?: "Untitled space", Modifier.weight(1f), style = packText(16, color = c.ink), maxLines = 1, overflow = TextOverflow.Ellipsis)
                    space.count?.let { Text(findsLabel(it), style = packText(13, color = c.muted)) }
                }
            }
        }
    }
}

private fun spaceDot(color: String?, fallback: Color): Color =
    color?.removePrefix("#")?.toLongOrNull(16)?.let { Color(0xFF000000 or it) } ?: fallback

/**
 * Says what a batch did, with Undo, for [UNDO_MANY_MS]; the bar shows the time left as it shortens.
 *
 * @param text What happened, such as "Tagged 12 finds".
 * @param onUndo Takes the batch back.
 * @param modifier Places the bar.
 */
@Composable
fun ChoiceUndoBar(text: String, onUndo: () -> Unit, modifier: Modifier = Modifier) {
    val c = packColors()
    var left by remember { mutableFloatStateOf(1f) }
    LaunchedEffect(Unit) {
        val start = withFrameNanos { it }
        while (left > 0f) {
            val now = withFrameNanos { it }
            left = (1f - (now - start) / 1_000_000f / UNDO_MANY_MS).coerceIn(0f, 1f)
        }
    }
    Row(
        modifier.heightIn(min = 52.dp).clip(CircleShape).background(c.ink).padding(start = 20.dp, end = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(text, Modifier.weight(1f).semantics { liveRegion = LiveRegionMode.Polite }, style = packText(14, color = c.paper), maxLines = 1,
            overflow = TextOverflow.Ellipsis)
        Box(
            Modifier.heightIn(min = 40.dp).clip(CircleShape).background(c.paper.copy(alpha = 0.14f))
                .clickable(role = Role.Button, onClickLabel = "Undo", onClick = onUndo).padding(horizontal = 14.dp),
            contentAlignment = Alignment.Center,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Undo", style = packText(14, FontWeight.SemiBold, PackGold))
                Canvas(Modifier.size(14.dp).clearAndSetSemantics {}) {
                    drawArc(PackGold.copy(alpha = 0.3f), 0f, 360f, false, style = Stroke(2.dp.toPx()))
                    drawArc(PackGold, -90f, 360f * left, false, style = Stroke(2.dp.toPx()))
                }
            }
        }
    }
}

/** The Inventory spark, a four-pointed star, filling a square of [side] pixels. */
private fun sparkPath(side: Float) = androidx.compose.ui.graphics.Path().apply {
    val c = side / 2f
    val inner = side * 0.12f
    moveTo(c, 0f); lineTo(c + inner, c - inner); lineTo(side, c); lineTo(c + inner, c + inner)
    lineTo(c, side); lineTo(c - inner, c + inner); lineTo(0f, c); lineTo(c - inner, c - inner); close()
}


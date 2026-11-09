package com.octaviantocan.mymind.ui.moment

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.TextAutoSize
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.library.parseHexColor
import com.octaviantocan.mymind.ui.share.ShareSheetState
import com.octaviantocan.mymind.ui.share.ShareSuggestion
import com.octaviantocan.mymind.ui.theme.AppFontFamily
import java.util.Locale

/** The words on the tag grid's button while ThyMind still looks for tags. */
const val FINDING_TAGS_TILE = "Finding tags"

private val Panel = Color(0xFF0B1A36)
private val Paper = Color(0xFFF4F2EF)
private val Ink = Color(0xFF0E333A)
private val Muted = Color(0xFFCFE0DE)
private val Hint = Color(0xFF8FA9AC)

/**
 * One tile of the tag grid: a tag, whether it is on what was saved, and whether ThyMind put it there.
 *
 * @property tag The tag's name.
 * @property on True while the tag is on what was saved.
 * @property byThyMind True for a tag ThyMind gave it.
 */
data class TagTile(val tag: String, val on: Boolean, val byThyMind: Boolean)

/**
 * @param state The share's state.
 * @return The tag grid's tiles for what is typed: ThyMind's tags, the user's, then recent tags, then any tag that holds
 *   the typed words; each tag once.
 */
fun tagTiles(state: ShareSheetState): List<TagTile> {
    val q = state.typed.trim().lowercase(Locale.ROOT)
    val found = state.foundTags
    val off = if (state.items.size == 1) com.octaviantocan.mymind.ui.share.thymindTags(state.cards.firstOrNull()).filter { name -> state.untagged.any { it.equals(name, true) } } else emptyList()
    val seen = HashSet<String>()
    val tiles = ArrayList<TagTile>()
    fun offer(tag: String, on: Boolean, byThyMind: Boolean) {
        if (!seen.add(tag.lowercase(Locale.ROOT))) return
        if (q.isNotEmpty() && q !in tag.lowercase(Locale.ROOT)) return
        tiles += TagTile(tag, on, byThyMind)
    }
    found.forEach { offer(it, on = true, byThyMind = true) }
    state.tags.forEach { offer(it, on = true, byThyMind = false) }
    off.forEach { offer(it, on = false, byThyMind = true) }
    state.recentTags.forEach { offer(it, on = false, byThyMind = false) }
    if (q.isNotEmpty()) state.allTags.forEach { offer(it, on = false, byThyMind = false) }
    return tiles
}

/**
 * Puts a tag on what was saved or takes it off, the way the share sheet's chips do: ThyMind's own tags are taken off
 * and put back, the user's are added and removed.
 *
 * @param state The share's state.
 * @param tile The tile tapped.
 */
fun toggleTag(state: ShareSheetState, tile: TagTile) {
    when {
        !tile.on -> state.add(ShareSuggestion.AddTag(tile.tag))
        tile.byThyMind -> state.untagged = state.untagged + tile.tag
        else -> state.tags = state.tags.filterNot { it.equals(tile.tag, ignoreCase = true) }
    }
}

/**
 * The back of the find's card as the tag step, M9 Icon grid (Octavian's pick, 5 November 2026): the spaces along the
 * top with the one the find is in lit, a field to search or add tags, a grid of tags with icons to tap on and off, and
 * a button that saves them. While [left] counts, a darker part drains from the button (share-sheet-fades-by-itself).
 *
 * @param state The share's state; taps change its spaces and tags.
 * @param accent The find's rarity colour.
 * @param left The part of the wait still to go, or null while nothing counts.
 * @param onTouch Called on every touch, which is expected to stop the wait.
 * @param onDone Called when the button is tapped.
 */
@Composable
internal fun TagGrid(state: ShareSheetState, accent: Color, left: () -> Float?, onTouch: () -> Unit, onDone: () -> Unit, modifier: Modifier = Modifier) {
    val latestTouch by rememberUpdatedState(onTouch)
    val haptic = rememberSheetHaptics()
    Box(
        modifier.clip(RoundedCornerShape(26.dp))
            .background(Brush.linearGradient(0f to accent, 0.38f to Color(0xFF3A2A12), 0.62f to accent, 1f to Color(0xFF2A1E0C), start = Offset.Zero, end = Offset(800f, 1100f)))
            .pointerInput(Unit) { awaitEachGesture { awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial); latestTouch() } }
            .padding(9.dp),
    ) {
        Column(
            Modifier.fillMaxSize().clip(RoundedCornerShape(18.dp)).background(Brush.verticalGradient(listOf(Color(0xFF10234A), Panel))).padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            SpaceRow(state, haptic::tick)
            SearchField(state, accent, haptic::tick)
            val tiles = tagTiles(state)
            val finding = state.findingTags && state.foundTags.isEmpty() && state.typed.isBlank()
            val grid = rememberLazyGridState()
            LazyVerticalGrid(
                GridCells.Fixed(3), Modifier.fillMaxWidth().weight(1f).fadeBottom { grid.canScrollForward }, grid,
                contentPadding = PaddingValues(bottom = 12.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                if (finding) item(key = "finding") { FindingTile() }
                items(tiles, key = { it.tag.lowercase(Locale.ROOT) }) { tile -> Tile(tile, accent) { haptic.tick(); toggleTag(state, tile) } }
            }
            val count = state.foundTags.size + state.tags.size
            DoneButton(if (count == 0) "Done" else "Save $count ${if (count == 1) "tag" else "tags"}", left) { haptic.tick(); onDone() }
        }
    }
}

/** The spaces to keep the find in; the one it is in is lit, and a tap moves it or takes it out. */
@Composable
private fun SpaceRow(state: ShareSheetState, tick: () -> Unit) {
    val suggested = state.suggestion?.space
    val spaces = (listOfNotNull(suggested) + state.spaces).filter { it.smart != true }.distinctBy { it.id }
    if (spaces.isEmpty()) return
    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        spaces.forEach { space ->
            val on = space.id in state.spaceIds
            Row(
                Modifier.height(38.dp).clip(RoundedCornerShape(12.dp)).background(if (on) Paper else Paper.copy(alpha = 0.05f))
                    .clickable(role = Role.Checkbox, onClickLabel = if (on) "Take out of ${space.name}" else "Keep in ${space.name}") {
                        tick(); state.spaceIds = if (on) state.spaceIds - space.id else listOf(space.id)
                    }
                    .semantics { stateDescription = if (on) "Kept here" else "" }
                    .padding(horizontal = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.size(8.dp).clip(CircleShape).background(parseHexColor(space.color) ?: Muted))
                Text(space.name.orEmpty(), style = text(13.5f, if (on) Ink else Muted, if (on) FontWeight.SemiBold else FontWeight.Normal), maxLines = 1)
            }
        }
    }
}

/** Search tags; words no tag has can be added as a new one. */
@Composable
private fun SearchField(state: ShareSheetState, accent: Color, tick: () -> Unit) {
    val q = state.typed.trim()
    val known = q.isEmpty() || (state.allTags + state.tags + state.foundTags + state.recentTags).any { it.equals(q, ignoreCase = true) }
    val add = { if (q.isNotEmpty()) { tick(); state.add(ShareSuggestion.AddTag(q.take(MAX_TAG))) } }
    Row(
        Modifier.fillMaxWidth().height(44.dp).clip(RoundedCornerShape(12.dp)).background(Paper.copy(alpha = 0.08f)).padding(start = 14.dp, end = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(SEARCH, Hint, 18f, 2.4f)
        Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            if (state.typed.isEmpty()) Text("Search tags", style = text(16f, Hint, FontWeight.Normal), maxLines = 1)
            BasicTextField(
                state.typed, { state.typed = it },
                Modifier.fillMaxWidth().semantics { contentDescription = "Search or add tags" },
                textStyle = text(16f, Paper, FontWeight.Normal), singleLine = true, cursorBrush = SolidColor(Paper),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                keyboardActions = KeyboardActions(onDone = { if (!known) add() else tagTiles(state).firstOrNull { !it.on }?.let { tick(); toggleTag(state, it); state.typed = "" } }),
            )
        }
        if (!known) {
            Box(
                Modifier.height(34.dp).clip(RoundedCornerShape(10.dp)).background(accent).clickable(role = Role.Button, onClickLabel = "Add $q as a tag") { add() }
                    .padding(horizontal = 12.dp),
                contentAlignment = Alignment.Center,
            ) { Text("Add", style = text(14f, Color(0xFF0B2227), FontWeight.Bold)) }
        }
    }
}

@Composable
private fun Tile(tile: TagTile, accent: Color, onTap: () -> Unit) {
    val pop = remember { Animatable(1f) }
    var taps by remember { mutableIntStateOf(0) }
    LaunchedEffect(taps) { if (taps > 0) { pop.snapTo(0.86f); pop.animateTo(1f, tween(320, easing = POP)) } }
    val shape = RoundedCornerShape(16.dp)
    Box(
        Modifier.height(TILE_HEIGHT).graphicsLayer { scaleX = pop.value; scaleY = pop.value }.clip(shape)
            .background(if (tile.on) Paper.copy(alpha = 0.13f) else Paper.copy(alpha = 0.04f))
            .then(if (tile.on) Modifier.border(2.dp, accent, shape) else Modifier)
            .clickable(role = Role.Checkbox, onClickLabel = if (tile.on) "Take off ${tile.tag}" else "Add ${tile.tag}") { taps++; onTap() }
            .semantics { stateDescription = if (tile.on) "On" else "Off" },
    ) {
        // The marks sit in a row of their own above the icon, 10 dp in from the corners, so neither touches the icon.
        Column(Modifier.fillMaxSize().padding(horizontal = 8.dp, vertical = 10.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Row(Modifier.fillMaxWidth().height(18.dp).padding(horizontal = 2.dp), verticalAlignment = Alignment.CenterVertically) {
                if (tile.byThyMind) {
                    Image(painterResource(R.drawable.ic_library_search_spark), "From Inventory", Modifier.size(14.dp),
                        colorFilter = ColorFilter.tint(Muted.copy(alpha = 0.8f)))
                }
                Spacer(Modifier.weight(1f))
                Box(
                    Modifier.size(18.dp).clip(RoundedCornerShape(5.dp))
                        .then(if (tile.on) Modifier.background(accent) else Modifier.border(1.5.dp, Paper.copy(alpha = 0.3f), RoundedCornerShape(5.dp))),
                    contentAlignment = Alignment.Center,
                ) { if (tile.on) Icon(CHECK, Color(0xFF0B2227), 11f, 4f) }
            }
            Spacer(Modifier.height(2.dp))
            Box(Modifier.size(38.dp).clip(RoundedCornerShape(11.dp)).background(if (tile.on) accent else Paper.copy(alpha = 0.08f)), contentAlignment = Alignment.Center) {
                Icon(tagIcon(tile.tag), if (tile.on) Color(0xFF0B2227) else Muted, 21f, 1.9f)
            }
            Spacer(Modifier.height(8.dp))
            // Long words shrink a little before they lose letters, for big system text.
            BasicText(tile.tag, style = text(13f, Paper, FontWeight.Normal).copy(textAlign = TextAlign.Center), maxLines = 1, overflow = TextOverflow.Ellipsis,
                autoSize = TextAutoSize.StepBased(minFontSize = 10.sp, maxFontSize = 13.sp))
        }
    }
}

/** Fades the last few dp of what is drawn while [more] says there is more below, so a cut-off row reads as a scroll. */
private fun Modifier.fadeBottom(more: () -> Boolean) = graphicsLayer { compositingStrategy = CompositingStrategy.Offscreen }.drawWithContent {
    drawContent()
    if (!more()) return@drawWithContent
    val f = 28.dp.toPx()
    drawRect(Brush.verticalGradient(listOf(Color.Transparent, Color.Black), startY = size.height - f, endY = size.height),
        topLeft = Offset(0f, size.height - f), size = Size(size.width, f), blendMode = BlendMode.DstOut)
}

/** A tile that breathes while ThyMind finds the tags for what was saved. */
@Composable
private fun FindingTile() {
    val breath by rememberInfiniteTransition(label = "finding tags").animateFloat(0.45f, 0.9f, infiniteRepeatable(tween(700), RepeatMode.Reverse), label = "breath")
    Column(
        Modifier.height(TILE_HEIGHT).clip(RoundedCornerShape(16.dp)).background(Paper.copy(alpha = 0.04f)).graphicsLayer { alpha = breath }
            .semantics { contentDescription = FINDING_TAGS_TILE },
        verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Image(painterResource(R.drawable.ic_library_search_spark), null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(Muted))
        Text(FINDING_TAGS_TILE, style = text(13f, Paper, FontWeight.Normal), maxLines = 1)
    }
}

@Composable
private fun DoneButton(label: String, left: () -> Float?, onClick: () -> Unit) {
    Box(
        Modifier.fillMaxWidth().height(48.dp).clip(RoundedCornerShape(24.dp)).background(Paper)
            .drawBehind { left()?.let { drawRect(Color(0xFF0B2227).copy(alpha = 0.18f), size = Size(size.width * it, size.height)) } }
            .clickable(role = Role.Button, onClickLabel = label, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) { Text(label, style = text(16f, Ink, FontWeight.SemiBold), maxLines = 1) }
}

/** A line icon drawn from a 24-unit path, [size] dp across. */
@Composable
private fun Icon(path: String, color: Color, size: Float, width: Float) {
    val shape = remember(path) { PathParser().parsePathString(path).toPath() }
    Box(Modifier.size(size.dp).drawBehind {
        val k = this.size.width / 24f
        scale(k, k, pivot = Offset.Zero) { drawPath(shape, color, style = Stroke(width, cap = StrokeCap.Round, join = StrokeJoin.Round)) }
    })
}

private fun text(size: Float, color: Color, weight: FontWeight) = TextStyle(color = color, fontSize = size.sp, lineHeight = (size * 1.3f).sp, fontWeight = weight, fontFamily = AppFontFamily)

/**
 * @param tag A tag's name.
 * @return A line icon for it from the words it holds, or a tag for any other.
 */
internal fun tagIcon(tag: String): String {
    val t = tag.lowercase(Locale.ROOT)
    if (DECADE.containsMatchIn(t)) return ICONS.getValue("time")
    val words = t.split(NOT_WORD).filter { it.isNotEmpty() }
    return ICON_WORDS.firstOrNull { (keys, _) -> keys.any { key -> words.any { it.startsWith(key) } } }?.second?.let(ICONS::getValue) ?: TAG
}

private const val MAX_TAG = 40
private val TILE_HEIGHT = 106.dp
private val POP = CubicBezierEasing(0.3f, 1.5f, 0.5f, 1f)
private val NOT_WORD = Regex("""[^\p{L}\p{N}]+""")
private val DECADE = Regex("""\b\d{2,4}s\b""")
private const val TAG = "M3 12V4h8l10 10-8 8zM7.5 8.5h.01"
private const val SEARCH = "M11 17.5a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13zM16 16l4.5 4.5"
private const val CHECK = "M5 12.5l4.5 4.5L19 7.5"
private val ICONS = mapOf(
    "camera" to "M4 8h3l2-2.5h6L17 8h3v11H4zM15.5 13a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0z",
    "time" to "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",
    "music" to "M9 18V5l11-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM20 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z",
    "book" to "M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2zM4 19V5M8 7h7",
    "food" to "M7 3v8M5 3v5a2 2 0 0 0 4 0V3M7 11v10M17 3c-2 0-3 3-3 6s1 4 3 4v8",
    "place" to "M12 21s-7-6.2-7-11a7 7 0 0 1 14 0c0 4.8-7 11-7 11zM12 12.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z",
    "design" to "M4 20l4-1 11-11-3-3L5 16zM14 6l3 3",
    "code" to "M8 7l-5 5 5 5M16 7l5 5-5 5",
    "video" to "M5 4h14v16H5zM10 9l5 3-5 3z",
    "heart" to "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
    "gift" to "M4 10h16v10H4zM3 7h18v3H3zM12 7v13M12 7c-2-4-6-3-5 0M12 7c2-4 6-3 5 0",
    "idea" to "M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9V16h7v-2.1A6 6 0 0 0 12 3z",
    "leaf" to "M5 19c0-8 6-14 15-14 0 9-6 15-14 15zM5 19l8-8",
    "pulse" to "M3 12h4l2-5 4 10 2-5h6",
    "work" to "M4 7h16v12H4zM9 7V5h6v2M4 12h16",
    "home" to "M4 11l8-7 8 7v9H4zM10 20v-6h4v6",
    "news" to "M4 5h13v14H6a2 2 0 0 1-2-2zM17 9h3v8a2 2 0 0 1-2 2M8 9h5M8 13h5",
    "shirt" to "M8 4l4 2 4-2 4 4-3 3v9H7v-9L4 8z",
    "star" to "M12 2.8l2.8 5.9 6.4.8-4.7 4.4 1.2 6.4L12 17.2l-5.7 3.1 1.2-6.4-4.7-4.4 6.4-.8z",
    "gear" to "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9L7 7M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1",
    "film" to "M5 3h14v18H5zM5 7h14M5 17h14M9 3v4M15 3v4M9 17v4M15 17v4",
)
private val ICON_WORDS = listOf(
    listOf("camera", "photo", "lens") to "camera",
    listOf("film", "35mm", "cinema", "movie") to "film",
    listOf("music", "song", "audio", "podcast", "album") to "music",
    listOf("book", "read", "essay", "article", "writing", "literature") to "book",
    listOf("food", "recipe", "cook", "eat", "coffee", "drink") to "food",
    listOf("travel", "trip", "map", "place", "city") to "place",
    listOf("design", "art", "ui", "ux", "typography", "illustration", "inspiration") to "design",
    listOf("code", "dev", "programming", "software", "github", "api") to "code",
    listOf("video", "youtube", "watch") to "video",
    listOf("wish", "love", "favourite", "favorite") to "heart",
    listOf("gift", "buy", "shop") to "gift",
    listOf("idea", "think", "ai", "learn") to "idea",
    listOf("nature", "walk", "outdoor", "plant", "garden") to "leaf",
    listOf("health", "fitness", "sport", "run") to "pulse",
    listOf("work", "business", "career", "startup") to "work",
    listOf("home", "house", "interior") to "home",
    listOf("news", "politic") to "news",
    listOf("fashion", "style", "cloth", "wear") to "shirt",
    listOf("space", "astro", "science", "star") to "star",
    listOf("gear", "tool", "tech") to "gear",
    listOf("history", "vintage", "retro", "old") to "time",
)

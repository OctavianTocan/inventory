package com.octaviantocan.mymind.ui.library

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.AiSpace
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.scrollEdgeFade
import com.octaviantocan.mymind.ui.shell.FieldHeight
import com.octaviantocan.mymind.ui.shell.FootIcon
import com.octaviantocan.mymind.ui.shell.FootMotion
import com.octaviantocan.mymind.ui.shell.FootPressTheme
import com.octaviantocan.mymind.ui.shell.FootShadow
import com.octaviantocan.mymind.ui.shell.floatingBar
import com.octaviantocan.mymind.ui.shell.footShadow
import com.octaviantocan.mymind.ui.shell.rememberFootMotionEnabled
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter

/**
 * What the search page shows near the thumb, beside the results it is given.
 *
 * @property query Text in the search field.
 * @property typing True while the keyboard is up, when the dock offers suggestions for the typed text.
 * @property engine The signed-in engine's name, such as "Claude Code", or null when none is.
 * @property recentSearches The searches last pressed on this phone, newest first.
 * @property tags The person's tags to start from, most recently used first.
 * @property openedLately Cards opened lately, latest first.
 * @property onlyRanked True while the results show only the cards the engine ranked first.
 */
@Immutable
data class SearchDockState(
    val query: String = "",
    val typing: Boolean = false,
    val engine: String? = null,
    val recentSearches: List<String> = emptyList(),
    val tags: List<String> = emptyList(),
    val openedLately: List<Card> = emptyList(),
    val onlyRanked: Boolean = false,
)

/**
 * What the dock's taps do.
 *
 * @property onAsk Presses search for the typed text, so the signed-in engine reads it.
 * @property onSearchFor Searches for the given text at once, as a tapped recent search does.
 * @property onForgetSearches Forgets the recent searches.
 * @property onOnlyRanked Shows only the ranked cards when true, every match when false.
 */
@Stable
class SearchDockActions(
    val onAsk: () -> Unit = {},
    val onSearchFor: (String) -> Unit = {},
    val onForgetSearches: () -> Unit = {},
    val onOnlyRanked: (Boolean) -> Unit = {},
)

/**
 * What Claude Code or Codex is doing with a pressed search, and after, as the dock's first line says it.
 *
 * @param rank What the engine is doing.
 * @return The words to show.
 */
fun aiNote(rank: AiRank): String = when (rank) {
    is AiRank.Understanding -> "${rank.engine} is reading what you wrote"
    is AiRank.Ranking -> "${rank.engine} is ranking what it found"
    is AiRank.Ranked -> if (rank.count > 0) "${rank.engine} ranked ${rank.count} first" else "${rank.engine} found nothing closer to what you mean"
    is AiRank.Failed -> rank.message
}

/** A source card's name on its chip: its title, else its site, else its type. */
internal fun sourceLabel(card: Card): String {
    val name = card.title?.trim()?.takeIf { it.isNotEmpty() } ?: card.domain?.takeIf { it.isNotBlank() } ?: card.type
    return if (name.length > SOURCE_CHARS) name.take(SOURCE_CHARS).trimEnd() + "…" else name
}

/** "1 card" or "24 cards". */
internal fun cardCount(count: Int): String = if (count == 1) "1 card" else "$count cards"

/** The longest word of a search of two words or more, to try on its own when nothing matched; else null. */
internal fun loneWord(query: String): String? {
    val words = query.trim().split(Regex("\\s+")).filter { it.isNotEmpty() }
    return if (words.size < 2) null else words.maxBy { it.length }
}

/** Just above the foot bar, in its side insets and raised with it by the keyboard (library-bars-rest-alike). */
internal fun Modifier.aboveFootBar(): Modifier = floatingBar().padding(bottom = FieldHeight + DOCK_GAP)

/**
 * The opened search page before anything is typed: things to start from at the top, and the recent searches and the
 * person's tags docked just above the bar.
 *
 * @param typeChips Card types to limit the search to.
 * @param dock Recent searches, tags and cards opened lately.
 * @param actions What the recent searches do.
 * @param onChoose Called with a tapped type or tag.
 * @param onOpenCard Called with a tapped card.
 * @param onLongPressCard Called with a held card, or null.
 */
@Composable
internal fun SearchStart(
    typeChips: List<Suggestion>,
    dock: SearchDockState,
    actions: SearchDockActions,
    onChoose: (Suggestion) -> Unit,
    onOpenCard: (Card) -> Unit,
    onLongPressCard: ((Card) -> Unit)?,
) {
    val colors = LibraryStyle.colors
    val scroll = rememberScrollState()
    Column(Modifier.fillMaxSize()) {
        Column(Modifier.weight(1f).fillMaxWidth().scrollEdgeFade(scroll, colors.background, top = 32.dp, bottom = 32.dp)
            .verticalScroll(scroll).padding(top = 20.dp, bottom = 16.dp)) {
            Text("Find something", Modifier.padding(horizontal = 20.dp, vertical = 12.dp).semantics { heading() },
                style = LibraryStyle.title, color = colors.text)
            if (dock.openedLately.isNotEmpty()) {
                SectionLabel("Opened lately")
                LazyRow(Modifier.fillMaxWidth().semantics { contentDescription = "Opened lately" },
                    contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    items(dock.openedLately, key = { it.id }) { card ->
                        GalleryTile(card, onClick = { onOpenCard(card) }, modifier = Modifier.size(RAIL_WIDTH, RAIL_HEIGHT),
                            onLongClick = onLongPressCard?.let { { it(card) } })
                    }
                }
            }
            SectionLabel("Kinds")
            ChipFlow(Modifier.fillMaxWidth().padding(horizontal = 20.dp)) {
                typeChips.forEach { chip -> SearchChip(chip.text, false) { onChoose(chip) } }
            }
        }
        if (dock.recentSearches.isEmpty() && dock.tags.isEmpty()) {
            // Keeps the kinds clear of the bar.
            Spacer(Modifier.fillMaxWidth().aboveFootBar())
        } else {
            Column(Modifier.fillMaxWidth().aboveFootBar()) {
                if (dock.recentSearches.isNotEmpty()) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        SectionLabel("Recent", Modifier.weight(1f), start = 8.dp)
                        DockTextButton("Clear", Modifier.semantics { contentDescription = "Clear recent searches" }, actions.onForgetSearches)
                    }
                    dock.recentSearches.forEach { text ->
                        DockRow({ FootIcon(R.drawable.ic_library_search, tint = colors.muted, size = 20.dp) }, text, null,
                            horizontal = 8.dp, minHeight = 48.dp) { actions.onSearchFor(text) }
                    }
                }
                if (dock.tags.isNotEmpty()) {
                    SectionLabel("Your tags", start = 8.dp)
                    LazyRow(Modifier.fillMaxWidth(), contentPadding = PaddingValues(horizontal = 8.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        items(dock.tags, key = { it }) { tag ->
                            SearchChip(tag.lowercase(), false) { onChoose(Suggestion("tag", tag, "tag:$tag")) }
                        }
                    }
                }
            }
        }
    }
}

/**
 * The card docked above the bar while there is a search: suggestions while typing, the engine's work and answer once
 * search is pressed, or what to try when nothing matched; and under it the filter the search is limited to.
 *
 * @param results The matches on screen.
 * @param filter The filter the search is limited to, or null.
 * @param suggestions Tags and types that match the typed text.
 * @param ai What the engine is doing with a pressed search, or null.
 * @param dock The typed text, the engine and what else the dock shows.
 * @param actions What the dock's buttons do.
 * @param scroll The results' scroll, which folds the engine's card down to its first line.
 * @param onChoose Called with a tapped suggestion.
 * @param onRemoveFilter Called when the filter is taken off.
 * @param onAiTag Called with a tapped tag the engine found.
 * @param onOpenSpace Called with a tapped space the engine found.
 * @param onOpenCard Called with a tapped card.
 */
@Composable
internal fun BoxScope.SearchResultsDock(
    results: FeedState,
    filter: SearchFilter?,
    suggestions: List<Suggestion>,
    ai: AiRank?,
    dock: SearchDockState,
    actions: SearchDockActions,
    scroll: LazyListState,
    onChoose: (Suggestion) -> Unit,
    onRemoveFilter: () -> Unit,
    onAiTag: (String) -> Unit,
    onOpenSpace: (AiSpace) -> Unit,
    onOpenCard: (Card) -> Unit,
) {
    val moving = rememberFootMotionEnabled()
    val offered = suggestions.filter { it.queryText != filter?.queryText }.take(OFFERED_SUGGESTIONS)
    val found = results.cards.take(OFFERED_CARDS)
    val typed = dock.query.trim()
    val mode = when {
        ai != null -> DockMode.Engine
        dock.typing && typed.isNotEmpty() && (offered.isNotEmpty() || found.isNotEmpty() || dock.engine != null) -> DockMode.Typing
        typed.isNotEmpty() && results.cards.isEmpty() && results.endReached && results.error == null && !results.loading -> DockMode.Nothing
        else -> null
    }
    Column(Modifier.align(Alignment.BottomCenter).fillMaxWidth().aboveFootBar(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        AnimatedContent(
            mode,
            transitionSpec = {
                if (moving) {
                    (fadeIn(tween(FootMotion.FAST, easing = FootMotion.SmoothOut)) + slideInVertically(tween(FootMotion.FAST, easing = FootMotion.SmoothOut)) { it / 6 })
                        .togetherWith(fadeOut(tween(FootMotion.QUICK))) using SizeTransform(clip = false)
                } else {
                    EnterTransition.None togetherWith ExitTransition.None
                }
            },
            contentAlignment = Alignment.BottomCenter,
            label = "search-dock",
        ) { shown ->
            when (shown) {
                DockMode.Typing -> DockCard(padded = false) {
                    offered.forEach { chip ->
                        val tag = chip.type != "type"
                        DockRow({ FootIcon(if (tag) R.drawable.tags else R.drawable.ic_library_grid, tint = LibraryStyle.colors.muted, size = 20.dp) },
                            if (tag) chip.text.lowercase() else chip.text, if (tag) "Tag" else "Kind") { onChoose(chip) }
                    }
                    found.forEach { card ->
                        DockRow({ CardThumbnail(card, Modifier.size(LEAD).clip(RoundedCornerShape(8.dp))) }, cardHeading(card),
                            card.domain?.takeIf { it.isNotBlank() } ?: typeLabel(card)) { onOpenCard(card) }
                    }
                    dock.engine?.let { engine ->
                        DockRow({ EngineMark(engine, working = false) }, "Ask $engine about “$typed”", null, onClick = actions.onAsk)
                    }
                }
                DockMode.Engine -> ai?.let { EngineDock(it, dock, actions, scroll, onAiTag, onOpenSpace, onOpenCard) }
                DockMode.Nothing -> NothingDock(typed, dock.engine, actions)
                null -> Spacer(Modifier.fillMaxWidth())
            }
        }
        filter?.let { FilterPill(it, results.takeIf { r -> r.endReached && r.error == null }?.cards?.size, onRemoveFilter) }
    }
}

private enum class DockMode { Typing, Engine, Nothing }

/** The engine's card: its step on one line, then what it read, its answer and what to do with the ranking. */
@Composable
private fun EngineDock(
    rank: AiRank,
    dock: SearchDockState,
    actions: SearchDockActions,
    scroll: LazyListState,
    onAiTag: (String) -> Unit,
    onOpenSpace: (AiSpace) -> Unit,
    onOpenCard: (Card) -> Unit,
) {
    val colors = LibraryStyle.colors
    val engine = when (rank) {
        is AiRank.Understanding -> rank.engine
        is AiRank.Ranking -> rank.engine
        is AiRank.Ranked -> rank.engine
        is AiRank.Failed -> dock.engine
    }
    val working = rank is AiRank.Understanding || rank is AiRank.Ranking
    var open by remember(rank) { mutableStateOf(true) }
    var more by remember(rank) { mutableStateOf(false) }
    var clipped by remember(rank) { mutableStateOf(false) }
    // Reading the results folds the card down to its first line, out of their way; a tap on that line opens it again.
    LaunchedEffect(scroll, rank) {
        snapshotFlow { scroll.scrolledBy() > FOLD_AFTER_PX }.distinctUntilChanged().filter { it }.collect { open = false; more = false }
    }
    val understood = rank.understood
    val body = (rank as? AiRank.Ranked)?.answer ?: understood?.label?.takeIf { it.isNotBlank() }
    val tags = understood?.tags.orEmpty()
    val spaces = understood?.spaces.orEmpty()
    val sources = (rank as? AiRank.Ranked)?.sources.orEmpty()
    val extras = tags.isNotEmpty() || spaces.isNotEmpty() || sources.isNotEmpty()
    DockCard {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 32.dp)
                .then(if (open) Modifier else Modifier.clickable(role = Role.Button, onClickLabel = "Show") { open = true }),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            if (engine != null) EngineMark(engine, working)
            Text(stepText(aiNote(rank), engine), Modifier.weight(1f).semantics { liveRegion = LiveRegionMode.Polite },
                style = LibraryStyle.body, color = colors.muted, maxLines = if (open) 3 else 1, overflow = TextOverflow.Ellipsis)
            // The folded card opens upward, so its caret points up.
            if (!open) FootIcon(R.drawable.ic_caret_down, Modifier.graphicsLayer { rotationZ = 180f }, tint = colors.muted, size = 18.dp)
        }
        if (!open) return@DockCard
        val bodyScroll = rememberScrollState()
        Column(Modifier.fillMaxWidth().heightIn(max = DOCK_MAX_BODY).scrollEdgeFade(bodyScroll, colors.raised, top = 12.dp, bottom = 12.dp)
            .verticalScroll(bodyScroll)) {
            if (body != null) {
                Text(body, Modifier.padding(top = 8.dp).semantics { liveRegion = LiveRegionMode.Polite }, style = ANSWER, color = colors.text,
                    maxLines = if (more) Int.MAX_VALUE else ANSWER_LINES, overflow = TextOverflow.Ellipsis,
                    onTextLayout = { if (!more) clipped = it.hasVisualOverflow })
            }
            if (working) {
                Column(Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    SkeletonLine(0.9f)
                    SkeletonLine(0.64f)
                }
            }
            if (more && (tags.isNotEmpty() || spaces.isNotEmpty())) {
                ChipFlow(Modifier.fillMaxWidth().padding(top = 12.dp)) {
                    tags.forEach { tag -> SearchChip("#${tag.lowercase()}", false, onRaised = true) { onAiTag(tag) } }
                    spaces.forEach { space -> SearchChip("Space: ${space.name}", false, onRaised = true) { onOpenSpace(space) } }
                }
            }
            if (more && sources.isNotEmpty()) {
                Text("From", Modifier.padding(top = 12.dp, bottom = 8.dp), style = LibraryStyle.body, color = colors.muted)
                ChipFlow(Modifier.fillMaxWidth()) {
                    sources.forEach { card -> SearchChip(sourceLabel(card), false, onRaised = true) { onOpenCard(card) } }
                }
            }
        }
        val ranked = (rank as? AiRank.Ranked)?.count ?: 0
        val canMore = clipped || extras
        if (ranked > 0 || canMore || rank is AiRank.Failed) {
            ChipFlow(Modifier.fillMaxWidth().padding(top = 12.dp)) {
                if (ranked > 0) {
                    DockButton(if (dock.onlyRanked) "Show all matches" else "Only these $ranked", dark = !dock.onlyRanked) {
                        actions.onOnlyRanked(!dock.onlyRanked)
                    }
                }
                if (canMore) DockButton(if (more) "Less" else "Read more", dark = false) { more = !more }
                if (rank is AiRank.Failed && dock.engine != null) DockButton("Try again", dark = true, onClick = actions.onAsk)
            }
        }
    }
}

/** What to try when nothing matched the typed words. */
@Composable
private fun NothingDock(typed: String, engine: String?, actions: SearchDockActions) {
    val colors = LibraryStyle.colors
    val word = loneWord(typed)
    DockCard {
        Text("Nothing matches “$typed” yet", Modifier.semantics { liveRegion = LiveRegionMode.Polite }, style = ROW_TITLE, color = colors.text)
        Text(if (engine != null) "Try fewer words, or let $engine read what you mean." else "Try fewer or different words.",
            Modifier.padding(top = 4.dp), style = LibraryStyle.body, color = colors.muted)
        if (word != null || engine != null) {
            ChipFlow(Modifier.fillMaxWidth().padding(top = 12.dp)) {
                word?.let { DockButton("Search “$it”", dark = true) { actions.onSearchFor(it) } }
                engine?.let { DockButton("Ask $it", dark = word == null, lead = { EngineMark(it, working = false, size = 18.dp) }, onClick = actions.onAsk) }
            }
        }
    }
}

/** The filter the search is limited to, as a raised pill above the bar; a tap takes it off. */
@Composable
private fun FilterPill(filter: SearchFilter, count: Int?, onRemove: () -> Unit) {
    val colors = LibraryStyle.colors
    FootPressTheme {
        Row(
            Modifier.heightIn(min = 44.dp).footShadow(CircleShape, FootShadow.Button).clip(CircleShape).background(colors.raised)
                .semantics(mergeDescendants = true) { contentDescription = "Remove the ${filter.label} filter" }
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onRemove)
                .padding(start = 14.dp, end = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            if (filter.queryText.startsWith("tag:")) FootIcon(R.drawable.tags, tint = colors.muted, size = 18.dp)
            Text(filter.label, style = LibraryStyle.body, color = colors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
            count?.let { Text("$it", style = LibraryStyle.body, color = colors.muted) }
            FootIcon(R.drawable.ic_sheet_close, tint = colors.muted, size = 16.dp)
        }
    }
}

/** The dock's raised card, 24dp round with the menu shadow. */
@Composable
private fun DockCard(padded: Boolean = true, content: @Composable () -> Unit) {
    val shape = RoundedCornerShape(DOCK_CORNER)
    Column(
        Modifier.fillMaxWidth().footShadow(shape, FootShadow.Menu).clip(shape).background(LibraryStyle.colors.raised)
            .then(if (padded) Modifier.padding(horizontal = 16.dp, vertical = 14.dp) else Modifier.padding(vertical = 4.dp)),
    ) { content() }
}

/** One tappable row: a lead mark or picture, a title and an optional detail under it. */
@Composable
private fun DockRow(
    lead: @Composable () -> Unit,
    title: String,
    detail: String?,
    horizontal: Dp = 16.dp,
    minHeight: Dp = 56.dp,
    onClick: () -> Unit,
) {
    val colors = LibraryStyle.colors
    FootPressTheme {
        Row(
            Modifier.fillMaxWidth().heightIn(min = minHeight).clip(RoundedCornerShape(12.dp))
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onClick)
                .padding(horizontal = horizontal, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(Modifier.size(LEAD), contentAlignment = Alignment.Center) { lead() }
            Column(Modifier.weight(1f)) {
                Text(title, style = ROW_TITLE, color = colors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
                detail?.let { Text(it, style = LibraryStyle.body, color = colors.muted, maxLines = 1, overflow = TextOverflow.Ellipsis) }
            }
        }
    }
}

/** A capsule button in the dock: dark for the step most likely next, raised for the rest. */
@Composable
private fun DockButton(label: String, dark: Boolean, lead: (@Composable () -> Unit)? = null, onClick: () -> Unit) {
    val colors = LibraryStyle.colors
    FootPressTheme {
        Row(
            Modifier.heightIn(min = 44.dp).clip(CircleShape).background(if (dark) colors.text else colors.background)
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onClick)
                .padding(horizontal = 16.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            lead?.invoke()
            Text(label, style = BUTTON, color = if (dark) colors.background else colors.text, maxLines = 1)
        }
    }
}

/** A plain text button, as Clear beside Recent. */
@Composable
private fun DockTextButton(label: String, modifier: Modifier, onClick: () -> Unit) {
    FootPressTheme {
        Box(modifier.heightIn(min = 44.dp).clip(CircleShape)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onClick)
            .padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
            Text(label, style = LibraryStyle.body, color = LibraryStyle.colors.muted)
        }
    }
}

/** Claude's mark in its own orange or OpenAI's in the text colour; while [working], a soft ring breathes around it. */
@Composable
private fun EngineMark(engine: String, working: Boolean, size: Dp = 20.dp) {
    val claude = !engine.contains("Codex", ignoreCase = true)
    val tint = if (claude) CLAUDE_ORANGE else LibraryStyle.colors.text
    val moving = rememberFootMotionEnabled()
    val ring = if (working && moving) {
        rememberInfiniteTransition(label = "engine-ring").animateFloat(0.08f, 0.24f,
            infiniteRepeatable(tween(BREATH_MS, easing = FootMotion.SmoothOut), RepeatMode.Reverse), label = "engine-ring-alpha").value
    } else if (working) 0.16f else 0f
    Box(Modifier.size(size + 6.dp).clip(CircleShape).background(tint.copy(alpha = ring)), contentAlignment = Alignment.Center) {
        Image(painterResource(if (claude) R.drawable.ic_si_claude else R.drawable.ic_si_openai), null, Modifier.size(size),
            colorFilter = ColorFilter.tint(tint))
    }
}

/** A grey line standing in for words still to come, breathing while motion is on. */
@Composable
private fun SkeletonLine(width: Float) {
    val moving = rememberFootMotionEnabled()
    val alpha = if (moving) {
        rememberInfiniteTransition(label = "skeleton").animateFloat(0.45f, 0.9f,
            infiniteRepeatable(tween(BREATH_MS, easing = FootMotion.SmoothOut), RepeatMode.Reverse), label = "skeleton-alpha").value
    } else 0.7f
    Box(Modifier.fillMaxWidth(width).height(12.dp).clip(RoundedCornerShape(6.dp)).background(LibraryStyle.colors.text.copy(alpha = alpha * SKELETON_INK)))
}

/** A small muted heading over a group of the opened page. */
@Composable
private fun SectionLabel(text: String, modifier: Modifier = Modifier, start: Dp = 20.dp) {
    Text(text, modifier.padding(start = start, end = 20.dp, top = 16.dp, bottom = 8.dp).semantics { heading() },
        style = LibraryStyle.body.copy(fontWeight = FontWeight.Medium), color = LibraryStyle.colors.muted)
}

/** The step with the engine's name in the text colour and weight, the rest muted. */
@Composable
private fun stepText(note: String, engine: String?) = buildAnnotatedString {
    if (engine != null && note.startsWith(engine)) {
        withStyle(SpanStyle(color = LibraryStyle.colors.text, fontWeight = FontWeight.Medium)) { append(engine) }
        append(note.removePrefix(engine))
    } else {
        append(note)
    }
}

/** A card type's chip name, such as "Articles", for a card without a site. */
private fun typeLabel(card: Card): String = SearchType.entries.firstOrNull { it.type == card.type }?.label ?: card.type

private val CLAUDE_ORANGE = Color(0xFFD97757)
private val DOCK_GAP = 10.dp
private val DOCK_CORNER = 24.dp
private val DOCK_MAX_BODY = 320.dp
private val LEAD = 36.dp
private val RAIL_WIDTH = 120.dp
private val RAIL_HEIGHT = 150.dp
private val ROW_TITLE = LibraryStyle.body.copy(fontSize = 17.sp, lineHeight = 22.sp)
private val ANSWER = LibraryStyle.body.copy(fontSize = 16.sp, lineHeight = 25.sp)
private val BUTTON = LibraryStyle.body.copy(fontWeight = FontWeight.Medium)
private const val ANSWER_LINES = 3
private const val OFFERED_SUGGESTIONS = 2
private const val OFFERED_CARDS = 2
private const val FOLD_AFTER_PX = 240f
private const val BREATH_MS = 900
private const val SKELETON_INK = 0.16f
private const val SOURCE_CHARS = 40

package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.AiSpace
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.theme.LibraryStyle

/**
 * The line under the Search title naming what Claude Code or Codex is doing with a pressed search, and after.
 *
 * @param rank What the engine is doing.
 * @return The words to show.
 */
fun aiNote(rank: AiRank): String = when (rank) {
    is AiRank.Understanding -> "${rank.engine} is reading what you mean…"
    is AiRank.Ranking -> "${rank.engine} is ranking what it found…"
    is AiRank.Ranked -> if (rank.count > 0) "Ranked by ${rank.engine}" else "${rank.engine} found nothing closer to what you mean"
    is AiRank.Failed -> rank.message
}

/**
 * What the engine made of a pressed search, under the Search title: what it is looking for in plain words, what it
 * is doing, chips for the person's own tags and spaces that match, and its answer to a question with the cards the
 * answer draws from.
 *
 * @param rank What the engine is doing.
 * @param onTag Called with a tapped tag's name, to filter the search by it.
 * @param onSpace Called with a tapped space, to open it.
 * @param onOpenCard Called with a tapped source of the answer.
 */
@Composable
fun AiSearchNote(rank: AiRank, onTag: (String) -> Unit, onSpace: (AiSpace) -> Unit, onOpenCard: (Card) -> Unit) {
    val colors = LibraryStyle.colors
    val understood = rank.understood
    Column(Modifier.fillMaxWidth().padding(start = 20.dp, end = 20.dp, bottom = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        understood?.label?.takeIf { it.isNotBlank() }?.let { Text(it, style = LibraryStyle.body, color = colors.text) }
        Text(aiNote(rank), Modifier.semantics { liveRegion = LiveRegionMode.Polite }, style = LibraryStyle.body, color = colors.muted)
        if (rank is AiRank.Ranked && rank.answer != null) AiAnswerBox(rank.answer, rank.sources, onOpenCard)
    }
    val tags = understood?.tags.orEmpty()
    val spaces = understood?.spaces.orEmpty()
    if (tags.isNotEmpty() || spaces.isNotEmpty()) {
        ChipFlow(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 12.dp)) {
            tags.forEach { tag -> SearchChip("#${tag.lowercase()}", false) { onTag(tag) } }
            spaces.forEach { space -> SearchChip("Space: ${space.name}", false) { onSpace(space) } }
        }
    }
}

@Composable
private fun AiAnswerBox(answer: String, sources: List<Card>, onOpenCard: (Card) -> Unit) {
    val colors = LibraryStyle.colors
    Column(Modifier.fillMaxWidth().padding(top = 4.dp).clip(RoundedCornerShape(10.dp)).background(colors.raised).padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(answer, Modifier.semantics { liveRegion = LiveRegionMode.Polite }, style = LibraryStyle.body, color = colors.text)
        if (sources.isNotEmpty()) {
            Text("From", style = LibraryStyle.label, color = colors.muted)
            ChipFlow(Modifier.fillMaxWidth()) {
                sources.forEach { card -> SearchChip(sourceLabel(card), false, onRaised = true) { onOpenCard(card) } }
            }
        }
    }
}

/** A source card's name on its chip: its title, else its site, else its type. */
internal fun sourceLabel(card: Card): String {
    val name = card.title?.trim()?.takeIf { it.isNotEmpty() } ?: card.domain?.takeIf { it.isNotBlank() } ?: card.type
    return if (name.length > SOURCE_CHARS) name.take(SOURCE_CHARS).trimEnd() + "…" else name
}

private const val SOURCE_CHARS = 40

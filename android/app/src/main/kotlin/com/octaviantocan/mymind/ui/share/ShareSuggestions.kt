package com.octaviantocan.mymind.ui.share

import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import java.util.Locale

/** One row under the share sheet's field: what tapping it, or Enter while it is first, adds to what was saved. */
sealed interface ShareSuggestion {
    /**
     * Puts what was saved in [space].
     *
     * @property detail Why it is suggested, or how many cards the space holds.
     */
    data class AddToSpace(val space: Space, val detail: String) : ShareSuggestion

    /**
     * Tags what was saved [tag].
     *
     * @property fresh True for a tag no card has yet.
     * @property recent True for a tag offered because it was added lately, before anything is typed.
     */
    data class AddTag(val tag: String, val fresh: Boolean = false, val recent: Boolean = false) : ShareSuggestion

    /** Makes [text] the note of what was saved. */
    data class UseAsNote(val text: String) : ShareSuggestion
}

/**
 * The rows the share sheet's one field offers (share-field-takes-everything). With nothing typed: the suggested space,
 * then recent tags. With text typed: the spaces whose name holds it, the tags that hold it, a new tag for one word no
 * tag matches, and Use as your note for words with a space between them.
 *
 * @param typed What is in the field.
 * @param spaces The user's spaces, in their order.
 * @param chosenSpaces Ids of the spaces already chosen.
 * @param allTags Every tag of the library.
 * @param recentTags Tags added last, newest first.
 * @param chosenTags Tags already chosen or already on what was saved.
 * @param suggestion The space suggested for what was saved, or null.
 * @return The rows, best first.
 */
fun shareSuggestions(
    typed: String,
    spaces: List<Space>,
    chosenSpaces: Set<String>,
    allTags: List<String>,
    recentTags: List<String>,
    chosenTags: Collection<String>,
    suggestion: SpaceSuggestion?,
): List<ShareSuggestion> {
    val chosen = { tag: String -> chosenTags.any { it.equals(tag, ignoreCase = true) } }
    val query = typed.trim()
    if (query.isEmpty()) {
        val space = suggestion?.takeIf { it.space.id !in chosenSpaces }?.let { ShareSuggestion.AddToSpace(it.space, if (it.alike > 0) "Suggested · ${alike(it.alike)}" else "Suggested by ThyMind") }
        val tags = recentTags.filterNot(chosen).distinctBy { it.lowercase(Locale.ROOT) }.take(RECENT_ROWS).map { ShareSuggestion.AddTag(it, recent = true) }
        return listOfNotNull(space) + tags
    }
    val rows = ArrayList<ShareSuggestion>()
    spaces.filter { it.smart != true && it.id !in chosenSpaces && it.name.orEmpty().contains(query, ignoreCase = true) }
        .take(SPACE_ROWS).mapTo(rows) { ShareSuggestion.AddToSpace(it, cardCount(it)) }
    val word = query.removePrefix("#").trim()
    allTags.filter { it.contains(word, ignoreCase = true) && !chosen(it) }
        .sortedWith(compareBy({ !it.startsWith(word, ignoreCase = true) }, { it.lowercase(Locale.ROOT) }))
        .take(TAG_ROWS).mapTo(rows) { ShareSuggestion.AddTag(it) }
    if (word.isNotEmpty() && word.none(Char::isWhitespace) && allTags.none { it.equals(word, ignoreCase = true) } && !chosen(word)) {
        rows += ShareSuggestion.AddTag(word, fresh = true)
    }
    if (query.any(Char::isWhitespace)) rows += ShareSuggestion.UseAsNote(query)
    return rows
}

/** "1 card like it" or "12 cards like it". */
private fun alike(count: Int) = if (count == 1) "1 card like it" else "$count cards like it"

/** "1 card" or "12 cards"; "Space" when the count is unknown. */
private fun cardCount(space: Space): String = space.count?.let { if (it == 1) "1 card" else "$it cards" } ?: "Space"

private const val RECENT_ROWS = 3
private const val SPACE_ROWS = 2
private const val TAG_ROWS = 3

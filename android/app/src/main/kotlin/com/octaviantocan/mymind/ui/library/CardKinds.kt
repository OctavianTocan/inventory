package com.octaviantocan.mymind.ui.library

import androidx.compose.ui.graphics.Color
import androidx.core.text.HtmlCompat
import com.octaviantocan.mymind.data.Card
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

private val MEDIA_LINK = Regex("""(?:https?://)?(?:pic\.(?:twitter|x)\.com|t\.co)/\S+""")
private val LINE_END_SPACE = Regex("""[ \t\u00a0]+(\n|$)""")

/** How a card is drawn in the grid, following the web front-end's card classes. */
enum class TileKind {
    /** Text the user wrote: notes, highlights, quotes. */
    Note,

    /** A social post with a coloured top edge. */
    Post,

    /** A code repository. */
    Repository,

    /** An article with its headline over the cover. */
    Article,

    /** A picture with the title underneath. */
    Media,

    /** A video: a picture with a play mark and the title underneath. */
    Video,

    /** Anything else: the title on a plain tile. */
    Plain,
}

private const val PINNED_FLAG = 512
private const val READABLE_FLAG = 1 shl 1
internal val REPOSITORY_ACCENT = Color(0xFF2DBA4E)

/** @return How [card] is drawn in the grid. */
fun tileKind(card: Card): TileKind = when (card.type) {
    "Note", "Content", "Quote", "Quotation" -> TileKind.Note
    "XPost", "ThreadsPost", "SubstackNote", "RedditPost", "BlueskyPost", "MastodonPost" -> TileKind.Post
    "Repository" -> TileKind.Repository
    "Article" -> TileKind.Article
    "Video", "YouTubeVideo", "InstagramReel" -> TileKind.Video
    else -> if (card.cover?.src != null || card.blob != null) TileKind.Media else TileKind.Plain
}

/** True when [this] card is pinned to Top of Mind. */
val Card.pinned: Boolean get() = (flags and PINNED_FLAG) != 0 || pinPosition != null

/** True when [this] card is an article the reader view can show. */
val Card.readable: Boolean get() = type == "Article" && (flags and READABLE_FLAG) != 0

/** Title shown for a card: its title, else its type. */
fun cardHeading(card: Card): String = card.title?.takeIf { it.isNotBlank() } ?: card.type

/**
 * @param card A card.
 * @return The text a note tile shows: the note's document as plain paragraphs, else its note or description.
 */
fun noteText(card: Card): String =
    card.prose?.let(::proseText)?.takeIf { it.isNotBlank() }
        ?: card.note?.takeIf { it.isNotBlank() }
        ?: card.description.orEmpty()

/**
 * @param card A social post.
 * @return The post's words without the bare media links the platform appends, such as `pic.twitter.com/…`.
 */
fun postText(card: Card): String =
    (card.post?.bodyHtml?.let { HtmlCompat.fromHtml(it, HtmlCompat.FROM_HTML_MODE_COMPACT).toString() }?.takeIf { it.isNotBlank() }
        ?: card.description.orEmpty()).replace(MEDIA_LINK, "").replace(LINE_END_SPACE, "$1").trim()

/**
 * @param node A stored document tree (`{"type": "doc", "content": [...]}`).
 * @return Its text, one line per paragraph or list item.
 */
fun proseText(node: JsonElement): String {
    val lines = mutableListOf<String>()
    fun walk(element: JsonElement, line: StringBuilder) {
        val obj = element as? JsonObject ?: return
        val type = (obj["type"] as? JsonPrimitive)?.contentOrNull
        (obj["text"] as? JsonPrimitive)?.contentOrNull?.let(line::append)
        if (type == "hardBreak") line.append('\n')
        val children = obj["content"] as? JsonArray ?: return
        if (type == "paragraph" || type == "heading" || type == "codeBlock") {
            val own = StringBuilder()
            children.forEach { walk(it, own) }
            lines += own.toString()
        } else {
            children.forEach { walk(it, line) }
        }
    }
    walk(node, StringBuilder())
    return lines.joinToString("\n").trim()
}

/**
 * @param type Card type of a post.
 * @return Colour of the post tile's top edge (`--card-color` in the web front-end).
 */
fun postColor(type: String): Color = when (type) {
    "XPost" -> Color(0xFF1E9BF0)
    "SubstackNote" -> Color(0xFFFF6719)
    "MastodonPost" -> Color(0xFF6364FF)
    "BlueskyPost" -> Color(0xFF1185FE)
    else -> Color(0xFF626C7A)
}

/**
 * Reuses the colour already shown on a card's tile for its related controls.
 *
 * @param card Card whose contextual sheet is opening.
 * @return Its platform edge, repository edge or picture colour; null for an uncoloured tile.
 */
fun cardAccent(card: Card): Color? = when (tileKind(card)) {
    TileKind.Post -> postColor(card.type)
    TileKind.Repository -> REPOSITORY_ACCENT
    TileKind.Article -> if (card.cover != null) parseHexColor(card.cover.dominantColor ?: card.blob?.dominantColor) else null
    TileKind.Media, TileKind.Video -> parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor)
    else -> null
}

/**
 * @param hex A colour such as `#1a2b3c` or `#f96`.
 * @return The colour, or null when [hex] is not a three- or six-digit hex colour.
 */
fun parseHexColor(hex: String?): Color? {
    val raw = hex?.removePrefix("#") ?: return null
    val digits = when (raw.length) {
        3 -> raw.map { "$it$it" }.joinToString("")
        6 -> raw
        else -> return null
    }
    return digits.toLongOrNull(16)?.let { Color(0xFF000000 or it) }
}

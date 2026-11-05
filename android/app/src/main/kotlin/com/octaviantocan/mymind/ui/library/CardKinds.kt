package com.octaviantocan.mymind.ui.library

import android.util.LruCache
import androidx.annotation.DrawableRes
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.core.text.HtmlCompat
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.PostArticle
import com.octaviantocan.mymind.data.ThreadPost
import com.octaviantocan.mymind.data.RedditComment
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import java.time.Instant
import java.util.Locale

private val MEDIA_LINK = Regex("""(?:https?://)?(?:pic\.(?:twitter|x)\.com|t\.co)/\S+""")
private val LINE_END_SPACE = Regex("""[ \t\u00a0]+(\n|$)""")
private val HTML_TAG = Regex("<[^>]*>")
private val X_STATUS = Regex("""https?://(?:www\.|mobile\.)?(?:x|twitter)\.com/(\w{1,15})/status(?:es)?/(\d+)""")
private val THREADS_USER = Regex("""https?://(?:www\.)?threads\.(?:net|com)/@([\w.]+)""")
private val BLUESKY_USER = Regex("""https?://bsky\.app/profile/([^/?#]+)""")
private val SUBSTACK_USER = Regex("""https?://(?:www\.)?substack\.com/@([\w-]+)""")
private val MASTODON_USER = Regex("""https?://([^/?#]+)/@(\w+)""")
private val NAMED_HANDLE = Regex("""^(.*?)\s*\(@([\w.]+)\)$""")
private val REDDIT_SUMMARY = Regex("""^Posted in r/\S+ by """)
private val PRICE = Regex("""^([A-Za-z]{3})\s+(\S+)$""")
private val CURRENCY_SIGNS = mapOf("GBP" to "£", "USD" to "$", "EUR" to "€", "JPY" to "¥")
private val EXTERNAL_VIDEOS = setOf("YouTubeVideo", "InstagramReel")

/** The first X id that carries its own time (4 November 2010); older ids are plain numbers. */
private const val X_FIRST_TIMED_ID = 29_700_859_247L
private const val X_EPOCH_MILLIS = 1_288_834_974_657L

/** A post with more words than this, or with a picture, stands two squares high: one square holds about four lines. */
private const val POST_SQUARE_CHARS = 72

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

    /** An audio file: a play mark with its length, the title and the start of what is said, on the page surface. */
    Audio,

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
    "Audio" -> TileKind.Audio
    else -> if (card.cover?.src != null || card.blob != null) TileKind.Media else TileKind.Plain
}

/** True when [this] card is pinned. */
val Card.pinned: Boolean get() = (flags and PINNED_FLAG) != 0 || pinPosition != null

/** True when [this] card is an article the reader view can show. */
val Card.readable: Boolean get() = type == "Article" && (flags and READABLE_FLAG) != 0

/** Title shown for a card: its title, else the address of a link the server has not read yet, else its type. */
fun cardHeading(card: Card): String =
    card.title?.takeIf { it.isNotBlank() } ?: card.url?.takeIf { card.type == "Placeholder" && it.isNotBlank() } ?: card.type

/**
 * @param seconds An audio file's length in seconds, fractions included.
 * @return It as `m:ss`, or `h:mm:ss` from an hour; null when the length is missing or not above zero.
 */
fun audioLength(seconds: Double?): String? =
    seconds?.takeIf { it.isFinite() && it > 0.0 }?.let { videoLength((it * 1000).toLong()) }

/**
 * @param card An audio card.
 * @return The words its tile shows under the title: what is said in it, else its summary; empty when it has neither.
 */
fun audioWords(card: Card): String =
    card.transcript?.trim()?.takeIf { it.isNotEmpty() } ?: card.description?.trim().orEmpty()

/**
 * @param card A card.
 * @return The text a note tile shows: the note's document as plain paragraphs, else its note or description.
 */
fun noteText(card: Card): String =
    card.prose?.let(::proseText)?.takeIf { it.isNotBlank() }
        ?: card.note?.takeIf { it.isNotBlank() }
        ?: card.description.orEmpty()

/** Plain text of each post's HTML already read, so a card composed again, or its second layout, parses none. */
private val HTML_TEXT = LruCache<String, String>(512)

/** The plain text of [html], parsed once and then remembered. */
private fun htmlText(html: String): String =
    HTML_TEXT.get(html) ?: HtmlCompat.fromHtml(html, HtmlCompat.FROM_HTML_MODE_COMPACT).toString().also { HTML_TEXT.put(html, it) }

/**
 * @param card A social post.
 * @return The post's words without the bare media links the platform appends, such as `pic.twitter.com/…`.
 */
fun postText(card: Card): String =
    (card.post?.bodyHtml?.let(::htmlText)?.takeIf { it.isNotBlank() }
        ?: card.description.takeIf { postArticle(card) == null }.orEmpty()).replace(MEDIA_LINK, "").replace(LINE_END_SPACE, "$1").trim()

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
 * @return The colour its tile is drawn in ([postBrand]), which its reader and sheets wash with.
 */
fun postColor(type: String): Color = postBrand(type).paper

/**
 * A social network's own colours, in which its posts' tiles and reader sections are drawn.
 *
 * @property paper The network's colour, deepened just enough that white words on it read at 4.5:1.
 * @property text The post's words and the author's name.
 * @property muted The handle, the day and other quiet lines.
 * @property line Hairlines drawn on [paper].
 * @property icon The network's own mark, drawn in [text].
 */
@Immutable
data class PostBrand(val paper: Color, val text: Color, val muted: Color, val line: Color, @param:DrawableRes val icon: Int?)

private val ON_COLOUR = Color.White
private val ON_COLOUR_MUTED = Color.White.copy(alpha = 0.8f)
private val ON_COLOUR_LINE = Color.White.copy(alpha = 0.3f)

/**
 * @param type Card type of a post.
 * @return Its network's colours: X in X's own black and greys, Threads in black, Reddit, Bluesky, Mastodon and
 *   Substack in their colours, and any other post in slate.
 */
fun postBrand(type: String): PostBrand = when (type) {
    "XPost" -> PostBrand(Color.Black, Color(0xFFE7E9EA), Color(0xFF71767B), Color(0xFF2F3336), R.drawable.ic_si_x)
    "ThreadsPost" -> PostBrand(Color.Black, ON_COLOUR, Color(0xFF8E8E8E), Color(0xFF333333), R.drawable.ic_si_threads)
    "RedditPost" -> PostBrand(Color(0xFFD93A00), ON_COLOUR, ON_COLOUR_MUTED, ON_COLOUR_LINE, R.drawable.ic_si_reddit)
    "BlueskyPost" -> PostBrand(Color(0xFF0D71D8), ON_COLOUR, ON_COLOUR_MUTED, ON_COLOUR_LINE, R.drawable.ic_si_bluesky)
    "MastodonPost" -> PostBrand(Color(0xFF5F60F5), ON_COLOUR, ON_COLOUR_MUTED, ON_COLOUR_LINE, R.drawable.ic_si_mastodon)
    "SubstackNote" -> PostBrand(Color(0xFFC85113), ON_COLOUR, ON_COLOUR_MUTED, ON_COLOUR_LINE, R.drawable.ic_si_substack)
    else -> PostBrand(Color(0xFF4A525E), ON_COLOUR, ON_COLOUR_MUTED, ON_COLOUR_LINE, null)
}

/**
 * Who wrote a post, as its tile and reader name them.
 *
 * @property name The author's name, or for Reddit the subreddit (`r/hiking`).
 * @property handle The author's handle (`@theo_maps`, `u/fernhollow`), when it is known and differs from [name].
 */
data class PostByline(val name: String, val handle: String?)

/**
 * @param card A social post.
 * @return Its author's name over the handle read from the post's own link, or the subreddit over `u/author` for
 *   Reddit; null when the card names nobody.
 */
fun postByline(card: Card): PostByline? {
    if (card.type == "RedditPost") {
        val subreddit = card.reddit?.subreddit?.trim()?.takeIf { it.isNotEmpty() }?.let { "r/$it" }
        val user = card.author?.trim()?.removePrefix("u/")?.takeIf { it.isNotEmpty() }?.let { "u/$it" }
        return (subreddit ?: user)?.let { PostByline(it, user.takeIf { subreddit != null }) }
    }
    val written = (card.post?.author ?: card.author)?.trim()?.removePrefix("by ")?.trim()?.takeIf { it.isNotEmpty() }
    val named = written?.let(NAMED_HANDLE::matchEntire)
    val name = named?.groupValues?.get(1)?.takeIf { it.isNotBlank() } ?: written
    val handle = postHandle(card) ?: named?.groupValues?.get(2)?.let { "@$it" }
    return when {
        name != null -> PostByline(name, handle?.takeIf { it != name })
        handle != null -> PostByline(handle, null)
        else -> null
    }
}

/** The author's handle as the post's own link gives it, such as `@theo_maps`, or null. */
private fun postHandle(card: Card): String? {
    val url = card.url ?: return null
    return when (card.type) {
        "XPost" -> X_STATUS.find(url)?.groupValues?.get(1)?.takeIf { it != "i" }?.let { "@$it" }
        "ThreadsPost" -> THREADS_USER.find(url)?.groupValues?.get(1)?.let { "@$it" }
        "BlueskyPost" -> BLUESKY_USER.find(url)?.groupValues?.get(1)?.takeIf { !it.startsWith("did:") }?.let { "@$it" }
        "SubstackNote" -> SUBSTACK_USER.find(url)?.groupValues?.get(1)?.let { "@$it" }
        "MastodonPost" -> MASTODON_USER.find(url)?.let { "@${it.groupValues[2]}@${it.groupValues[1]}" }
        else -> null
    }
}

/**
 * @param card A social post.
 * @return When it was posted, read from an X post's id (X's ids carry their own time); null for other posts and for X
 *   posts older than November 2010.
 */
fun postedAt(card: Card): Instant? {
    if (card.type != "XPost") return null
    val id = card.url?.let(X_STATUS::find)?.groupValues?.get(2)?.toLongOrNull()?.takeIf { it >= X_FIRST_TIMED_ID } ?: return null
    return Instant.ofEpochMilli((id shr 22) + X_EPOCH_MILLIS)
}

/**
 * @param card A card.
 * @return The posts of the X thread [card] ends, oldest first and [card]'s own post last, when the server kept one;
 *   empty for a single post and every other card.
 */
fun postThread(card: Card): List<ThreadPost> =
    if (card.type == "XPost") card.post?.thread.orEmpty().takeIf { it.size > 1 }.orEmpty() else emptyList()

/**
 * @param post A post of an X thread.
 * @return Its words as plain text, without the bare media links X appends.
 */
fun threadPostWords(post: ThreadPost): String =
    post.bodyHtml?.let(::htmlText).orEmpty()
        .replace(MEDIA_LINK, "").replace(LINE_END_SPACE, "$1").trim()

/**
 * @param card An X thread ([postThread]).
 * @return The name the model gave the whole thread, which the server sends with its summary; null until the thread has
 *   been summed up, when its tile shows the first post's words instead.
 */
fun threadName(card: Card): String? = card.title?.trim()?.takeIf { it.isNotEmpty() && !card.summary.isNullOrBlank() }

/**
 * @param card A social post.
 * @return The words its tile and reader show: the post's own words, or a Reddit post's description unless it is only
 *   the page's "Posted in r/… by …" line.
 */
fun postWords(card: Card): String {
    val words = postText(card)
    return if (card.type == "RedditPost" && REDDIT_SUMMARY.containsMatchIn(words)) "" else words
}

/**
 * @param card A card.
 * @return The X article an X post carries, when the server read one with a title; null otherwise.
 */
fun postArticle(card: Card): PostArticle? = card.article?.takeIf { card.type == "XPost" && !it.title.isNullOrBlank() }

/**
 * @param card A card.
 * @return The top comments kept with a Reddit post when it was saved, each with its author and words; empty for other
 *   cards.
 */
fun redditComments(card: Card): List<RedditComment> = if (card.type != "RedditPost") emptyList()
    else card.reddit?.comments.orEmpty().filter { !it.author.isNullOrBlank() && !it.body.isNullOrBlank() }

/**
 * @param card A card.
 * @return True for a social post with a picture or an X article, or with more words than one square holds, which
 *   stands two squares high. It reads the stored words without parsing them, as the layout asks for every card.
 */
internal fun postStandsTall(card: Card): Boolean {
    if (tileKind(card) != TileKind.Post) return false
    if (galleryPicture(card) != null || postArticle(card) != null || postThread(card).isNotEmpty()) return true
    val words = card.post?.bodyHtml?.replace(HTML_TAG, "") ?: card.description.orEmpty()
    val title = if (card.type == "RedditPost") card.reddit?.title.orEmpty() else ""
    return words.replace(MEDIA_LINK, "").trim().length + title.length > POST_SQUARE_CHARS
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

/** True for a PDF with a picture of its first page, which stands two squares high with the page whole. */
internal fun pdfStandsTall(card: Card): Boolean = card.type == "Document" && galleryPicture(card) != null

/** True for a product with a photo, which stands two squares high as a pin. */
internal fun productStandsTall(card: Card): Boolean = card.type == "Product" && galleryPicture(card) != null

/**
 * @param card A product.
 * @return Its price with the currency's sign, such as "£24" for the shop's "GBP 24.00" or "CHF 30" for a currency
 *   without one here; null when the shop gave none.
 */
fun productPrice(card: Card): String? {
    val raw = card.product?.price?.trim()?.takeIf { it.isNotEmpty() } ?: return null
    val parts = PRICE.matchEntire(raw)
    val code = parts?.groupValues?.get(1)?.uppercase()
    val amount = parts?.groupValues?.get(2) ?: raw
    val number = amount.toDoubleOrNull()
    val shown = when {
        number == null -> amount
        number % 1.0 == 0.0 -> number.toLong().toString()
        else -> String.format(Locale.UK, "%.2f", number)
    }
    if (code == null) return shown
    return CURRENCY_SIGNS[code]?.let { it + shown } ?: "$code $shown"
}

/**
 * @param card A product.
 * @return Its brand, else the shop's name or address; null when none is known.
 */
fun productShop(card: Card): String? = listOf(card.product?.brand, card.siteName, card.domain).firstOrNull { !it.isNullOrBlank() }?.trim()

/**
 * The store an app is on.
 *
 * @property name Such as "Google Play".
 * @property icon The store's mark.
 */
data class AppStore(val name: String, @param:DrawableRes val icon: Int?)

/**
 * @param card An app.
 * @return The store it is on, from the store the capture found, else its address; null when neither is known.
 */
fun appStore(card: Card): AppStore? = when (card.app?.distributor) {
    "play-store" -> AppStore("Google Play", R.drawable.ic_si_googleplay)
    "app-store" -> AppStore("App Store", R.drawable.ic_si_appstore)
    else -> card.domain?.takeIf { it.isNotBlank() }?.let { AppStore(it, null) }
}

/**
 * The words a video from another site shows under its picture.
 *
 * @property title The video's title.
 * @property channel Who posted it, when known.
 */
data class VideoCaption(val title: String, val channel: String?)

/**
 * @param card A card.
 * @return The title and channel a YouTube or Instagram video with a picture shows under it; null for other cards and
 *   for a video with a note of its own, which keeps the note beside its picture instead.
 */
fun videoCaption(card: Card): VideoCaption? {
    if (card.type !in EXTERNAL_VIDEOS || galleryPicture(card) == null || !card.note.isNullOrBlank()) return null
    val title = card.title?.trim()?.takeIf { it.isNotEmpty() } ?: return null
    return VideoCaption(title, card.author?.trim()?.removePrefix("by ")?.trim()?.takeIf { it.isNotEmpty() })
}

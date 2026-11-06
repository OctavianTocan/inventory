package com.octaviantocan.mymind.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject

/** JSON configuration shared by every request and response. */
val AppJson = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
}

@Serializable
data class Tag(val id: String? = null, val name: String? = null, val flags: Int? = null)

@Serializable
data class Cover(
    val src: String? = null,
    val width: Int? = null,
    val height: Int? = null,
    val type: String? = null,
    val dominantColor: String? = null,
    val thumbHash: String? = null,
)

@Serializable
data class Blob(
    val path: String? = null,
    val type: String? = null,
    val width: Int? = null,
    val height: Int? = null,
    val dominantColor: String? = null,
    val thumbHash: String? = null,
    /** An audio or video file's length in seconds, as the server read it; null when it did not. */
    val duration: Double? = null,
)

@Serializable
data class GalleryImage(val src: String? = null, val width: Int? = null, val height: Int? = null)

/**
 * A social post's saved copy.
 *
 * @property thread For an X post that ends a thread its author wrote, every post of the thread kept when it was saved,
 *   oldest first, this post last; null for a single post.
 */
@Serializable
data class Post(
    val bodyHtml: String? = null,
    val author: String? = null,
    val hasVideo: Boolean? = null,
    val gallery: List<GalleryImage>? = null,
    val thread: List<ThreadPost>? = null,
)

/** One post of an X thread as it was saved: its words as HTML, whether it held a video, and its pictures. */
@Serializable
data class ThreadPost(val bodyHtml: String? = null, val hasVideo: Boolean? = null, val gallery: List<GalleryImage>? = null)

@Serializable
data class AppInfo(
    val distributor: String? = null,
    val srcset: String? = null,
    val width: Int? = null,
    val height: Int? = null,
)

@Serializable
data class Product(val price: String? = null, val brand: String? = null)

/** One of the top comments kept with a Reddit post when it was saved. */
@Serializable
data class RedditComment(val author: String? = null, val body: String? = null)

@Serializable
data class RedditInfo(val title: String? = null, val subreddit: String? = null, val comments: List<RedditComment>? = null)

/** The X article a post carries: its title, its opening lines and its address on X. */
@Serializable
data class PostArticle(val title: String? = null, val preview: String? = null, val url: String? = null)

/** A library card. [type] is kept as a string so card types this app does not know still decode. */
@Serializable
data class Card(
    val id: String,
    val type: String,
    val flags: Int,
    val created: String,
    val modified: String,
    val bumped: String,
    val deleted: String? = null,
    val title: String? = null,
    val description: String? = null,
    /** The model's summary, which the server sends only for an X thread. */
    val summary: String? = null,
    val url: String? = null,
    val domain: String? = null,
    val siteName: String? = null,
    val author: String? = null,
    val note: String? = null,
    val tags: List<Tag>? = null,
    val spaces: List<String>? = null,
    /** The card's place in each of its [spaces], by space id: a space lists its cards from the smallest. */
    val spaceRanks: Map<String, Double>? = null,
    /** The id of the space ThyMind's AI would file this card in, when it picked one. */
    val suggestedSpace: String? = null,
    /** For a repeat save ThyMind folded into an earlier card and put in the trash, that earlier card's id. */
    val mergedInto: String? = null,
    val pinPosition: Long? = null,
    val cover: Cover? = null,
    val blob: Blob? = null,
    val prose: JsonElement? = null,
    val post: Post? = null,
    val app: AppInfo? = null,
    val product: Product? = null,
    val reddit: RedditInfo? = null,
    /** A video's length in seconds, as its site gives it, for videos the server does not keep. */
    val videoLength: Int? = null,
    /** What is said in a video saved as a file, as the server transcribed it; null until then or when nobody speaks. */
    val transcript: String? = null,
    val article: PostArticle? = null,
    val state: String? = null,
)

@Serializable
data class CardPage(val cards: List<Card> = emptyList(), val next: String? = null)

/**
 * One answer of the server's changes feed.
 *
 * @property cards Cards changed since the request's position, oldest change first.
 * @property removed Ids of cards trashed or deleted since.
 * @property next Position to ask from next time.
 * @property more True while further answers follow.
 * @property reset True when the server's log started again: the phone drops its copy before keeping these cards.
 * @property spaces The spaces, on the last answer only.
 */
@Serializable
data class ChangesPage(
    val cards: List<Card> = emptyList(),
    val removed: List<String> = emptyList(),
    val next: String,
    val more: Boolean = false,
    val reset: Boolean = false,
    val spaces: List<Space>? = null,
)

/**
 * One change made on the phone, as the server's push takes it.
 *
 * @property id The change's place in the phone's queue; the server applies each id once.
 * @property name What the change does, such as `pin`.
 * @property args The change's fields, such as `cardId`.
 */
@Serializable
data class Change(val id: Long, val name: String, val args: JsonObject)

/**
 * A push of the phone's waiting changes.
 *
 * @property clientId The id the phone's copy goes by with the server.
 * @property changes The changes, oldest first.
 */
@Serializable
data class PushRequest(val clientId: String, val changes: List<Change>)

/**
 * The server's answer to a push.
 *
 * @property lastChangeId The highest change id the server has handled for the phone, applied or turned down; null
 *   from an older server, which answers `{}` to a push it does not know and applies nothing.
 * @property refused Ids of the changes the server turned down.
 * @property tagIds The server's id of each tag the push added, by change id.
 * @property cardIds The id the server keeps for each card the push saved, by change id: the phone's own, or that of a
 *   link already saved.
 */
@Serializable
data class PushAnswer(
    val lastChangeId: Long? = null,
    val refused: List<Long> = emptyList(),
    val tagIds: Map<String, String> = emptyMap(),
    val cardIds: Map<String, String> = emptyMap(),
)

@Serializable
data class Space(
    val id: String,
    val name: String? = null,
    val color: String? = null,
    val position: Long? = null,
    val smart: Boolean? = null,
    val count: Int? = null,
)

@Serializable
data class SpaceList(val spaces: List<Space> = emptyList())

@Serializable
data class TagCount(val name: String, val count: Double? = null)

@Serializable
data class TagList(val tags: List<TagCount> = emptyList())

/**
 * The signed-in account.
 *
 * @property email Email address.
 * @property kind How the account signs in: `google` or `password`.
 * @property colorScheme Appearance the account picked for library pages: `light`, `dark` or `system`; `/app/v1/me`
 *   does not send it, so the app reads it from [PageConfig].
 * @property photoUrl Google credential photo cached for this accepted session, or null.
 * @property displayName Google credential name cached for this accepted session, or null.
 */
@Serializable
data class User(
    val email: String? = null,
    val kind: String? = null,
    val colorScheme: String? = null,
    val photoUrl: String? = null,
    val displayName: String? = null,
)

/**
 * An AI app connected to the account through ThyMind's MCP server (`GET /app/v1/agents`).
 *
 * @property id The server's id for its sign-in, used to remove it.
 * @property name The name the AI app gave itself, such as `Claude`.
 * @property connected When it connected, as an ISO time.
 * @property used When it last reached the library, as an ISO time.
 */
@Serializable
data class ConnectedAgent(val id: String, val name: String, val connected: String, val used: String)

/** @property agents The account's connected AI apps, most recently used first. */
@Serializable
data class AgentList(val agents: List<ConnectedAgent> = emptyList())

/**
 * Account settings and feature switches from the config script of a web page.
 *
 * @property colorScheme Appearance the account picked: `light`, `dark` or `system`, or null when the page has none.
 * @property features Features the plan unlocks, such as `reader` or `sameVibe`.
 * @property isGuest True for a guest session.
 */
@Serializable
data class PageConfig(
    val colorScheme: String? = null,
    val features: Set<String> = emptySet(),
    val isGuest: Boolean = false,
)

private val CONFIG_SCRIPT = Regex("""<script id="config" type="application/json">(.*?)</script>""", RegexOption.DOT_MATCHES_ALL)

/**
 * Reads the config script out of a web page.
 *
 * @param html The page.
 * @return The config, or an empty one when the page has no config script.
 */
fun pageConfigOf(html: String): PageConfig =
    CONFIG_SCRIPT.find(html)?.let { AppJson.decodeFromString(PageConfig.serializer(), it.groupValues[1]) } ?: PageConfig()

/**
 * A search suggestion.
 *
 * @property type `type` for a card type or `tag` for a tag.
 * @property text What the suggestion shows.
 * @property queryText The filter it stands for, such as `type:article` or `tag:recipes`.
 */
@Serializable
data class Suggestion(val type: String, val text: String, val queryText: String)

/**
 * A tag as `/tags/suggest` and `/tags/recent` send it.
 *
 * @property name Tag name.
 */
@Serializable
data class TagName(val name: String)

/**
 * Body of a tag write.
 *
 * @property name Tag name.
 */
@Serializable
data class NameBody(val name: String)

/**
 * Body of a space creation.
 *
 * @property name Space name.
 * @property color Space colour as a CSS hex colour.
 */
@Serializable
data class NewSpaceBody(val name: String, val color: String)

/**
 * Body of `PATCH /spaces/<id>` that renames a space and keeps its colour.
 *
 * @property name The space's new name.
 */
@Serializable
data class RenameSpaceBody(val name: String)

/**
 * An answer that names what it created.
 *
 * @property id Id of the new object.
 */
@Serializable
data class IdAnswer(val id: String)

@Serializable
data class TokenAnswer(val token: String, val expiresAt: String? = null, val user: User? = null)

@Serializable
data class NonceAnswer(val nonce: String, val expiresAt: String? = null)

@Serializable
data class ErrorAnswer(val error: String? = null)

@Serializable
data class GoogleSignInRequest(val idToken: String, val nonce: String, val device: String)

@Serializable
data class PasswordSignInRequest(val username: String, val password: String, val device: String)

/**
 * Permission to upload one file, from `POST /uploads/authorize`.
 *
 * @property url Path the file's pieces are sent to, such as `/uploads/<id>`.
 * @property token Upload token for the `x-upload-token` header.
 */
@Serializable
data class UploadAuthorization(val url: String, val token: String)

/**
 * Answer to one uploaded piece of a file.
 *
 * @property id Upload id, sent as `x-upload-id` with the later pieces.
 * @property blob The stored file once complete; its id is the `uploadId` a Media card is made from.
 * @property received Bytes the server holds so far.
 * @property done True after the last piece.
 */
@Serializable
data class UploadChunkAnswer(val id: String, val blob: UploadBlob? = null, val received: Double? = null, val done: Boolean = false)

/**
 * The stored file of an upload.
 *
 * @property id Id `POST /objects` takes as `uploadId`.
 */
@Serializable
data class UploadBlob(val id: String)

/**
 * Body of `POST /objects` for an uploaded file.
 *
 * @property type Always `Media`.
 * @property uploadId Blob id of the finished upload.
 * @property spaceId Space the new card joins, or null.
 */
@Serializable
data class MediaBody(val type: String, val uploadId: String, val spaceId: String? = null)

/**
 * Body of a card created from a link or text.
 *
 * @property content The link or the text.
 * @property source Where the card came from.
 * @property spaceId Space the new card joins, or null.
 */
@Serializable
data class ContentBody(val content: String, val source: String, val spaceId: String? = null)

/**
 * Back today and the sky, as `GET /app/v1/sky` answers for one day.
 *
 * @property day The phone's date the answer is for, `YYYY-MM-DD`.
 * @property back Today's old finds by id, at most three, the same all day.
 * @property active The constellation the finds light now, or null.
 * @property done Finished constellations, oldest first.
 * @property stars Every find opened from Back today or the sky.
 */
@Serializable
data class Sky(
    val day: String = "",
    val back: List<String> = emptyList(),
    val active: Constellation? = null,
    val done: List<Constellation> = emptyList(),
    val stars: List<SkyStar> = emptyList(),
)

/**
 * A constellation: a recent save and the old finds nearest to it.
 *
 * @property id The server's id.
 * @property figure Which figure it is drawn as, such as `ship`.
 * @property name Its name, such as "The Voyager".
 * @property seed The recent save it formed around, lit from the start.
 * @property cards The seed, then its finds.
 * @property lit The seed and the finds opened since it formed.
 * @property completed When its last star lit, or null while it is active.
 */
@Serializable
data class Constellation(
    val id: Long = 0,
    val figure: String = "",
    val name: String = "",
    val seed: String = "",
    val cards: List<String> = emptyList(),
    val lit: List<String> = emptyList(),
    val created: String? = null,
    val completed: String? = null,
)

/**
 * @property id A find opened from Back today or the sky.
 * @property visited When it was last opened.
 */
@Serializable
data class SkyStar(val id: String, val visited: String? = null)

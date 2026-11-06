package com.octaviantocan.mymind.ui.moment

import androidx.compose.ui.graphics.Color
import com.octaviantocan.mymind.data.Card
import java.net.URI
import java.util.Locale

/**
 * How rare a find is in the user's own library (find-rarity-from-library): its colour, name, stars and points, and the
 * colours the shooting star passes through before it lands.
 */
enum class Rarity(val label: String, val stars: Int, val points: Int, val color: Color, val tease: List<Color>) {
    Common("Common", 1, 10, Color(0xFFA39E92), listOf(StarWhite)),
    Rare("Rare", 3, 40, Color(0xFF3E8DB5), listOf(StarWhite, Color(0xFF3E8DB5))),
    Legendary("Legendary", 5, 80, Color(0xFFE2A23B), listOf(StarWhite, Color(0xFF3E8DB5), Color(0xFFE2A23B))),
}

internal val StarWhite = Color(0xFFFFFFFF)

/** A source holding fewer than this many earlier cards makes a Rare find. */
const val RARE_BELOW = 5

/**
 * @param earlier How many cards from the same source the library already held.
 * @return Legendary for the first find from a source, Rare while the source is still small, else Common.
 */
fun rarityOf(earlier: Int): Rarity = when {
    earlier <= 0 -> Rarity.Legendary
    earlier < RARE_BELOW -> Rarity.Rare
    else -> Rarity.Common
}

/**
 * @param card A card from the library.
 * @return Where it comes from, for counting finds: its site for a link, else its kind; null for one not read yet.
 */
fun sourceOf(card: Card): String? {
    val host = card.domain?.takeIf { it.isNotBlank() } ?: card.url?.let(::hostOf)
    return host?.let(::siteKey) ?: card.type.takeIf { it != "Placeholder" && it.isNotBlank() }?.let { "kind:$it" }
}

/**
 * @param link The link shared, or null for words or a file.
 * @param mime The shared file's type, or null.
 * @return The source of something shared, the same way [sourceOf] names a saved card's.
 */
fun sourceOfShare(link: String?, mime: String?): String = when {
    link != null -> hostOf(link)?.let(::siteKey) ?: "kind:Note"
    mime == null -> "kind:Note"
    mime.startsWith("image/") -> "kind:Image"
    mime.startsWith("video/") -> "kind:Video"
    mime.startsWith("audio/") -> "kind:Audio"
    mime == "application/pdf" -> "kind:PDF"
    else -> "kind:Document"
}

/**
 * @param cards The library as the phone keeps it.
 * @param source The source of the find.
 * @param leaveOut Ids of the cards this share just saved, which are not earlier finds.
 * @return How rare the find is among [cards].
 */
fun rarityIn(cards: List<Card>, source: String, leaveOut: Set<String> = emptySet()): Rarity =
    rarityOf(cards.count { it.id !in leaveOut && sourceOf(it) == source })

/** One site under all its names: no www or mobile prefix, and the old names of X and YouTube's short links. */
internal fun siteKey(host: String): String {
    val bare = host.lowercase(Locale.ROOT).removePrefix("www.").removePrefix("m.").removePrefix("mobile.")
    return "site:" + (SITE_ALIASES[bare] ?: bare)
}

private fun hostOf(link: String): String? = runCatching { URI(link.trim()).host }.getOrNull()?.takeIf { it.isNotBlank() }

private val SITE_ALIASES = mapOf("twitter.com" to "x.com", "youtu.be" to "youtube.com", "music.youtube.com" to "youtube.com")

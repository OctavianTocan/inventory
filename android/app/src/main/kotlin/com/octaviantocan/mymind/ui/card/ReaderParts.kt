package com.octaviantocan.mymind.ui.card

import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.noteText
import com.octaviantocan.mymind.ui.library.postText
import com.octaviantocan.mymind.ui.library.proseText
import com.octaviantocan.mymind.ui.library.tileKind
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlin.math.ceil

/** The parts of a card the reader shows as its own sections, under the picture and title. */
enum class ReaderPart(val heading: String) {
    /** A social post's own words. */
    Post("Post"),

    /** The words of a note, highlight or quote. */
    Note("Note"),

    /** The description saved with a link. */
    Summary("Summary"),

    /** The note the user wrote on a card that is not itself a note. */
    YourNote("Your note"),

    /** The text captured from the page. */
    Page("From the page"),
}

/**
 * One section of the reader's text.
 *
 * @property part Which part of the card it is.
 * @property text Its words, trimmed.
 */
data class ReaderText(val part: ReaderPart, val text: String)

/**
 * @param card A card.
 * @return The reader's text sections, in order: the post or note itself, the saved summary, the user's own note and the
 *   text from the page, without blanks or repeats. The reader shows the tags between the summary and the note.
 */
fun readerSections(card: Card): List<ReaderText> {
    val prose = card.prose?.let(::proseText)
    val parts = when (tileKind(card)) {
        TileKind.Note -> listOf(ReaderPart.Note to noteText(card), ReaderPart.YourNote to card.note)
        TileKind.Post -> listOf(ReaderPart.Post to postText(card), ReaderPart.YourNote to card.note, ReaderPart.Page to prose)
        else -> listOf(ReaderPart.Summary to card.description, ReaderPart.YourNote to card.note, ReaderPart.Page to prose)
    }
    val seen = mutableSetOf<String>()
    return parts.mapNotNull { (part, text) -> text?.trim()?.takeIf { it.isNotEmpty() && seen.add(it) }?.let { ReaderText(part, it) } }
}

/**
 * @param text Words to read.
 * @return Minutes they take at 200 words a minute, rounded up; 0 for no words.
 */
fun readingMinutes(text: String): Int {
    val words = text.split(Regex("\\s+")).count { it.isNotEmpty() }
    return if (words == 0) 0 else ceil(words / WORDS_PER_MINUTE).toInt()
}

/**
 * @param card A card.
 * @return How long its page text takes to read, such as "12 min read", or null when it has no page text.
 */
fun readingTime(card: Card): String? {
    val page = readerSections(card).firstOrNull { it.part == ReaderPart.Page } ?: return null
    return "${readingMinutes(page.text)} min read"
}

/**
 * @param created When the card was saved, as the server writes it (ISO 8601).
 * @param today The day it is now, in the phone's time zone.
 * @param zone The phone's time zone.
 * @return Such as "Saved 24 Oct", with the year when it is not this year; null when [created] can't be read.
 */
fun savedLabel(created: String, today: LocalDate = LocalDate.now(), zone: ZoneId = ZoneId.systemDefault()): String? {
    val day = runCatching { Instant.parse(created).atZone(zone).toLocalDate() }.getOrNull() ?: return null
    val pattern = if (day.year == today.year) "d MMM" else "d MMM yyyy"
    return "Saved " + day.format(DateTimeFormatter.ofPattern(pattern, Locale.UK))
}

/**
 * @param card A card.
 * @return Who made it and where it is from, such as "Henry David Thoreau · The Atlantic", or null when neither is known.
 */
fun bylineOf(card: Card): String? =
    listOfNotNull(card.author?.trim()?.takeIf { it.isNotEmpty() }, siteOf(card)).distinct().joinToString(" · ").ifEmpty { null }

/** Where the card came from: its site, else its domain, else the link's host; null for a card without a link. */
internal fun siteOf(card: Card): String? = card.siteName?.takeIf { it.isNotBlank() }
    ?: card.domain?.takeIf { it.isNotBlank() }
    ?: card.url?.let { runCatching { java.net.URI(it).host }.getOrNull() }?.removePrefix("www.")

/** The name the Open source button speaks for a card with a link. */
internal fun sourceName(card: Card): String = siteOf(card) ?: "Source"

/**
 * @param card A card.
 * @return Its link when it is a web address the reader can open, else null.
 */
internal fun sourceUrl(card: Card): String? = card.url?.takeIf { it.startsWith("https://") || it.startsWith("http://") }

private const val WORDS_PER_MINUTE = 200.0

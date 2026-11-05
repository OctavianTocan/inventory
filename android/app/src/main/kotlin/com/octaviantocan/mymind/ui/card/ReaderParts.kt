package com.octaviantocan.mymind.ui.card

import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.noteText
import com.octaviantocan.mymind.ui.library.postThread
import com.octaviantocan.mymind.ui.library.postWords
import com.octaviantocan.mymind.ui.library.threadPostWords
import com.octaviantocan.mymind.ui.library.postedAt
import com.octaviantocan.mymind.ui.library.proseText
import com.octaviantocan.mymind.ui.library.tileKind
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlin.math.ceil

/** The parts of a card the reader shows as its own sections; [readerSections] gives the ones made of words. */
enum class ReaderPart(val heading: String) {
    /** The card's picture, over its title. */
    Picture("Picture"),

    /** A social post's own words. */
    Post("Post"),

    /** The words of a note, highlight or quote. */
    Note("Note"),

    /** The description saved with a link. */
    Summary("Summary"),

    /** What is said in a saved video, as the server transcribed it. */
    Transcript("Transcript"),

    /** The note the user wrote on a card that is not itself a note. */
    YourNote("Your note"),

    /** The text captured from the page. */
    Page("From the page"),

    /** The card's tags. */
    Tags("Tags"),
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
 * @return The reader's text sections, in order: an X thread's summary, the post or note itself (every post of a thread),
 *   the saved summary, a video's transcript, the user's own note and the
 *   text from the page, without blanks or repeats. The reader shows the tags between the summary and the note.
 */
fun readerSections(card: Card): List<ReaderText> {
    val prose = card.prose?.let(::proseText)
    val parts = when (tileKind(card)) {
        TileKind.Note -> listOf(ReaderPart.Note to noteText(card), ReaderPart.YourNote to card.note)
        TileKind.Post -> postThread(card).takeIf { it.isNotEmpty() }?.let { thread ->
            listOf(ReaderPart.Summary to card.summary, ReaderPart.Post to thread.map(::threadPostWords).filter { it.isNotEmpty() }.joinToString("\n\n"),
                ReaderPart.YourNote to card.note, ReaderPart.Page to prose)
        } ?: listOf(ReaderPart.Post to postWords(card), ReaderPart.YourNote to card.note, ReaderPart.Page to prose)
        else -> listOf(ReaderPart.Summary to card.description, ReaderPart.Transcript to card.transcript, ReaderPart.YourNote to card.note,
            ReaderPart.Page to prose)
    }
    val seen = mutableSetOf<String>()
    return parts.mapNotNull { (part, text) -> text?.trim()?.takeIf { it.isNotEmpty() && seen.add(it) }?.let { ReaderText(part, it) } }
}

/** What a row of a part's sheet does. */
enum class PartAction {
    /** Puts the part's words on the clipboard. */
    Copy,

    /** Hands the part's words to another app. */
    Share,

    /** Opens Add tags. */
    EditTags,

    /** Opens the user's note to write or change it. */
    EditNote,

    /** Opens Reader Mode. */
    Read,

    /** Saves the card's picture file to Downloads. */
    SavePicture,

    /** Puts the card's picture on the clipboard. */
    CopyPicture,

    /** Hands the card's picture to another app. */
    SharePicture,
}

/**
 * One row of a part's sheet.
 *
 * @property action What it does.
 * @property label What it says.
 */
data class PartChoice(val action: PartAction, val label: String)

/**
 * The few things holding a part of the reader offers, in order.
 *
 * @param part The part held.
 * @param card The card it belongs to.
 * @return Its sheet's rows; none for a part the card does not have.
 */
fun partChoices(part: ReaderPart, card: Card): List<PartChoice> {
    val words = partText(part, card)
    return when (part) {
        ReaderPart.Post, ReaderPart.Note, ReaderPart.Summary, ReaderPart.Transcript -> {
            if (words == null) return emptyList()
            val name = part.heading.lowercase()
            listOf(PartChoice(PartAction.Copy, "Copy $name"), PartChoice(PartAction.Share, "Share $name"))
        }
        ReaderPart.YourNote -> if (words == null) listOf(PartChoice(PartAction.EditNote, "Write a note"))
            else listOf(PartChoice(PartAction.EditNote, "Edit note"), PartChoice(PartAction.Copy, "Copy note"))
        ReaderPart.Page -> if (words == null) emptyList()
            else listOf(PartChoice(PartAction.Read, "Read in Reader Mode"), PartChoice(PartAction.Copy, "Copy text"))
        ReaderPart.Tags -> listOfNotNull(PartChoice(PartAction.EditTags, "Edit tags"), words?.let { PartChoice(PartAction.Copy, "Copy tags") })
        ReaderPart.Picture -> if (galleryPicture(card) == null) emptyList() else listOfNotNull(
            PartChoice(PartAction.SavePicture, "Save picture").takeIf { card.blob?.path != null },
            PartChoice(PartAction.CopyPicture, "Copy picture"),
            PartChoice(PartAction.SharePicture, "Share picture"),
        )
    }
}

/**
 * @param part A part of the reader.
 * @param card The card it belongs to.
 * @return The part's words as Copy and Share hand them over (tags joined by commas), or null when it has none.
 */
fun partText(part: ReaderPart, card: Card): String? = when (part) {
    ReaderPart.Tags -> card.tags.orEmpty().mapNotNull { it.name?.trim()?.takeIf(String::isNotEmpty) }.joinToString(", ").ifEmpty { null }
    ReaderPart.YourNote -> card.note?.trim()?.takeIf { it.isNotEmpty() }
    ReaderPart.Picture -> null
    else -> readerSections(card).firstOrNull { it.part == part }?.text
}

/**
 * @param card A card.
 * @return The page's text as Reader Mode lays it out, one paragraph per line of the capture; empty without page text.
 */
fun readerParagraphs(card: Card): List<String> =
    partText(ReaderPart.Page, card)?.lines()?.map { it.trim() }?.filter { it.isNotEmpty() }.orEmpty()

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
 * @param card A social post.
 * @param today The day it is now, in the phone's time zone.
 * @param zone The phone's time zone.
 * @return The day it was posted, such as "14 Oct" (with the year when it is not this year), for an X post; null when
 *   the post does not say.
 */
fun postDay(card: Card, today: LocalDate = LocalDate.now(), zone: ZoneId = ZoneId.systemDefault()): String? {
    val day = postedAt(card)?.atZone(zone)?.toLocalDate() ?: return null
    return day.format(DateTimeFormatter.ofPattern(if (day.year == today.year) "d MMM" else "d MMM yyyy", Locale.UK))
}

/**
 * @param card A social post.
 * @param today The day it is now, in the phone's time zone.
 * @param zone The phone's time zone.
 * @return When it was posted (for an X post) and when ThyMind saved its own copy, such as
 *   "8:14 am · 14 Oct 2026 · Saved 14 Oct"; null when neither can be read.
 */
fun postTimeLine(card: Card, today: LocalDate = LocalDate.now(), zone: ZoneId = ZoneId.systemDefault()): String? {
    val posted = postedAt(card)?.atZone(zone)?.let { "${clock(it)} · ${it.format(DateTimeFormatter.ofPattern("d MMM yyyy", Locale.UK))}" }
    return listOfNotNull(posted, savedLabel(card.created, today, zone)).joinToString(" · ").ifEmpty { null }
}

/**
 * @param pages How many pages the PDF has, once the phone has read it.
 * @param minutes How long its words take to read, once the phone has read them.
 * @param saved When it was saved, such as "Saved 24 Oct".
 * @return Such as "PDF · 48 pages · 12 min read · Saved 24 Oct", leaving out what is not known.
 */
fun pdfDetail(pages: Int?, minutes: Int?, saved: String?): String = listOfNotNull(
    "PDF",
    pages?.takeIf { it > 0 }?.let { if (it == 1) "1 page" else "$it pages" },
    minutes?.takeIf { it > 0 }?.let { "$it min read" },
    saved,
).joinToString(" · ")

/** Such as "8:14 am", as X writes its times. */
private fun clock(time: ZonedDateTime): String =
    "${time.format(DateTimeFormatter.ofPattern("h:mm", Locale.UK))} ${if (time.hour < 12) "am" else "pm"}"

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

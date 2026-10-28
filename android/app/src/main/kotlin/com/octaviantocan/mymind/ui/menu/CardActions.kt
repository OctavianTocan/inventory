package com.octaviantocan.mymind.ui.menu

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.noteText
import com.octaviantocan.mymind.ui.library.postText
import com.octaviantocan.mymind.ui.library.pinned
import com.octaviantocan.mymind.ui.library.readable
import com.octaviantocan.mymind.ui.library.tileKind

/**
 * One row of the card menu.
 *
 * @property action What the row does.
 * @property locked True for an upsell row the account's plan does not include; it shows a lock.
 */
data class CardMenuItem(val action: CardMenuAction, val locked: Boolean = false)

/**
 * The card menu rows the web front-end asks the app to show for [card], in its order.
 *
 * @param card Card the menu is for.
 * @param spaceId Space the card was opened from, or null outside a space.
 * @param features Plan features of the account, such as `sameVibe` and `reader`, from the web page config.
 * @return The rows, top to bottom.
 */
fun cardMenuItems(card: Card, spaceId: String?, features: Set<String> = emptySet()): List<CardMenuItem> = buildList {
    if (card.type == "Image") add(CardMenuItem(CardMenuAction.SameVibe, locked = "sameVibe" !in features))
    if (card.readable && "reader" !in features) add(CardMenuItem(CardMenuAction.ReaderMode, locked = true))
    if (card.blob != null) add(CardMenuItem(CardMenuAction.Download))
    if (!card.url.isNullOrBlank()) add(CardMenuItem(CardMenuAction.ViewSource))
    if (canCopy(card)) add(CardMenuItem(CardMenuAction.Copy))
    add(CardMenuItem(CardMenuAction.AddTags))
    add(CardMenuItem(if (spaceId != null) CardMenuAction.RemoveFromSpace else CardMenuAction.AddToSpace))
    add(CardMenuItem(if (card.pinned) CardMenuAction.RemoveFromTopOfMind else CardMenuAction.AddToTopOfMind))
    add(CardMenuItem(CardMenuAction.DeleteCard))
}

private fun canCopy(card: Card) = when (card.type) {
    "Note", "Content", "Quote", "Quotation" -> true
    "Image" -> card.blob?.type != "image/svg+xml"
    else -> false
}

/** What happened after a card menu row was tapped. */
sealed interface CardMenuOutcome {
    /**
     * The card went to the trash.
     *
     * @property cardId Card that was deleted.
     */
    data class Deleted(val cardId: String) : CardMenuOutcome

    /**
     * The card left a space.
     *
     * @property spaceId Space the card left.
     * @property cardId Card that left.
     */
    data class RemovedFromSpace(val spaceId: String, val cardId: String) : CardMenuOutcome

    /**
     * The card was pinned to Top of Mind or taken off it.
     *
     * @property card The card with its new pin state.
     */
    data class PinChanged(val card: Card) : CardMenuOutcome

    /**
     * The card's text is ready for the clipboard.
     *
     * @property text Text to copy.
     */
    data class Copy(val text: String) : CardMenuOutcome

    /**
     * The card's source page should open in the browser.
     *
     * @property url Address of the source.
     */
    data class OpenUrl(val url: String) : CardMenuOutcome

    /**
     * The card's file was saved to the Downloads folder.
     *
     * @property fileName Name of the saved file.
     */
    data class Downloaded(val fileName: String) : CardMenuOutcome

    /**
     * The library should show the cards that look like this one.
     *
     * @property card Card whose look is searched for.
     */
    data class ShowSimilar(val card: Card) : CardMenuOutcome

    /**
     * The menu should switch to a submenu.
     *
     * @property submenu Submenu to show.
     */
    data class OpenSubmenu(val submenu: CardSubmenu) : CardMenuOutcome

    /**
     * The row has no effect in this app yet.
     *
     * @property action The row that was tapped.
     */
    data class NotAvailable(val action: CardMenuAction) : CardMenuOutcome

    /**
     * The server refused or could not be reached.
     *
     * @property failure Why.
     */
    data class Failed(val failure: ApiFailure) : CardMenuOutcome
}

/**
 * The first page a card's sheet opens on; its child pages are [CardSubmenu]s.
 *
 * @property title Header title of the page.
 */
enum class CardSheetRoot(val title: String) {
    /** The card's actions. */
    Actions("Card actions"),

    /** Ways to share the card. */
    Share("Share"),
}

/**
 * The pages that replace the card menu inside its sheet.
 *
 * @property title Header title.
 * @property parent Parent page; null returns to the action list.
 */
enum class CardSubmenu(val title: String, val parent: CardSubmenu? = null) {
    /** Add tags. */
    Tags("Add tags"),

    /** Select a space. */
    Spaces("Select a space"),

    /** Create a space inside the space selector. */
    CreateSpace("Create a space", Spaces);

    /** Navigation depth determines the direction of the sheet page transition. */
    val depth: Int get() = (parent?.depth ?: 0) + 1
}

/**
 * Carries out card menu rows against the library.
 *
 * @param library Reads and writes of the account's library.
 * @param saver Where Download puts a card's file.
 */
class CardActions(private val library: Library, private val saver: FileSaver) {
    /**
     * @param item Row the user tapped.
     * @param card Card the menu is for.
     * @param spaceId Space the card was opened from, or null.
     * @return What happened; the caller updates its lists and shows the result.
     */
    suspend fun perform(item: CardMenuItem, card: Card, spaceId: String?): CardMenuOutcome {
        if (item.locked) return CardMenuOutcome.NotAvailable(item.action)
        return when (item.action) {
            CardMenuAction.ViewSource -> card.url?.let { CardMenuOutcome.OpenUrl(it) } ?: CardMenuOutcome.NotAvailable(item.action)
            CardMenuAction.Copy -> copyText(card)?.let { CardMenuOutcome.Copy(it) } ?: CardMenuOutcome.NotAvailable(item.action)
            CardMenuAction.Download -> download(card) ?: CardMenuOutcome.NotAvailable(item.action)
            CardMenuAction.SameVibe -> CardMenuOutcome.ShowSimilar(card)
            CardMenuAction.AddTags -> CardMenuOutcome.OpenSubmenu(CardSubmenu.Tags)
            CardMenuAction.AddToSpace -> CardMenuOutcome.OpenSubmenu(CardSubmenu.Spaces)
            CardMenuAction.AddToTopOfMind, CardMenuAction.RemoveFromTopOfMind -> {
                val pin = item.action == CardMenuAction.AddToTopOfMind
                library.setPinned(card.id, pin).then { CardMenuOutcome.PinChanged(card.withPin(pin)) }
            }
            CardMenuAction.DeleteCard -> library.trash(card.id).then { CardMenuOutcome.Deleted(card.id) }
            CardMenuAction.RemoveFromSpace -> {
                val space = spaceId ?: return CardMenuOutcome.NotAvailable(item.action)
                library.removeFromSpace(space, card.id).then { CardMenuOutcome.RemovedFromSpace(space, card.id) }
            }
            CardMenuAction.ReaderMode -> CardMenuOutcome.NotAvailable(item.action)
        }
    }

    private suspend fun download(card: Card): CardMenuOutcome? {
        val blob = card.blob ?: return null
        val path = blob.path ?: return null
        val name = downloadName(card)
        return when (val saved = saver.save(name, blob.type) { out -> library.media(path) { it.copyTo(out) } }) {
            is ApiResult.Ok -> CardMenuOutcome.Downloaded(name)
            is ApiResult.Failed -> CardMenuOutcome.Failed(saved.failure)
        }
    }
}

/**
 * The text Copy puts on the clipboard: a note's plain text. The web copies `GET /cards/{id}/markdown`, which no
 * backend of this project serves, so formatting is lost and pictures are not copied.
 *
 * @param card Card to copy.
 * @return The text, or null when the card has none to copy.
 */
fun copyText(card: Card): String? = when (card.type) {
    "Note", "Content", "Quote", "Quotation" -> noteText(card).ifBlank { card.title }?.takeIf { it.isNotBlank() }
    else -> null
}

/**
 * The words Share sends as text: a note's own words, or a post's words with its link.
 *
 * @param card Card to share.
 * @return The text, or null when the card has no words of its own to share, such as a saved link or a picture.
 */
fun shareText(card: Card): String? = when (tileKind(card)) {
    TileKind.Note -> noteText(card).ifBlank { null }
    TileKind.Post -> postText(card).takeIf { it.isNotBlank() }?.let { words -> listOfNotNull(words, card.url?.takeIf { it.isNotBlank() }).joinToString("\n\n") }
    else -> null
}

/**
 * The file name Download saves a card's file under: the card id and its format, as the web names it.
 *
 * @param card Card with an uploaded file.
 * @return Such as `abc123.png`.
 */
fun downloadName(card: Card): String {
    val fromPath = card.blob?.path?.substringAfterLast('/')?.substringAfterLast('.', "")?.takeIf { it.length in 1..5 }
    val fromType = card.blob?.type?.substringAfter('/')?.substringBefore('+')?.substringBefore(';')
    val format = (fromPath ?: fromType)?.lowercase()?.takeIf { it.isNotEmpty() }
    return if (format != null) "${card.id}.$format" else card.id
}

private inline fun ApiResult<Unit>.then(done: () -> CardMenuOutcome): CardMenuOutcome = when (this) {
    is ApiResult.Ok -> done()
    is ApiResult.Failed -> CardMenuOutcome.Failed(failure)
}

private const val PINNED_FLAG = 512

/** Copy of the card with the Top of Mind pin set or cleared. */
private fun Card.withPin(pinned: Boolean): Card =
    if (pinned) copy(flags = flags or PINNED_FLAG) else copy(flags = flags and PINNED_FLAG.inv(), pinPosition = null)

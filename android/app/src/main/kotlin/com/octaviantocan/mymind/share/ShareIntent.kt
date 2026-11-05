package com.octaviantocan.mymind.share

import com.octaviantocan.mymind.ui.library.UploadQueue

/** One thing a share brings. */
sealed interface ShareItem {
    /**
     * A link or a piece of text.
     *
     * @property note Words shared around a link, kept as its card's note when the card has none.
     */
    data class Text(val text: String, val note: String? = null) : ShareItem

    /** A picture, video or PDF the sharing app handed over by address. */
    data class Stream(val uri: String) : ShareItem
}

/** What the share target does with an intent. */
sealed interface ShareState {
    /** The intent holds nothing to save. */
    data object Malformed : ShareState

    /** More than [MAX_SHARE_ITEMS] streams. */
    data object TooManyItems : ShareState

    /** A stream is over [MAX_SHARE_BYTES]. */
    data object FileTooBig : ShareState

    /** A shared PDF is locked with a password. */
    data object PdfHasPassword : ShareState

    /** Nobody is signed in. */
    data object NoLogin : ShareState

    /** The intent can be saved. */
    data class Ready(val items: List<ShareItem>) : ShareState
}

/** Largest file the share target takes: as big as an upload from inside the app, 5 GB. */
const val MAX_SHARE_BYTES = UploadQueue.MAX_UPLOAD_SIZE

/** Most streams one share takes. */
const val MAX_SHARE_ITEMS = 20

/** Reads the files behind shared addresses. */
interface ShareProbe {
    /** @return Length in bytes of the file at [uri], or null when it cannot be told. */
    fun size(uri: String): Long?

    /** @return True when the PDF at [uri] needs a password. */
    fun isLockedPdf(uri: String): Boolean
}

/**
 * The parts of an intent the share target reads.
 *
 * @property action Intent action.
 * @property mime Intent MIME type, or an empty string.
 * @property streams `EXTRA_STREAM` addresses, or null when there are none.
 * @property text `EXTRA_TEXT`, or the text of `PROCESS_TEXT`.
 * @property data The intent's data address.
 */
data class ShareRequest(
    val action: String?,
    val mime: String,
    val streams: List<String>?,
    val text: String?,
    val data: String?,
)

/** Intent action of a text selection sent to the share target. */
const val ACTION_PROCESS_TEXT = "android.intent.action.PROCESS_TEXT"

private const val ACTION_SEND = "android.intent.action.SEND"
private const val ACTION_SEND_MULTIPLE = "android.intent.action.SEND_MULTIPLE"
private const val PDF = "application/pdf"

/**
 * Decides what to do with an intent the way the original share target does.
 *
 * @param request The intent's parts.
 * @param signedIn Whether a session exists.
 * @param probe Reads the shared files.
 * @return The state the sheet opens in.
 */
fun parseShare(request: ShareRequest, signedIn: Boolean, probe: ShareProbe): ShareState {
    if (!signedIn) return ShareState.NoLogin
    val streams = request.streams.orEmpty()
    return when {
        request.action == ACTION_SEND_MULTIPLE -> {
            if (streams.isEmpty()) return ShareState.Malformed
            if (streams.size > MAX_SHARE_ITEMS) return ShareState.TooManyItems
            checkFiles(streams, request.mime, probe) ?: ShareState.Ready(streams.map { ShareItem.Stream(it) })
        }
        request.action == ACTION_SEND && streams.isNotEmpty() && isFileMime(request.mime) ->
            checkFiles(streams.take(1), request.mime, probe) ?: ShareState.Ready(listOf(ShareItem.Stream(streams.first())))
        else -> {
            val text = (request.text ?: request.data)?.trim().orEmpty()
            val item = if (request.action == ACTION_PROCESS_TEXT) ShareItem.Text(text) else sharedText(text)
            if (text.isEmpty()) ShareState.Malformed else ShareState.Ready(listOf(item))
        }
    }
}

private val LINK_IN_TEXT = Regex("""https?://[^\s<>"“”‘’]+""")

/**
 * What a shared text saves: text holding exactly one link and other words, such as "Check this out https://…" or a
 * place name above a map link, saves the link with those words as its note; any other text saves as it came.
 *
 * @param text Shared text, trimmed.
 * @return The item to save.
 */
fun sharedText(text: String): ShareItem.Text {
    val found = LINK_IN_TEXT.findAll(text).toList()
    if (found.size != 1) return ShareItem.Text(text)
    val match = found.single()
    val link = withoutClosingPunctuation(match.value)
    val words = text.removeRange(match.range)
        .lines().map { it.trim() }.filter { it.isNotEmpty() }.joinToString("\n")
    return ShareItem.Text(link, words.takeIf { line -> line.any { it.isLetterOrDigit() } })
}

// A link at the end of a sentence or inside brackets loses the punctuation that closes them; a bracket the link opened stays.
private fun withoutClosingPunctuation(link: String): String {
    var end = link.length
    while (end > 0) {
        val last = link[end - 1]
        val opening = when (last) {
            ')' -> '('
            ']' -> '['
            '}' -> '{'
            else -> null
        }
        val closes = when {
            last in ".,;:!?'" -> true
            opening != null -> link.substring(0, end).count { it == last } > link.substring(0, end).count { it == opening }
            else -> false
        }
        if (!closes) break
        end--
    }
    return link.substring(0, end)
}

private fun isFileMime(mime: String) = mime.startsWith("image/") || mime.startsWith("video/") || mime.startsWith("audio/") || mime == PDF

private fun checkFiles(uris: List<String>, mime: String, probe: ShareProbe): ShareState? {
    if (mime == PDF && uris.any { probe.isLockedPdf(it) }) return ShareState.PdfHasPassword
    if (uris.any { (probe.size(it) ?: 0L) > MAX_SHARE_BYTES }) return ShareState.FileTooBig
    return null
}

package com.octaviantocan.mymind.share

/** One thing a share brings. */
sealed interface ShareItem {
    /** A link or a piece of text. */
    data class Text(val text: String) : ShareItem

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

/** Largest file the share target takes: 25 MB. */
const val MAX_SHARE_BYTES = 26_214_400L

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
            if (text.isEmpty()) ShareState.Malformed else ShareState.Ready(listOf(ShareItem.Text(text)))
        }
    }
}

private fun isFileMime(mime: String) = mime.startsWith("image/") || mime.startsWith("video/") || mime == PDF

private fun checkFiles(uris: List<String>, mime: String, probe: ShareProbe): ShareState? {
    if (mime == PDF && uris.any { probe.isLockedPdf(it) }) return ShareState.PdfHasPassword
    if (uris.any { (probe.size(it) ?: 0L) > MAX_SHARE_BYTES }) return ShareState.FileTooBig
    return null
}

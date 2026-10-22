package com.octaviantocan.mymind.share

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.ui.library.UploadQueue
import com.octaviantocan.mymind.update.UploadActivity
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * What the user added in the sheet.
 *
 * @property tags Tags for every saved card.
 * @property note Note for every saved card, or blank for none.
 * @property spaceIds Spaces every saved card joins.
 */
data class ShareDetails(val tags: List<String> = emptyList(), val note: String = "", val spaceIds: Set<String> = emptySet())

/**
 * Saves what was shared and puts the sheet's details on the saved cards.
 *
 * @param scope Scope the saving runs in.
 * @param library Target of the calls.
 * @param io Dispatcher the files are read on.
 * @param activity Counter of the uploads in flight, which counts a share until it is saved, or null.
 * @param open Turns a shared address into a file, or null when it cannot be read.
 */
class ShareController(
    private val scope: CoroutineScope,
    private val library: Library,
    io: CoroutineDispatcher = Dispatchers.IO,
    private val activity: UploadActivity? = null,
    private val open: suspend (String) -> UploadFile?,
) {
    private val savedIds = MutableStateFlow<List<String>>(emptyList())
    private val failures = MutableStateFlow(0)
    private var expected = 0

    /** Upload queue of the shared files. */
    val queue = UploadQueue(scope, library, onUploaded = { id, _ -> savedIds.update { it + id } }, io = io, activity = activity)

    /** Cards saved so far. */
    val cards: StateFlow<List<String>> = savedIds

    /**
     * Starts saving every item.
     *
     * @param items What was shared.
     */
    fun start(items: List<ShareItem>) {
        expected = items.size
        activity?.begin()
        scope.launch { save(items) }.invokeOnCompletion { activity?.end() }
    }

    private suspend fun save(items: List<ShareItem>) {
        val files = ArrayList<UploadFile>()
        for (item in items) {
            when (item) {
                is ShareItem.Text -> when (val saved = library.createFromContent(item.text, SOURCE, null)) {
                    is ApiResult.Ok -> {
                        item.note?.let { words -> noteIfEmpty(saved.value, words) }
                        savedIds.update { it + saved.value }
                    }
                    is ApiResult.Failed -> failures.update { it + 1 }
                }
                is ShareItem.Stream -> open(item.uri)?.let { files += it } ?: failures.update { it + 1 }
            }
        }
        queue.add(files, null)
    }

    // A link saved before keeps its note; a failure leaves the card as it is.
    private suspend fun noteIfEmpty(cardId: String, words: String) {
        val card = (library.card(cardId) as? ApiResult.Ok)?.value ?: return
        if (card.note.isNullOrBlank()) library.setNote(cardId, words)
    }

    /**
     * Waits until every item is saved or has failed, then puts [details] on the saved cards.
     *
     * @param details Tags, note and spaces for every card.
     * @return Success, or the first failure; with nothing saved, a network failure.
     */
    suspend fun finish(details: ShareDetails): ApiResult<Unit> {
        activity?.begin()
        try {
            return applyDetails(details)
        } finally {
            activity?.end()
        }
    }

    private suspend fun applyDetails(details: ShareDetails): ApiResult<Unit> {
        val ids = combine(savedIds, failures, queue.state) { ids, failed, uploads ->
            ids to (ids.size + failed + uploads.items.count { it.error != null } >= expected)
        }.first { it.second }.first
        if (ids.isEmpty()) return ApiResult.Failed(ApiFailure.Network)
        for (id in ids) {
            for (tag in details.tags) (library.addTag(id, tag) as? ApiResult.Failed)?.let { return it }
            if (details.note.isNotBlank()) (library.setNote(id, details.note.trim()) as? ApiResult.Failed)?.let { return it }
            for (space in details.spaceIds) (library.addToSpace(space, id) as? ApiResult.Failed)?.let { return it }
        }
        return ApiResult.Ok(Unit)
    }

    private companion object {
        const val SOURCE = "clipboard"
    }
}

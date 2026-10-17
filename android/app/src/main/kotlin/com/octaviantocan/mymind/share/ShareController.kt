package com.octaviantocan.mymind.share

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.ui.library.UploadQueue
import com.octaviantocan.mymind.update.UploadActivity
import com.octaviantocan.mymind.update.UploadWork
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicInteger

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
 * @param activity Registry that learns whether a share is still being saved, or null.
 * @param open Turns a shared address into a file, or null when it cannot be read.
 */
class ShareController(
    private val scope: CoroutineScope,
    private val library: Library,
    io: CoroutineDispatcher = Dispatchers.IO,
    activity: UploadActivity? = null,
    private val open: suspend (String) -> UploadFile?,
) : UploadWork {
    private val savedIds = MutableStateFlow<List<String>>(emptyList())
    private val failures = MutableStateFlow(0)
    private var expected = 0
    private val working = AtomicInteger(0)

    /** Upload queue of the shared files. */
    val queue = UploadQueue(scope, library, onUploaded = { id, _ -> savedIds.update { it + id } }, io = io)

    override val busy: Boolean get() = working.get() > 0 || queue.busy

    init {
        activity?.register(this)
    }

    /** Cards saved so far. */
    val cards: StateFlow<List<String>> = savedIds

    /**
     * Starts saving every item.
     *
     * @param items What was shared.
     */
    fun start(items: List<ShareItem>) {
        expected = items.size
        working.incrementAndGet()
        scope.launch {
            try {
                save(items)
            } finally {
                working.decrementAndGet()
            }
        }
    }

    private suspend fun save(items: List<ShareItem>) {
        val files = ArrayList<UploadFile>()
        for (item in items) {
            when (item) {
                is ShareItem.Text -> when (val saved = library.createFromContent(item.text, SOURCE, null)) {
                    is ApiResult.Ok -> savedIds.update { it + saved.value }
                    is ApiResult.Failed -> failures.update { it + 1 }
                }
                is ShareItem.Stream -> open(item.uri)?.let { files += it } ?: failures.update { it + 1 }
            }
        }
        queue.add(files, null)
    }

    /**
     * Waits until every item is saved or has failed, then puts [details] on the saved cards.
     *
     * @param details Tags, note and spaces for every card.
     * @return Success, or the first failure; with nothing saved, a network failure.
     */
    suspend fun finish(details: ShareDetails): ApiResult<Unit> {
        working.incrementAndGet()
        try {
            return applyDetails(details)
        } finally {
            working.decrementAndGet()
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

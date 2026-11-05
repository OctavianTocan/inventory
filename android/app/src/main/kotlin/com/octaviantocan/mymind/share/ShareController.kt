package com.octaviantocan.mymind.share

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.ui.library.UploadQueue
import com.octaviantocan.mymind.update.UploadActivity
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
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
 * @property untag Names of tags ThyMind put on the cards that the user took off.
 */
data class ShareDetails(
    val tags: List<String> = emptyList(),
    val note: String = "",
    val spaceIds: Set<String> = emptySet(),
    val untag: Set<String> = emptySet(),
) {
    /** True when there is nothing to change on the cards. */
    fun isEmpty() = tags.isEmpty() && note.isBlank() && spaceIds.isEmpty() && untag.isEmpty()
}

/**
 * How far saving a share has come.
 *
 * @property saved Ids of the cards saved so far, in the order they were saved.
 * @property failed How many items could not be saved.
 * @property expected How many items the share brought; zero until saving starts.
 * @property problem Why the first item that failed could not be saved, or null.
 */
data class ShareProgress(val saved: List<String> = emptyList(), val failed: Int = 0, val expected: Int = 0, val problem: String? = null) {
    /** True once every item is saved or has failed. */
    val done: Boolean get() = expected > 0 && saved.size + failed >= expected
}

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
    private val firstProblem = MutableStateFlow<String?>(null)
    private val expected = MutableStateFlow(0)

    /** Upload queue of the shared files. */
    val queue = UploadQueue(scope, library, onUploaded = { id, _ -> savedIds.update { it + id } }, io = io, activity = activity)

    /** Cards saved so far. */
    val cards: StateFlow<List<String>> = savedIds

    /** How far saving has come, counting uploads that failed. */
    val progress: Flow<ShareProgress> = combine(savedIds, failures, firstProblem, expected, queue.state) { ids, failed, problem, count, uploads ->
        val errors = uploads.items.mapNotNull { it.error }
        ShareProgress(ids, failed + errors.size, count, problem ?: errors.firstOrNull())
    }

    /**
     * Starts saving every item.
     *
     * @param items What was shared.
     */
    fun start(items: List<ShareItem>) {
        expected.value = items.size
        activity?.begin()
        scope.launch { save(items) }.invokeOnCompletion { activity?.end() }
    }

    private suspend fun save(items: List<ShareItem>) {
        val files = ArrayList<UploadFile>()
        for (item in items) {
            when (item) {
                is ShareItem.Text -> when (val saved = library.createFromContent(item.text, SOURCE, null, item.note)) {
                    is ApiResult.Ok -> savedIds.update { it + saved.value }
                    is ApiResult.Failed -> failed(describe(saved.failure))
                }
                is ShareItem.Stream -> open(item.uri)?.let { files += it } ?: failed(UNREADABLE)
            }
        }
        queue.add(files, null)
    }

    private fun failed(problem: String) {
        firstProblem.update { it ?: problem }
        failures.update { it + 1 }
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
        val ids = progress.first { it.done }.saved
        if (ids.isEmpty()) return ApiResult.Failed(ApiFailure.Network)
        for (id in ids) {
            if (details.untag.isNotEmpty()) (untag(id, details.untag) as? ApiResult.Failed)?.let { return it }
            for (tag in details.tags) (library.addTag(id, tag) as? ApiResult.Failed)?.let { return it }
            if (details.note.isNotBlank()) (library.setNote(id, details.note.trim()) as? ApiResult.Failed)?.let { return it }
            for (space in details.spaceIds) (library.addToSpace(space, id) as? ApiResult.Failed)?.let { return it }
        }
        return ApiResult.Ok(Unit)
    }

    private suspend fun untag(id: String, names: Set<String>): ApiResult<Unit> {
        val card = when (val read = library.card(id)) {
            is ApiResult.Ok -> read.value
            is ApiResult.Failed -> return read
        }
        val wrong = card.tags.orEmpty().filter { tag -> (tag.flags ?: 0) and AI_TAG != 0 && names.any { it.equals(tag.name, ignoreCase = true) } }
        for (tag in wrong) (tag.id?.let { library.removeTag(it) } as? ApiResult.Failed)?.let { return it }
        return ApiResult.Ok(Unit)
    }

    private companion object {
        const val AI_TAG = 2
        const val SOURCE = "clipboard"
        const val UNREADABLE = "The app you shared from did not let ThyMind read the file."
    }
}

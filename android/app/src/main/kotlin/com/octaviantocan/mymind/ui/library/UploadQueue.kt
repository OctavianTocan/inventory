package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.UploadAuthorization
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.update.UploadActivity
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.io.IOException
import java.io.InputStream

/**
 * One file in the upload queue.
 *
 * @property id Number of the file in the order files were added.
 * @property name File name.
 * @property mime Media type, which picks the row's icon and label.
 * @property added When the file was added, in milliseconds since the epoch.
 * @property running True while the file is being sent.
 * @property error Why the upload failed, or null.
 */
data class QueuedUpload(
    val id: Long,
    val name: String,
    val mime: String,
    val added: Long,
    val running: Boolean = false,
    val error: String? = null,
)

/**
 * The upload queue strip.
 *
 * @property items Files waiting, being sent or failed, in the order they were added; a finished upload leaves.
 * @property details True while the strip lists the items.
 */
data class UploadQueueState(val items: List<QueuedUpload> = emptyList(), val details: Boolean = false)

/**
 * Uploads files one after another the way the web does: authorize, send each file in pieces, then create its Media
 * card and, inside a space, put the card into it.
 *
 * @param scope Scope the uploads run in.
 * @param library Target of the upload calls.
 * @param onUploaded Called with each new card's id and the space it joined, or null.
 * @param clock Current time in milliseconds since the epoch.
 * @param chunkSize Largest piece sent in one request.
 * @param io Dispatcher the files are opened and read on.
 * @param activity Counter of the uploads in flight, or null.
 */
class UploadQueue(
    private val scope: CoroutineScope,
    private val library: Library,
    private val onUploaded: suspend (cardId: String, spaceId: String?) -> Unit,
    private val clock: () -> Long = System::currentTimeMillis,
    private val chunkSize: Long = CHUNK_SIZE,
    private val io: CoroutineDispatcher = Dispatchers.IO,
    private val activity: UploadActivity? = null,
) {
    private val mutableState = MutableStateFlow(UploadQueueState())
    private val turn = Mutex()
    private var nextId = 0L

    /** The items and whether the list is open. */
    val state: StateFlow<UploadQueueState> = mutableState

    /**
     * Queues files and uploads them after the ones already queued.
     *
     * @param files Picked files.
     * @param spaceId Space the new cards join, or null for the library only.
     */
    fun add(files: List<UploadFile>, spaceId: String?) {
        if (files.isEmpty()) return
        val queued = files.map { file -> QueuedUpload(nextId++, file.name, file.mime, clock()) to file }
        mutableState.update { it.copy(items = it.items + queued.map { (item, _) -> item }) }
        activity?.begin()
        scope.launch {
            turn.withLock { queued.chunked(AUTHORIZE_LIMIT).forEach { batch -> send(batch, spaceId) } }
        }.invokeOnCompletion { activity?.end() }
    }

    /** Opens or closes the list of items. */
    fun toggleDetails() = mutableState.update { it.copy(details = !it.details) }

    /**
     * Takes a failed item off the queue.
     *
     * @param id The item's [QueuedUpload.id].
     */
    fun dismiss(id: Long) = mutableState.update { state -> state.copy(items = state.items.filterNot { it.id == id }) }

    private suspend fun send(batch: List<Pair<QueuedUpload, UploadFile>>, spaceId: String?) {
        val readable = batch.filter { (item, file) ->
            when {
                file.size <= 0 -> fail(item.id, EMPTY_FILE)
                file.size > MAX_UPLOAD_SIZE -> fail(item.id, TOO_LARGE)
                else -> return@filter true
            }
            false
        }
        if (readable.isEmpty()) return
        val authorizations = when (val answer = library.authorizeUploads(readable.size)) {
            is ApiResult.Ok -> answer.value
            is ApiResult.Failed -> return readable.forEach { (item, _) -> fail(item.id, describe(answer.failure)) }
        }
        readable.forEachIndexed { index, (item, file) ->
            val authorization = authorizations.getOrNull(index) ?: return@forEachIndexed fail(item.id, UNKNOWN_ERROR)
            mark(item.id) { it.copy(running = true) }
            when (val created = upload(authorization, file, spaceId)) {
                is ApiResult.Ok -> {
                    mutableState.update { state -> state.copy(items = state.items.filterNot { it.id == item.id }) }
                    onUploaded(created.value, spaceId)
                }
                is ApiResult.Failed -> fail(item.id, describe(created.failure))
            }
        }
    }

    private suspend fun upload(authorization: UploadAuthorization, file: UploadFile, spaceId: String?): ApiResult<String> {
        val blobId = try {
            withContext(io) { file.open().use { sendPieces(authorization, file, it) } }
        } catch (_: IOException) {
            return ApiResult.Failed(ApiFailure.Network)
        } catch (_: SecurityException) {
            return ApiResult.Failed(ApiFailure.Server(0))
        }
        val uploadId = when (blobId) {
            is ApiResult.Ok -> blobId.value
            is ApiResult.Failed -> return blobId
        }
        val cardId = when (val answer = library.createMedia(uploadId, spaceId)) {
            is ApiResult.Ok -> answer.value
            is ApiResult.Failed -> return answer
        }
        if (spaceId != null) (library.addToSpace(spaceId, cardId) as? ApiResult.Failed)?.let { return it }
        return ApiResult.Ok(cardId)
    }

    /** Sends [file] piece by piece from [source] and answers the finished upload's blob id. */
    private suspend fun sendPieces(authorization: UploadAuthorization, file: UploadFile, source: InputStream): ApiResult<String> {
        var uploadId: String? = null
        var blobId: String? = null
        var start = 0L
        while (start < file.size) {
            val length = minOf(chunkSize, file.size - start)
            when (val answer = library.uploadChunk(authorization, uploadId, file, start, length, source)) {
                is ApiResult.Ok -> {
                    uploadId = uploadId ?: answer.value.id
                    blobId = answer.value.blob?.id ?: answer.value.id
                }
                is ApiResult.Failed -> return answer
            }
            start += length
        }
        return blobId?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Server(0))
    }

    private fun fail(id: Long, error: String) = mark(id) { it.copy(running = false, error = error) }

    private fun mark(id: Long, change: (QueuedUpload) -> QueuedUpload) =
        mutableState.update { state -> state.copy(items = state.items.map { if (it.id == id) change(it) else it }) }

    /** Upload limits of the web flow. */
    companion object {
        /** Largest piece of a file the server takes in one request: 32 MiB. */
        const val CHUNK_SIZE = 32L * 1024 * 1024

        /** Largest file the server takes (`maxUploadSize`). */
        const val MAX_UPLOAD_SIZE = 500_000_000L

        /** Most authorizations one `POST /uploads/authorize` hands out. */
        const val AUTHORIZE_LIMIT = 100
    }
}

private const val EMPTY_FILE = "This file is empty."
private const val TOO_LARGE = "This file is larger than 500 MB."
private const val UNKNOWN_ERROR = "Unknown error"

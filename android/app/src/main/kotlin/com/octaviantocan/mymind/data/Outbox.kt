package com.octaviantocan.mymind.data

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/** How the ids of tags added while offline start, so a removal can tell them from the server's. */
internal const val LOCAL_TAG = "outbox-tag:"

// Card flag bit of a card pinned to Top of Mind.
private const val PINNED_FLAG = 512

/** A change to a card made in the app, kept on the phone until the server has it. */
@Serializable
sealed interface Edit {
    /** The card changed, or null when the phone does not know it. */
    val cardId: String?

    /** @property pinned True pins the card to Top of Mind, false takes it off. */
    @Serializable @SerialName("pin")
    data class Pin(override val cardId: String, val pinned: Boolean) : Edit

    /** Moves the card to the trash. */
    @Serializable @SerialName("trash")
    data class Trash(override val cardId: String) : Edit

    /** @property text The card's note. */
    @Serializable @SerialName("note")
    data class Note(override val cardId: String, val text: String) : Edit

    /**
     * @property name Tag name.
     * @property localId The id the phone shows the tag under until the server gives it one.
     */
    @Serializable @SerialName("add-tag")
    data class AddTag(override val cardId: String, val name: String, val localId: String) : Edit

    /** @property tagId The server's id of the card's tag. */
    @Serializable @SerialName("remove-tag")
    data class RemoveTag(override val cardId: String?, val tagId: String) : Edit

    /** @property spaceId Space the card joins. */
    @Serializable @SerialName("add-to-space")
    data class AddToSpace(override val cardId: String, val spaceId: String) : Edit

    /** @property spaceId Space the card leaves. */
    @Serializable @SerialName("remove-from-space")
    data class RemoveFromSpace(override val cardId: String, val spaceId: String) : Edit
}

/**
 * The card as it is once [edit] is made, as the phone shows it before the server answers.
 *
 * @param edit A change to this card.
 * @return The changed card, or null when the change trashes it.
 */
fun Card.edited(edit: Edit): Card? = when (edit) {
    is Edit.Pin -> if (edit.pinned) copy(flags = flags or PINNED_FLAG) else copy(flags = flags and PINNED_FLAG.inv(), pinPosition = null)
    is Edit.Trash -> null
    is Edit.Note -> copy(note = edit.text)
    is Edit.AddTag -> if (tags.orEmpty().any { it.name == edit.name }) this else copy(tags = tags.orEmpty() + Tag(edit.localId, edit.name))
    is Edit.RemoveTag -> copy(tags = tags.orEmpty().filterNot { it.id == edit.tagId })
    is Edit.AddToSpace -> if (edit.spaceId in spaces.orEmpty()) this else copy(spaces = spaces.orEmpty() + edit.spaceId)
    is Edit.RemoveFromSpace -> copy(spaces = spaces.orEmpty() - edit.spaceId, spaceRanks = spaceRanks?.minus(edit.spaceId))
}

/**
 * The notice shown when the server refused a change made offline.
 *
 * @param card The changed card as the phone had it, or null.
 * @return One sentence naming the card by its title when it has one.
 */
fun refusedNotice(card: Card?): String =
    card?.title?.takeIf { it.isNotBlank() }?.let { "Your change to “$it” could not be saved." } ?: "A change to a card could not be saved."

/**
 * Changes made on the phone, kept in the phone's copy and sent in the order they were made once the server can be
 * reached. The copy shows each change as soon as it is queued.
 *
 * With [push], the queue goes out in batches, each change under its place in the queue, so the server applies a change
 * sent twice only once. A server without the push gets each change through its own request instead.
 *
 * @param copy The phone's copy, which holds the queue.
 * @param server The server's library the changes go to.
 * @param onQueued Called after a change joins the queue, such as to send it once the phone is online.
 * @param onRefused Called with a change the server refused, and its card as the phone had it; the change is dropped.
 * @param push Sends a batch through the server's push, or null to send each change on its own.
 */
class Outbox(
    private val copy: LibraryCopy,
    private val server: Library,
    private val onQueued: () -> Unit = {},
    private val onRefused: (Edit, Card?) -> Unit = { _, _ -> },
    private val push: (suspend (PushRequest) -> ApiResult<PushAnswer>)? = null,
) {
    private val sending = Mutex()
    private val sentTags = ConcurrentHashMap<String, String>()

    // False once the server answered a push with anything but a failure worth trying again, such as an older server's
    // 404; the app's next start asks again.
    @Volatile
    private var pushes = push != null

    /** @return True when no change waits to be sent, or the queue cannot be read. */
    suspend fun isEmpty(): Boolean = copy.firstQueued() == null

    /**
     * Queues a change and shows it in the copy.
     *
     * @param edit The change.
     * @return True when it was queued; false when signed out or the copy failed.
     */
    suspend fun queue(edit: Edit): Boolean = copy.queue(edit).also { if (it) onQueued() }

    /** @return A new id for a tag added while offline. */
    fun newTagId(): String = LOCAL_TAG + UUID.randomUUID()

    /**
     * Takes back a tag added while offline.
     *
     * @param localId The tag's id on the phone.
     * @return The server's id of the tag once sent; [localId] itself when the phone no longer knows it; null when it
     *   never left the phone and is now gone from the queue and the copy.
     */
    suspend fun takeBackTag(localId: String): String? = if (copy.unqueueTag(localId)) null else sentTags[localId] ?: localId

    /**
     * Sends the queued changes in order. A change the server refuses is dropped, its card read again, and
     * [onRefused] told.
     *
     * @return Null once nothing waits; otherwise the failure that stopped it, such as no network, with that change and
     *   the ones after it kept.
     */
    suspend fun send(): ApiFailure? = sending.withLock { sendQueued() }

    private suspend fun sendQueued(): ApiFailure? {
        if (pushes) {
            val failure = pushQueued() ?: return null
            if (failure.keepsChanges()) return failure
            pushes = false
        }
        while (true) {
            val (seq, edit) = copy.firstQueued() ?: return null
            val failure = (deliver(edit) as? ApiResult.Failed)?.failure
            if (failure != null && failure.keepsChanges()) return failure
            if (!copy.unqueue(seq)) return ApiFailure.Server(0)
            if (failure != null) refuse(edit)
        }
    }

    // Sends the queue through the push, a batch at a time; null once nothing waits, else the failure that stopped it.
    private suspend fun pushQueued(): ApiFailure? {
        val send = push ?: return NO_PUSH
        val client = copy.clientId() ?: return ApiFailure.Server(0)
        while (true) {
            val batch = copy.queued(PUSH_BATCH)
            if (batch.isEmpty()) return null
            // An older server answers a write it does not know with `{}`, and an answer the app cannot read fails as
            // Server(0): either way the server has no push.
            val answer = when (val sent = send(PushRequest(client, batch.map { (seq, edit) -> changeOf(seq, edit) }))) {
                is ApiResult.Ok -> sent.value
                is ApiResult.Failed -> return if (sent.failure == ApiFailure.Server(0)) NO_PUSH else sent.failure
            }
            val last = answer.lastChangeId ?: return NO_PUSH
            batch.forEach { (seq, edit) -> if (edit is Edit.AddTag) answer.tagIds[seq.toString()]?.let { sentTags[edit.localId] = it } }
            if (!copy.unqueueThrough(last)) return ApiFailure.Server(0)
            batch.forEach { (seq, edit) -> if (seq in answer.refused) refuse(edit) }
            if (last < batch.last().first) return ApiFailure.Server(0)
        }
    }

    // Puts the server's version of a refused change's card in the copy and tells [onRefused].
    private suspend fun refuse(edit: Edit) {
        val id = edit.cardId
        val card = id?.let { copy.card(it) }
        if (id != null) when (val fresh = server.card(id)) {
            is ApiResult.Ok -> copy.replace(id, fresh.value)
            is ApiResult.Failed -> if (fresh.failure == ApiFailure.Server(404)) copy.replace(id, null)
        }
        onRefused(edit, card)
    }

    private suspend fun deliver(edit: Edit): ApiResult<Unit> = when (edit) {
        is Edit.Pin -> server.setPinned(edit.cardId, edit.pinned)
        is Edit.Trash -> server.trash(edit.cardId)
        is Edit.Note -> server.setNote(edit.cardId, edit.text)
        is Edit.AddTag -> when (val answer = server.addTag(edit.cardId, edit.name)) {
            is ApiResult.Ok -> ApiResult.Ok(Unit).also { answer.value.id?.let { sentTags[edit.localId] = it } }
            is ApiResult.Failed -> answer
        }
        is Edit.RemoveTag -> server.removeTag(edit.tagId)
        is Edit.AddToSpace -> server.addToSpace(edit.spaceId, edit.cardId)
        is Edit.RemoveFromSpace -> server.removeFromSpace(edit.spaceId, edit.cardId)
    }
}

// Changes per push; the server takes up to 100.
private const val PUSH_BATCH = 50

// How a push that found no push on the server ends: not worth trying again, so the changes go one by one.
private val NO_PUSH = ApiFailure.Server(404)

// A change as the push takes it: the edit's fields under its serial name.
private fun changeOf(seq: Long, edit: Edit): Change {
    val fields = AppJson.encodeToJsonElement(Edit.serializer(), edit).jsonObject
    return Change(seq, fields.getValue("type").jsonPrimitive.content, JsonObject(fields - "type"))
}

// Failures worth trying again later: no network, a busy or broken server, or a session that needs signing in again.
private fun ApiFailure.keepsChanges(): Boolean = when (this) {
    ApiFailure.Network, ApiFailure.Locked, ApiFailure.Unauthorized, ApiFailure.InvalidToken -> true
    ApiFailure.NotAllowed, ApiFailure.Invalid -> false
    is ApiFailure.Server -> code == 0 || code == 408 || code >= 500
}

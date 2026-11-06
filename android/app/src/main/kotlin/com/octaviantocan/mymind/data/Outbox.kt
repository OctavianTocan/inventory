package com.octaviantocan.mymind.data

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.net.URI
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/** How the ids of tags added while offline start, so a removal can tell them from the server's. */
internal const val LOCAL_TAG = "outbox-tag:"

// Card flag bit of a card pinned.
private const val PINNED_FLAG = 512

/** A change made in the app, kept on the phone until the server has it. */
@Serializable
sealed interface Edit {
    /** The card changed, or null when the phone does not know it or the change is to a space. */
    val cardId: String?

    /** @property pinned True pins the card, false takes it off. */
    @Serializable @SerialName("pin")
    data class Pin(override val cardId: String, val pinned: Boolean) : Edit

    /** @property rotation How far the card's picture turns clockwise, in degrees: 0, 90, 180 or 270. */
    @Serializable @SerialName("rotate")
    data class Rotate(override val cardId: String, val rotation: Int) : Edit

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

    /**
     * Makes an empty space at the end of the list.
     *
     * @property spaceId The id the phone made for it ([newPhoneId]), which the server keeps.
     * @property name Space name.
     * @property color Space colour as a CSS hex colour.
     */
    @Serializable @SerialName("create-space")
    data class CreateSpace(val spaceId: String, val name: String, val color: String) : Edit {
        override val cardId: String? get() = null
    }

    /** @property spaceId Space to delete; its cards stay in the library. */
    @Serializable @SerialName("delete-space")
    data class DeleteSpace(val spaceId: String) : Edit {
        override val cardId: String? get() = null
    }

    /** Opened from Back today or the sky, which lights the find's star; nothing on the card changes. */
    @Serializable @SerialName("visit")
    data class Visit(override val cardId: String) : Edit

    /**
     * Saves a link or text as a new card.
     *
     * @property cardId The id the phone made for it ([newPhoneId]), which the server keeps unless the link is already
     *   saved.
     * @property content The link or the text.
     * @property source Where the card came from.
     * @property note Words that become its note unless a link saved before has one, or null.
     * @property made When it was saved, as the card's times until the server's card arrives.
     */
    @Serializable @SerialName("create-card")
    data class CreateCard(
        override val cardId: String,
        val content: String,
        val source: String,
        val note: String? = null,
        val made: String,
    ) : Edit
}

/** @return A new id for something the phone makes, such as a space: 32 lowercase hex digits, as the server's push takes. */
fun newPhoneId(): String = UUID.randomUUID().toString().replace("-", "")

// A link the server saves as a link: the whole text one web address, as the server's own test reads it.
private val LINK = Regex("""^https?://\S+$""")

/**
 * The card the phone shows for [this] change until the server's arrives: a link waiting to be read, as the server
 * first saves one, or a note of the text.
 */
fun Edit.CreateCard.placeholder(): Card {
    val link = content.trim().takeIf { LINK.matches(it) }
    val host = link?.let { runCatching { URI(it).host }.getOrNull() }
    val card = Card(id = cardId, type = "Note", flags = 0, created = made, modified = made, bumped = made, note = note?.takeIf { it.isNotBlank() })
    return if (link != null) {
        card.copy(type = "Placeholder", url = link, domain = host, siteName = host, state = "saving")
    } else {
        card.copy(prose = AppJson.parseToJsonElement(proseOf(content)))
    }
}

/**
 * The card as it is once [edit] is made, as the phone shows it before the server answers.
 *
 * @param edit A change to this card.
 * @return The changed card, or null when the change trashes it.
 */
fun Card.edited(edit: Edit): Card? = when (edit) {
    is Edit.Pin -> if (edit.pinned) copy(flags = flags or PINNED_FLAG) else copy(flags = flags and PINNED_FLAG.inv(), pinPosition = null)
    is Edit.Rotate -> copy(rotation = edit.rotation.takeIf { it != 0 })
    is Edit.Trash -> null
    is Edit.Note -> copy(note = edit.text)
    is Edit.AddTag -> if (tags.orEmpty().any { it.name == edit.name }) this else copy(tags = tags.orEmpty() + Tag(edit.localId, edit.name))
    is Edit.RemoveTag -> copy(tags = tags.orEmpty().filterNot { it.id == edit.tagId })
    is Edit.AddToSpace -> if (edit.spaceId in spaces.orEmpty()) this else copy(spaces = spaces.orEmpty() + edit.spaceId)
    is Edit.RemoveFromSpace -> copy(spaces = spaces.orEmpty() - edit.spaceId, spaceRanks = spaceRanks?.minus(edit.spaceId))
    is Edit.CreateSpace, is Edit.CreateCard, is Edit.Visit -> this
    is Edit.DeleteSpace -> copy(spaces = spaces.orEmpty() - edit.spaceId, spaceRanks = spaceRanks?.minus(edit.spaceId))
}

/**
 * The notice shown when the server refused a change made on the phone.
 *
 * @param edit The refused change.
 * @param card The changed card as the phone had it, or null.
 * @return One sentence naming the space, or the card by its title when it has one.
 */
fun refusedNotice(edit: Edit, card: Card?): String = when (edit) {
    is Edit.CreateSpace -> "Your new space “${edit.name}” could not be saved."
    is Edit.DeleteSpace -> "A space could not be deleted."
    is Edit.CreateCard -> "Something you shared could not be saved."
    else -> card?.title?.takeIf { it.isNotBlank() }?.let { "Your change to “$it” could not be saved." } ?: "A change to a card could not be saved."
}

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
     * @return True once this copy's server has taken a push, so it keeps the ids the phone makes, as for a new space;
     *   false before, or when the copy cannot be read.
     */
    suspend fun pushWorks(): Boolean = copy.pushWorks()

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
            copy.markPushWorks()
            batch.forEach { (seq, edit) -> if (edit is Edit.AddTag) answer.tagIds[seq.toString()]?.let { sentTags[edit.localId] = it } }
            if (!copy.unqueueThrough(last)) return ApiFailure.Server(0)
            for ((seq, edit) in batch) {
                when {
                    seq > last -> {}
                    seq in answer.refused -> refuse(edit)
                    // A link already saved keeps its own id, and a change already handled answers none: the phone's
                    // card makes way for the server's.
                    edit is Edit.CreateCard && answer.cardIds[seq.toString()] != edit.cardId -> reread(edit.cardId)
                }
            }
            if (last < batch.last().first) return ApiFailure.Server(0)
        }
    }

    // Puts the server's version of a refused change's card in the copy and tells [onRefused]. A visit to a card that is
    // gone needs neither.
    private suspend fun refuse(edit: Edit) {
        if (edit is Edit.Visit) return
        val card = edit.cardId?.let { copy.card(it) }
        edit.cardId?.let { reread(it) }
        onRefused(edit, card)
    }

    // Puts the server's version of a card in the copy, or takes the card out when the server has none.
    private suspend fun reread(id: String) {
        when (val fresh = server.card(id)) {
            is ApiResult.Ok -> copy.replace(id, fresh.value)
            is ApiResult.Failed -> if (fresh.failure == ApiFailure.Server(404)) copy.replace(id, null)
        }
    }

    private suspend fun deliver(edit: Edit): ApiResult<Unit> = when (edit) {
        is Edit.Pin -> server.setPinned(edit.cardId, edit.pinned)
        is Edit.Rotate -> server.setRotation(edit.cardId, edit.rotation)
        is Edit.Trash -> server.trash(edit.cardId)
        is Edit.Note -> server.setNote(edit.cardId, edit.text)
        is Edit.AddTag -> when (val answer = server.addTag(edit.cardId, edit.name)) {
            is ApiResult.Ok -> ApiResult.Ok(Unit).also { answer.value.id?.let { sentTags[edit.localId] = it } }
            is ApiResult.Failed -> answer
        }
        is Edit.RemoveTag -> server.removeTag(edit.tagId)
        is Edit.AddToSpace -> server.addToSpace(edit.spaceId, edit.cardId)
        is Edit.RemoveFromSpace -> server.removeFromSpace(edit.spaceId, edit.cardId)
        // Only the push keeps the id the phone made, so a server without it cannot take the space or card.
        is Edit.CreateSpace, is Edit.CreateCard -> ApiResult.Failed(ApiFailure.Invalid)
        // A server without the push keeps no sky.
        is Edit.Visit -> ApiResult.Ok(Unit)
        is Edit.DeleteSpace -> server.deleteSpace(edit.spaceId)
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

package com.octaviantocan.mymind.data

import kotlinx.coroutines.withTimeoutOrNull

private const val WRITE_SYNC_WAIT_MS = 5_000L

/**
 * The library the screens use. Once the phone's copy holds the whole library and the server still feeds it changes,
 * the gallery, Top of Mind, cards, the spaces and space pages come from the copy without waiting on the network; space
 * pages only when the copy's cards carry their place in the space, which older servers leave out. Everything else
 * comes from the server, and the copy answers search, cards, spaces and space pages too when the network is down.
 *
 * Each write goes to the server; once it succeeds, a sync brings the copy up to date before the write returns (for at
 * most a few seconds), so a list read from the copy right after a pin or a trash shows it. A change to a card (pin,
 * trash, note, tags, spaces) made without network, or while earlier ones still wait, joins the [outbox] instead and
 * reads as done: the copy shows it at once and the server gets it, in order, once it can be reached.
 *
 * @param server The server's library.
 * @param copy The phone's copy.
 * @param sync Keeps the copy up to date, sending the outbox first.
 * @param outbox Changes waiting for the server.
 */
class LocalFirstLibrary(
    private val server: Library,
    private val copy: LibraryCopy,
    private val sync: LibrarySync,
    private val outbox: Outbox,
) : Library by server {
    override suspend fun cards(cursor: String?): ApiResult<CardPage> = when {
        cursor != null && cursor.startsWith(LOCAL_CURSOR) -> copy.page(cursor)?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Server(0))
        cursor == null && wholeCopy() -> copy.page(null)?.let { ApiResult.Ok(it) } ?: server.cards(null)
        else -> server.cards(cursor)
    }

    override suspend fun topOfMind(cursor: String?): ApiResult<CardPage> =
        (if (cursor == null && wholeCopy()) copy.pinned() else null)?.let { ApiResult.Ok(it) } ?: server.topOfMind(cursor)

    override suspend fun search(query: String, cursor: String?): ApiResult<CardPage> =
        offline(server.search(query, cursor)) { if (cursor == null) copy.search(query) else null }

    override suspend fun card(cardId: String): ApiResult<Card> =
        (if (wholeCopy()) copy.card(cardId) else null)?.let { ApiResult.Ok(it) } ?: offline(server.card(cardId)) { copy.card(cardId) }

    override suspend fun spaces(): ApiResult<SpaceList> =
        (if (wholeCopy()) copy.spaces() else null)?.let { ApiResult.Ok(SpaceList(it)) } ?: offline(server.spaces()) { copy.spaces()?.let(::SpaceList) }

    override suspend fun spaceCards(spaceId: String, cursor: String?): ApiResult<CardPage> {
        if (cursor == null && wholeCopy()) {
            copy.spaceCards(spaceId)?.takeIf { page -> page.cards.all { spaceId in it.spaceRanks.orEmpty() } }?.let { return ApiResult.Ok(it) }
        }
        return offline(server.spaceCards(spaceId, cursor)) { if (cursor == null) copy.spaceCards(spaceId) else null }
    }

    override suspend fun addTag(cardId: String, name: String): ApiResult<Tag> {
        val localId = outbox.newTagId()
        return change(Edit.AddTag(cardId, name, localId), Tag(localId, name)) { server.addTag(cardId, name) }
    }

    override suspend fun removeTag(tagId: String): ApiResult<Unit> {
        if (!tagId.startsWith(LOCAL_TAG)) return change(Edit.RemoveTag(null, tagId), Unit) { server.removeTag(tagId) }
        val sent = outbox.takeBackTag(tagId) ?: return ApiResult.Ok(Unit)
        return if (sent.startsWith(LOCAL_TAG)) ApiResult.Failed(ApiFailure.Invalid) else removeTag(sent)
    }

    override suspend fun addToSpace(spaceId: String, cardId: String): ApiResult<Unit> =
        change(Edit.AddToSpace(cardId, spaceId), Unit) { server.addToSpace(spaceId, cardId) }

    override suspend fun createSpace(name: String, color: String): ApiResult<String> = written(server.createSpace(name, color))

    override suspend fun deleteSpace(spaceId: String): ApiResult<Unit> = written(server.deleteSpace(spaceId))

    override suspend fun setPinned(cardId: String, pinned: Boolean): ApiResult<Unit> =
        change(Edit.Pin(cardId, pinned), Unit) { server.setPinned(cardId, pinned) }

    override suspend fun trash(cardId: String): ApiResult<Unit> = change(Edit.Trash(cardId), Unit) { server.trash(cardId) }

    override suspend fun removeFromSpace(spaceId: String, cardId: String): ApiResult<Unit> =
        change(Edit.RemoveFromSpace(cardId, spaceId), Unit) { server.removeFromSpace(spaceId, cardId) }

    override suspend fun createMedia(uploadId: String, spaceId: String?): ApiResult<String> = written(server.createMedia(uploadId, spaceId))

    override suspend fun createFromContent(content: String, source: String, spaceId: String?): ApiResult<String> =
        written(server.createFromContent(content, source, spaceId))

    override suspend fun setNote(cardId: String, text: String): ApiResult<Unit> = change(Edit.Note(cardId, text), Unit) { server.setNote(cardId, text) }

    // A copy kept up to date: whole, and the server still has the changes feed that feeds it.
    private suspend fun wholeCopy() = sync.available && copy.complete()

    // The server's answer, or what the copy holds when the network is down and the copy has it.
    private suspend fun <T> offline(answer: ApiResult<T>, kept: suspend () -> T?): ApiResult<T> =
        if (answer is ApiResult.Failed && answer.failure == ApiFailure.Network) kept()?.let { ApiResult.Ok(it) } ?: answer else answer

    // Sends a change straight to the server while nothing waits and the network works; otherwise queues it behind the
    // others, answers `queued` and starts a sync, which sends the outbox first.
    private suspend fun <T> change(edit: Edit, queued: T, send: suspend () -> ApiResult<T>): ApiResult<T> {
        val answer = if (outbox.isEmpty()) send() else null
        if (answer != null && !(answer is ApiResult.Failed && answer.failure == ApiFailure.Network)) return written(answer)
        if (!outbox.queue(edit)) return answer ?: send()
        sync.start(afterCurrent = true)
        return ApiResult.Ok(queued)
    }

    // After a successful write, a sync that starts after it: awaited while the gallery reads the copy, left running otherwise.
    private suspend fun <T> written(answer: ApiResult<T>): ApiResult<T> {
        if (answer is ApiResult.Ok && sync.available) {
            if (copy.complete()) withTimeoutOrNull(WRITE_SYNC_WAIT_MS) { sync.sync(afterCurrent = true) } else sync.start(afterCurrent = true)
        }
        return answer
    }
}

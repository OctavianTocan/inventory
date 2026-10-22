package com.octaviantocan.mymind.data

import kotlinx.coroutines.withTimeoutOrNull

private const val WRITE_SYNC_WAIT_MS = 5_000L

/**
 * The library the screens use: the gallery and Top of Mind from the phone's copy once it holds the whole library and
 * the server still feeds it changes, the server for everything else, and the copy again for search, cards, spaces and space pages when the network is down.
 *
 * Each write goes to the server; once it succeeds, a sync brings the copy up to date before the write returns (for at
 * most a few seconds), so a list read from the copy right after a pin or a trash shows it.
 *
 * @param server The server's library.
 * @param copy The phone's copy.
 * @param sync Keeps the copy up to date.
 */
class LocalFirstLibrary(private val server: Library, private val copy: LibraryCopy, private val sync: LibrarySync) : Library by server {
    override suspend fun cards(cursor: String?): ApiResult<CardPage> = when {
        cursor != null && cursor.startsWith(LOCAL_CURSOR) -> copy.page(cursor)?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Server(0))
        cursor == null && wholeCopy() -> copy.page(null)?.let { ApiResult.Ok(it) } ?: server.cards(null)
        else -> server.cards(cursor)
    }

    override suspend fun topOfMind(cursor: String?): ApiResult<CardPage> =
        (if (cursor == null && wholeCopy()) copy.pinned() else null)?.let { ApiResult.Ok(it) } ?: server.topOfMind(cursor)

    override suspend fun search(query: String, cursor: String?): ApiResult<CardPage> =
        offline(server.search(query, cursor)) { if (cursor == null) copy.search(query) else null }

    override suspend fun card(cardId: String): ApiResult<Card> = offline(server.card(cardId)) { copy.card(cardId) }

    override suspend fun spaces(): ApiResult<SpaceList> = offline(server.spaces()) { copy.spaces()?.let(::SpaceList) }

    override suspend fun spaceCards(spaceId: String, cursor: String?): ApiResult<CardPage> =
        offline(server.spaceCards(spaceId, cursor)) { if (cursor == null) copy.spaceCards(spaceId) else null }

    override suspend fun addTag(cardId: String, name: String): ApiResult<Tag> = written(server.addTag(cardId, name))

    override suspend fun removeTag(tagId: String): ApiResult<Unit> = written(server.removeTag(tagId))

    override suspend fun addToSpace(spaceId: String, cardId: String): ApiResult<Unit> = written(server.addToSpace(spaceId, cardId))

    override suspend fun createSpace(name: String, color: String): ApiResult<String> = written(server.createSpace(name, color))

    override suspend fun deleteSpace(spaceId: String): ApiResult<Unit> = written(server.deleteSpace(spaceId))

    override suspend fun setPinned(cardId: String, pinned: Boolean): ApiResult<Unit> = written(server.setPinned(cardId, pinned))

    override suspend fun trash(cardId: String): ApiResult<Unit> = written(server.trash(cardId))

    override suspend fun removeFromSpace(spaceId: String, cardId: String): ApiResult<Unit> = written(server.removeFromSpace(spaceId, cardId))

    override suspend fun createMedia(uploadId: String, spaceId: String?): ApiResult<String> = written(server.createMedia(uploadId, spaceId))

    override suspend fun createFromContent(content: String, source: String, spaceId: String?): ApiResult<String> =
        written(server.createFromContent(content, source, spaceId))

    override suspend fun setNote(cardId: String, text: String): ApiResult<Unit> = written(server.setNote(cardId, text))

    // A copy kept up to date: whole, and the server still has the changes feed that feeds it.
    private suspend fun wholeCopy() = sync.available && copy.complete()

    // The server's answer, or what the copy holds when the network is down and the copy has it.
    private suspend fun <T> offline(answer: ApiResult<T>, kept: suspend () -> T?): ApiResult<T> =
        if (answer is ApiResult.Failed && answer.failure == ApiFailure.Network) kept()?.let { ApiResult.Ok(it) } ?: answer else answer

    // After a successful write, a sync that starts after it: awaited while the gallery reads the copy, left running otherwise.
    private suspend fun <T> written(answer: ApiResult<T>): ApiResult<T> {
        if (answer is ApiResult.Ok && sync.available) {
            if (copy.complete()) withTimeoutOrNull(WRITE_SYNC_WAIT_MS) { sync.sync(afterCurrent = true) } else sync.start(afterCurrent = true)
        }
        return answer
    }
}

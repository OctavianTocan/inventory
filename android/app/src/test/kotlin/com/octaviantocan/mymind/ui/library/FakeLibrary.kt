package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.PageConfig
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.SpaceList
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.UploadAuthorization
import com.octaviantocan.mymind.data.UploadBlob
import com.octaviantocan.mymind.data.UploadChunkAnswer
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.data.User
import kotlinx.coroutines.CompletableDeferred
import java.io.ByteArrayOutputStream
import java.io.InputStream

/**
 * A made-up card for tests.
 *
 * @param id Card id.
 * @param type Card type.
 * @param flags Card flags.
 * @param url Source address.
 * @param blob Uploaded file.
 * @param pinPosition Pinned position.
 * @param title Title.
 * @param note Note text.
 * @param tags Tags.
 * @param spaces Ids of the spaces the card is in.
 * @return The card.
 */
fun testCard(
    id: String,
    type: String = "WebPage",
    flags: Int = 0,
    url: String? = null,
    blob: Blob? = null,
    pinPosition: Long? = null,
    title: String? = "Card $id",
    note: String? = null,
    tags: List<Tag>? = null,
    spaces: List<String>? = null,
) = Card(
    id = id,
    type = type,
    flags = flags,
    created = "2026-01-01T00:00:00Z",
    modified = "2026-01-01T00:00:00Z",
    bumped = "2026-01-01T00:00:00Z",
    title = title,
    url = url,
    blob = blob,
    pinPosition = pinPosition,
    note = note,
    tags = tags,
    spaces = spaces,
)

/**
 * An in-memory [Library] that records every call and never talks to a server.
 *
 * @param pages Pages of the library by cursor; a missing cursor fails with a network error.
 * @param spaceList Spaces.
 * @param spacePages First page of each space by id.
 * @param user Account `me` answers with.
 * @param config Page config, or null to fail its read with a network error.
 * @param allTags Every tag of the library.
 * @param recent Recent tags.
 * @param suggestions Search suggestions by query.
 * @param media Bytes of uploaded files by path.
 * @param pinnedPages Pages of Pinned by cursor.
 * @param cardsById Cards `card` answers with; a missing id fails with 404.
 * @param searchHits Cards every search answers with.
 */
class FakeLibrary(
    var pages: Map<String?, CardPage> = mapOf(null to CardPage()),
    var spaceList: List<Space> = emptyList(),
    var spacePages: Map<String, CardPage> = emptyMap(),
    var user: User = User(),
    var config: PageConfig? = PageConfig(),
    var allTags: List<String> = emptyList(),
    var recent: List<String> = emptyList(),
    var suggestions: Map<String, List<Suggestion>> = emptyMap(),
    var media: Map<String, ByteArray> = emptyMap(),
    var pinnedPages: Map<String?, CardPage> = mapOf(null to CardPage()),
    var cardsById: Map<String, Card> = emptyMap(),
    var searchHits: List<Card> = listOf(testCard("hit")),
) : Library {
    /** Calls in the order they arrived, such as `trash:c1`. */
    val calls = mutableListOf<String>()

    /** When set, every write fails with it. */
    var writeFailure: ApiFailure? = null

    /** Holds a successful create-space response until completed, without blocking the test thread. */
    var createSpaceGate: CompletableDeferred<Unit>? = null

    /** When set, upload pieces fail with it. */
    var chunkFailure: ApiFailure? = null

    /** Bytes each upload received, by upload id. */
    val uploaded = mutableMapOf<String, ByteArrayOutputStream>()

    override suspend fun me(): ApiResult<User> = ApiResult.Ok(user).also { calls += "me" }

    override suspend fun cards(cursor: String?): ApiResult<CardPage> {
        calls += "cards:$cursor"
        return pages[cursor]?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Network)
    }

    override suspend fun spaces(): ApiResult<SpaceList> = ApiResult.Ok(SpaceList(spaceList)).also { calls += "spaces" }

    override suspend fun spaceCards(spaceId: String, cursor: String?): ApiResult<CardPage> {
        calls += "spaceCards:$spaceId:$cursor"
        return ApiResult.Ok(spacePages[spaceId] ?: CardPage())
    }

    override suspend fun search(query: String, cursor: String?): ApiResult<CardPage> {
        calls += "search:$query:$cursor"
        return ApiResult.Ok(CardPage(searchHits))
    }

    override suspend fun pageConfig(): ApiResult<PageConfig> {
        calls += "config"
        return config?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Network)
    }

    override suspend fun searchSuggestions(query: String): ApiResult<List<Suggestion>> {
        calls += "suggest:$query"
        return ApiResult.Ok(suggestions[query].orEmpty())
    }

    override suspend fun tagSuggestions(query: String): ApiResult<List<String>> {
        calls += "tagSuggest:$query"
        return ApiResult.Ok(allTags.filter { it.contains(query, ignoreCase = true) })
    }

    override suspend fun tags(): ApiResult<List<String>> = ApiResult.Ok(allTags).also { calls += "tags" }

    override suspend fun recentTags(): ApiResult<List<String>> = ApiResult.Ok(recent).also { calls += "recentTags" }

    override suspend fun addTag(cardId: String, name: String) = write("tag:$cardId:$name") { Tag(id = "t-$name", name = name) }

    override suspend fun removeTag(tagId: String) = write("untag:$tagId") {}

    override suspend fun addToSpace(spaceId: String, cardId: String) = write("space:$spaceId:$cardId") {}

    override suspend fun createSpace(name: String, color: String): ApiResult<String> {
        val answer = write("newSpace:$name:$color") { "s-new" }
        if (answer is ApiResult.Ok) createSpaceGate?.await()
        return answer
    }

    override suspend fun deleteSpace(spaceId: String) = write("deleteSpace:$spaceId") {}

    override suspend fun renameSpace(spaceId: String, name: String) = write("renameSpace:$spaceId:$name") {}

    override suspend fun media(path: String, sink: (InputStream) -> Unit): ApiResult<Unit> {
        calls += "media:$path"
        val bytes = media[path] ?: return ApiResult.Failed(ApiFailure.Server(404))
        sink(bytes.inputStream())
        return ApiResult.Ok(Unit)
    }

    override suspend fun setPinned(cardId: String, pinned: Boolean) = write("pin:$cardId:$pinned") {}

    override suspend fun setRotation(cardId: String, rotation: Int) = write("rotate:$cardId:$rotation") {}

    override suspend fun trash(cardId: String) = write("trash:$cardId") {}

    override suspend fun removeFromSpace(spaceId: String, cardId: String) = write("unspace:$spaceId:$cardId") {}

    override suspend fun pinned(cursor: String?): ApiResult<CardPage> {
        calls += "pinned:$cursor"
        return pinnedPages[cursor]?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Network)
    }

    override suspend fun card(cardId: String): ApiResult<Card> {
        calls += "card:$cardId"
        return cardsById[cardId]?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Server(404))
    }

    override suspend fun authorizeUploads(count: Int) =
        write("authorize:$count") { (1..count).map { UploadAuthorization("/uploads/u${authorized + it}", "token${authorized + it}") } }
            .also { if (it is ApiResult.Ok) authorized += count }

    override suspend fun uploadChunk(
        authorization: UploadAuthorization,
        uploadId: String?,
        file: UploadFile,
        start: Long,
        length: Long,
        source: InputStream,
    ): ApiResult<UploadChunkAnswer> {
        val id = authorization.url.substringAfterLast('/')
        calls += "chunk:${authorization.token}:$uploadId:${file.name}:$start+$length/${file.size}"
        chunkFailure?.let { return ApiResult.Failed(it) }
        val bytes = ByteArray(length.toInt())
        var read = 0
        while (read < bytes.size) read += source.read(bytes, read, bytes.size - read).also { check(it > 0) }
        uploaded.getOrPut(id) { ByteArrayOutputStream() }.write(bytes)
        val done = start + length == file.size
        return ApiResult.Ok(UploadChunkAnswer(id, blob = UploadBlob(id), received = (start + length).toDouble(), done = done))
    }

    override suspend fun createMedia(uploadId: String, spaceId: String?) = write("media:$uploadId:$spaceId") { "card-$uploadId" }

    override suspend fun createFromContent(content: String, source: String, spaceId: String?) =
        write("content:$content:$source:$spaceId") { "card-${content.hashCode()}" }

    override suspend fun setNote(cardId: String, text: String) = write("note:$cardId:$text") {}

    private var authorized = 0

    private fun <T> write(call: String, value: () -> T): ApiResult<T> {
        calls += call
        return writeFailure?.let { ApiResult.Failed(it) } ?: ApiResult.Ok(value())
    }
}

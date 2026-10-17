package com.octaviantocan.mymind.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.addJsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.encodeToString
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.Interceptor
import okhttp3.MediaType
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import okio.BufferedSink
import java.io.IOException
import java.io.InputStream
import java.time.Duration

/**
 * Adds the bearer token and signs the session out when the server rejects a token-bearing request with 401. An upload
 * piece refused with 401 leaves the session alone, since that answer is about its upload token.
 */
class AuthInterceptor(private val session: Session) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        val token = session.token ?: return chain.proceed(chain.request())
        val request = chain.request()
        val response = chain.proceed(request.newBuilder().header("Authorization", "Bearer $token").build())
        if (response.code == 401 && request.header(UPLOAD_TOKEN_HEADER) == null) session.invalidate(token)
        return response
    }
}

private const val UPLOAD_TOKEN_HEADER = "x-upload-token"

/** Why a call failed, in terms the UI can word. */
sealed interface ApiFailure {
    data object NotAllowed : ApiFailure
    data object Invalid : ApiFailure
    data object Locked : ApiFailure
    data object InvalidToken : ApiFailure
    data object Unauthorized : ApiFailure
    data object Network : ApiFailure
    data class Server(val code: Int) : ApiFailure
}

sealed interface ApiResult<out T> {
    data class Ok<T>(val value: T) : ApiResult<T>
    data class Failed(val failure: ApiFailure) : ApiResult<Nothing>
}

/**
 * Turns a success into another value and keeps a failure as it is.
 *
 * @param transform Maps the success value.
 * @return The mapped result.
 */
inline fun <T, R> ApiResult<T>.map(transform: (T) -> R): ApiResult<R> = when (this) {
    is ApiResult.Ok -> ApiResult.Ok(transform(value))
    is ApiResult.Failed -> this
}

/** Client for the `/app/v1` server contract. */
class MymindApi(private val client: OkHttpClient, private val server: String) {
    // On a slow uplink the socket buffers still hold many seconds of a piece when the last write returns, and the
    // server answers only after the last byte, so the default 10 s wait for the answer drops slow uploads.
    private val uploadClient = client.newBuilder().readTimeout(UPLOAD_ANSWER_TIMEOUT).build()

    suspend fun nonce(): ApiResult<NonceAnswer> =
        send(post("/app/v1/auth/nonce", null), NonceAnswer.serializer())

    suspend fun signInWithGoogle(idToken: String, nonce: String, device: String): ApiResult<TokenAnswer> =
        send(
            post("/app/v1/auth/google", AppJson.encodeToString(GoogleSignInRequest(idToken, nonce, device))),
            TokenAnswer.serializer(),
        )

    suspend fun signInWithPassword(username: String, password: String, device: String): ApiResult<TokenAnswer> =
        send(
            post("/app/v1/auth/password", AppJson.encodeToString(PasswordSignInRequest(username, password, device))),
            TokenAnswer.serializer(),
        )

    suspend fun signOut(): ApiResult<Unit> =
        withContext(Dispatchers.IO) { execute(post("/app/v1/auth/signout", null)) { ApiResult.Ok(Unit) } }

    /** @return The account of the signed-in session. */
    suspend fun me(): ApiResult<User> =
        send(Request.Builder().url(server.trimEnd('/') + "/app/v1/me").get().build(), User.serializer())

    /**
     * @param limit page size, 1 to 200.
     * @param cursor `next` of the previous page, or null for the first page.
     */
    suspend fun cards(limit: Int, cursor: String?): ApiResult<CardPage> {
        val url = server.trimEnd('/').toHttpUrl().newBuilder()
            .addPathSegments("app/v1/cards")
            .addQueryParameter("limit", limit.toString())
            .apply { if (cursor != null) addQueryParameter("cursor", cursor) }
            .build()
        return send(Request.Builder().url(url).get().build(), CardPage.serializer())
    }

    /**
     * @param id Card id.
     * @return The card as the server has it now.
     */
    suspend fun card(id: String): ApiResult<Card> = send(get("app/v1/cards/$id"), Card.serializer())

    /** @return The spaces in the user's order. */
    suspend fun spaces(): ApiResult<SpaceList> = send(get("app/v1/spaces"), SpaceList.serializer())

    /**
     * @param id Space id.
     * @param limit page size, 1 to 200.
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of the space's cards in the space's order.
     */
    suspend fun spaceCards(id: String, limit: Int, cursor: String?): ApiResult<CardPage> =
        send(get("app/v1/spaces/$id/cards", "limit" to limit.toString(), "cursor" to cursor), CardPage.serializer())

    /**
     * @param limit page size, 1 to 200.
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of the pinned cards in pin order.
     */
    suspend fun topOfMind(limit: Int, cursor: String?): ApiResult<CardPage> =
        send(get("app/v1/top-of-mind", "limit" to limit.toString(), "cursor" to cursor), CardPage.serializer())

    /**
     * Reads the Serendipity deck from the partial web page, which carries its cards as `mymind-card` elements.
     *
     * @return Card ids of the deck in its order, or the failure.
     */
    suspend fun serendipityIds(): ApiResult<List<String>> = withContext(Dispatchers.IO) {
        // The document request gets the page shell without the deck; the web's deck script asks for the partial.
        val request = Request.Builder().url(url("serendipity")).header("Accept", "text/html").header("x-partial", "1").get().build()
        execute(request) { ApiResult.Ok(serendipityIdsOf(it)) }
    }

    /**
     * Asks for permission to upload files.
     *
     * @param count Number of files, 1 to 100.
     * @return One authorization per file.
     */
    suspend fun authorizeUploads(count: Int): ApiResult<List<UploadAuthorization>> =
        send(
            Request.Builder().url(url("uploads/authorize", "count" to count.toString())).post(ByteArray(0).toRequestBody()).build(),
            ListSerializer(UploadAuthorization.serializer()),
        )

    /**
     * Sends one piece of a file with the web upload headers; the app token stays in `Authorization`.
     *
     * @param authorization The file's authorization from [authorizeUploads].
     * @param uploadId Id from the first piece's answer, or null for the first piece.
     * @param name File name.
     * @param mime File media type.
     * @param total Length of the whole file in bytes.
     * @param start Offset of the piece's first byte in the file.
     * @param length Bytes in the piece, at most 32 MiB.
     * @param source Stream at [start]; exactly [length] bytes are read from it and it is left open.
     * @return The server's answer, or the failure.
     */
    suspend fun uploadChunk(
        authorization: UploadAuthorization,
        uploadId: String?,
        name: String,
        mime: String,
        total: Long,
        start: Long,
        length: Long,
        source: InputStream,
    ): ApiResult<UploadChunkAnswer> {
        val request = Request.Builder()
            .url(server.trimEnd('/') + "/" + authorization.url.trimStart('/'))
            .header("Content-Range", "bytes $start-${start + length - 1}/$total")
            .header("x-file-name", encodeUri(name))
            .header(UPLOAD_TOKEN_HEADER, authorization.token)
            .apply { if (uploadId != null) header("x-upload-id", uploadId) }
            .put(StreamBody(mime.toMediaTypeOrNull() ?: OCTET_STREAM, source, length))
            .build()
        return send(request, UploadChunkAnswer.serializer(), uploadClient)
    }

    /**
     * Creates a Media card from a finished upload.
     *
     * @param uploadId Blob id of the finished upload.
     * @param spaceId Space the card joins, or null.
     * @return The new card's id.
     */
    suspend fun createMedia(uploadId: String, spaceId: String?): ApiResult<String> =
        send(
            Request.Builder().url(url("objects"))
                .post(AppJson.encodeToString(MediaBody("Media", uploadId, spaceId)).toRequestBody(JSON_TYPE)).build(),
            IdAnswer.serializer(),
        ).map { it.id }

    /**
     * Saves a link or a piece of text as a card; a link already in the library is bumped instead of duplicated.
     *
     * @param content The link or the text.
     * @param source Where the card came from.
     * @param spaceId Space the new card joins, or null.
     * @return The card's id, whether the card is new (201) or was there already (200).
     */
    suspend fun createFromContent(content: String, source: String, spaceId: String?): ApiResult<String> =
        send(
            Request.Builder().url(url("objects"))
                .post(AppJson.encodeToString(ContentBody(content, source, spaceId)).toRequestBody(JSON_TYPE)).build(),
            IdAnswer.serializer(),
        ).map { it.id }

    /**
     * Sets a card's note.
     *
     * @param cardId Card id.
     * @param text Note text; each line becomes a paragraph.
     * @return Success, or the failure.
     */
    suspend fun setNote(cardId: String, text: String): ApiResult<Unit> =
        send(
            Request.Builder().url(url("objects/$cardId/note")).put(proseOf(text).toRequestBody(JSON_TYPE)).build(),
            IdAnswer.serializer(),
        ).map { }

    /**
     * @param query Search text in mymind's query syntax.
     * @param limit page size, 1 to 200.
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of matching cards.
     */
    suspend fun search(query: String, limit: Int, cursor: String?): ApiResult<CardPage> =
        send(get("app/v1/search", "q" to query, "limit" to limit.toString(), "cursor" to cursor), CardPage.serializer())

    /**
     * Reads the account settings and feature switches that the web page carries in its config script.
     *
     * @return The config, or the failure; a page without a config script gives an empty one.
     */
    suspend fun pageConfig(): ApiResult<PageConfig> = withContext(Dispatchers.IO) {
        execute(documentRequest("everything")) { ApiResult.Ok(pageConfigOf(it)) }
    }

    /**
     * @param query Typed search text, or empty for the card types of the library.
     * @return Card types and tags that match, best first.
     */
    suspend fun searchSuggestions(query: String): ApiResult<List<Suggestion>> =
        send(get("search/suggestions", "q" to query.ifEmpty { null }), ListSerializer(Suggestion.serializer()))

    /**
     * @param query Text a tag name contains.
     * @return Matching tags, most used first.
     */
    suspend fun tagSuggestions(query: String): ApiResult<List<TagName>> =
        send(get("tags/suggest", "q" to query), ListSerializer(TagName.serializer()))

    /** @return Every tag the user added to a card, with its card count. */
    suspend fun tags(): ApiResult<TagList> = send(get("app/v1/tags"), TagList.serializer())

    /** @return The tags the user added last, newest first. */
    suspend fun recentTags(): ApiResult<List<TagName>> =
        send(get("tags/recent", "take" to "5"), ListSerializer(TagName.serializer()))

    /**
     * Adds a tag to a card; a tag the card already has is answered as it is.
     *
     * @param cardId Card id.
     * @param name Tag name.
     * @return The tag with the id that [removeTag] takes.
     */
    suspend fun addTag(cardId: String, name: String): ApiResult<Tag> =
        send(
            Request.Builder().url(url("objects/$cardId/tags")).post(AppJson.encodeToString(NameBody(name)).toRequestBody(JSON_TYPE)).build(),
            Tag.serializer(),
        )

    /**
     * Takes a tag off its card.
     *
     * @param tagId Tag id from the card's tags.
     * @return Success, or the failure.
     */
    suspend fun removeTag(tagId: String): ApiResult<Unit> = write(Request.Builder().url(url("annotations/$tagId")).delete())

    /**
     * Puts a card into a space.
     *
     * @param spaceId Space id.
     * @param cardId Card id.
     * @return Success, or the failure.
     */
    suspend fun addToSpace(spaceId: String, cardId: String): ApiResult<Unit> =
        write(Request.Builder().url(url("spaces/$spaceId/objects/$cardId")).put("{}".toRequestBody(JSON_TYPE)))

    /**
     * Creates an empty space at the end of the list.
     *
     * @param name Space name.
     * @param color Space colour as a CSS hex colour, such as `#f96` or `#75ffc0`.
     * @return The new space's id.
     */
    suspend fun createSpace(name: String, color: String): ApiResult<String> =
        send(
            Request.Builder().url(url("spaces")).post(AppJson.encodeToString(NewSpaceBody(name, color)).toRequestBody(JSON_TYPE)).build(),
            IdAnswer.serializer(),
        ).map { it.id }

    /**
     * Deletes a space; its cards stay in the library.
     *
     * @param id Space id.
     * @return Success, or the failure.
     */
    suspend fun deleteSpace(id: String): ApiResult<Unit> = write(Request.Builder().url(url("spaces/$id")).delete())

    /**
     * Streams an uploaded file as it was saved.
     *
     * @param path The card's `blob.path`.
     * @param sink Receives the file's bytes; it runs on the I/O dispatcher and must not keep the stream.
     * @return Success once [sink] returned, or the failure.
     */
    suspend fun media(path: String, sink: (InputStream) -> Unit): ApiResult<Unit> = withContext(Dispatchers.IO) {
        try {
            client.newCall(get("media/${path.trimStart('/')}")).execute().use { response ->
                if (response.isSuccessful) ApiResult.Ok(sink(response.body.byteStream()))
                else ApiResult.Failed(failureOf(response.code, response.body.string()))
            }
        } catch (_: IOException) {
            ApiResult.Failed(ApiFailure.Network)
        }
    }

    /**
     * Pins the card to Top of Mind or takes it off.
     *
     * @param id Card id.
     * @param pinned True to pin, false to unpin.
     * @return Success, or the failure.
     */
    suspend fun setPinned(id: String, pinned: Boolean): ApiResult<Unit> =
        write(Request.Builder().url(url("objects/$id/${if (pinned) "pin" else "unpin"}")).post("{}".toRequestBody(JSON_TYPE)))

    /**
     * Moves the card to the trash.
     *
     * @param id Card id.
     * @return Success, or the failure.
     */
    suspend fun trash(id: String): ApiResult<Unit> = write(Request.Builder().url(url("objects/$id")).delete())

    /**
     * Takes a card out of a space; the card itself stays in the library.
     *
     * @param spaceId Space id.
     * @param cardId Card id.
     * @return Success, or the failure.
     */
    suspend fun removeFromSpace(spaceId: String, cardId: String): ApiResult<Unit> =
        write(Request.Builder().url(url("spaces/$spaceId/objects/$cardId")).delete())

    /**
     * Tells the server the user went through a whole Serendipity deck.
     *
     * @return Success, or the failure.
     */
    suspend fun completeSerendipity(): ApiResult<Unit> =
        write(Request.Builder().url(url("serendipity/complete")).post("{}".toRequestBody(JSON_TYPE)))

    private fun url(path: String, vararg query: Pair<String, String?>) = server.trimEnd('/').toHttpUrl().newBuilder()
        .addPathSegments(path)
        .apply { query.forEach { (name, value) -> if (value != null) addQueryParameter(name, value) } }
        .build()

    private fun get(path: String, vararg query: Pair<String, String?>): Request =
        Request.Builder().url(url(path, *query)).get().build()

    private fun documentRequest(path: String): Request = Request.Builder().url(url(path))
        .header("Accept", "text/html")
        .header("Sec-Fetch-Dest", "document")
        .get()
        .build()

    private suspend fun write(request: Request.Builder): ApiResult<Unit> =
        withContext(Dispatchers.IO) { execute(request.build()) { ApiResult.Ok(Unit) } }

    private fun post(path: String, json: String?): Request = Request.Builder()
        .url(server.trimEnd('/') + path)
        .post((json ?: "").toRequestBody(JSON_TYPE))
        .build()

    private suspend fun <T> send(request: Request, serializer: KSerializer<T>, http: OkHttpClient = client): ApiResult<T> =
        withContext(Dispatchers.IO) {
            execute(request, http) { body -> ApiResult.Ok(AppJson.decodeFromString(serializer, body)) }
        }

    private fun <T> execute(request: Request, http: OkHttpClient = client, onOk: (String) -> ApiResult<T>): ApiResult<T> = try {
        http.newCall(request).execute().use { response ->
            val body = response.body.string()
            if (response.isSuccessful) onOk(body) else ApiResult.Failed(failureOf(response.code, body))
        }
    } catch (_: IOException) {
        ApiResult.Failed(ApiFailure.Network)
    } catch (_: IllegalArgumentException) {
        ApiResult.Failed(ApiFailure.Server(0))
    }

    private companion object {
        val JSON_TYPE = "application/json".toMediaType()
        val OCTET_STREAM = "application/octet-stream".toMediaType()
        val UPLOAD_ANSWER_TIMEOUT: Duration = Duration.ofMinutes(5)
    }
}

/** A request body of the next [length] bytes of [source], sent once. */
private class StreamBody(private val type: MediaType, private val source: InputStream, private val length: Long) : RequestBody() {
    override fun contentType() = type

    override fun contentLength() = length

    override fun isOneShot() = true

    override fun writeTo(sink: BufferedSink) {
        val buffer = ByteArray(STREAM_BUFFER)
        var left = length
        while (left > 0) {
            val read = source.read(buffer, 0, minOf(buffer.size.toLong(), left).toInt())
            if (read < 0) throw IOException("The file ended early")
            sink.write(buffer, 0, read)
            left -= read
        }
    }
}

private const val STREAM_BUFFER = 64 * 1024

/** [text] as JavaScript's `encodeURI` writes it: UTF-8 percent escapes for everything but the URI characters. */
private fun encodeUri(text: String): String = buildString {
    for (byte in text.toByteArray(Charsets.UTF_8)) {
        val char = (byte.toInt() and 0xFF).toChar()
        if (char.code < 0x80 && (char.isLetterOrDigit() || char in URI_KEPT)) append(char) else append("%%%02X".format(byte.toInt() and 0xFF))
    }
}

private const val URI_KEPT = ";,/?:@&=+$-_.!~*'()#"

/** Maps an error response to a failure; [body] is the raw response text. */
fun failureOf(code: Int, body: String): ApiFailure {
    val error = runCatching { AppJson.decodeFromString(ErrorAnswer.serializer(), body).error }.getOrNull()
    return when {
        error == "not-allowed" -> ApiFailure.NotAllowed
        error == "invalid" -> ApiFailure.Invalid
        error == "locked" || code == 429 -> ApiFailure.Locked
        error == "invalid-token" -> ApiFailure.InvalidToken
        code == 401 -> ApiFailure.Unauthorized
        else -> ApiFailure.Server(code)
    }
}

/** Prose document with one paragraph per line of [text]. */
internal fun proseOf(text: String): String = buildJsonObject {
    put("type", "doc")
    putJsonArray("content") {
        text.lines().forEach { line ->
            addJsonObject {
                put("type", "paragraph")
                if (line.isNotEmpty()) putJsonArray("content") { addJsonObject { put("type", "text"); put("text", line) } }
            }
        }
    }
}.toString()

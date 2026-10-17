package com.octaviantocan.mymind.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.KSerializer
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.encodeToString
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import java.io.IOException
import java.io.InputStream

/** Adds the bearer token and signs the session out when the server rejects a token-bearing request with 401. */
class AuthInterceptor(private val session: Session) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        val token = session.token ?: return chain.proceed(chain.request())
        val response = chain.proceed(
            chain.request().newBuilder().header("Authorization", "Bearer $token").build(),
        )
        if (response.code == 401) session.invalidate(token)
        return response
    }
}

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
        val request = Request.Builder().url(url("everything"))
            .header("Accept", "text/html")
            .header("Sec-Fetch-Dest", "document")
            .get()
            .build()
        execute(request) { ApiResult.Ok(pageConfigOf(it)) }
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

    private suspend fun write(request: Request.Builder): ApiResult<Unit> =
        withContext(Dispatchers.IO) { execute(request.build()) { ApiResult.Ok(Unit) } }

    private fun post(path: String, json: String?): Request = Request.Builder()
        .url(server.trimEnd('/') + path)
        .post((json ?: "").toRequestBody(JSON_TYPE))
        .build()

    private suspend fun <T> send(request: Request, serializer: KSerializer<T>): ApiResult<T> =
        withContext(Dispatchers.IO) {
            execute(request) { body -> ApiResult.Ok(AppJson.decodeFromString(serializer, body)) }
        }

    private fun <T> execute(request: Request, onOk: (String) -> ApiResult<T>): ApiResult<T> = try {
        client.newCall(request).execute().use { response ->
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
    }
}

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

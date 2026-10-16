package com.octaviantocan.mymind.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.KSerializer
import kotlinx.serialization.encodeToString
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import java.io.IOException

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

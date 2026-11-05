package com.octaviantocan.mymind.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException

/**
 * A failed feedback call, in the server's own sentence when it gave one.
 *
 * @param message What to tell the person.
 * @property notFound True when the server no longer knows the job (404), so it need not be asked about again.
 */
class FeedbackProblem(message: String, val notFound: Boolean = false) : Exception(message)

/**
 * Where a feedback job stands, as the server describes it.
 *
 * @property id The job's id; empty only in a body the server sent without one.
 * @property state One of the job states, such as `working` or `shipped`.
 * @property title The notification's first line; empty from a send, which answers only [id] and [state].
 * @property detail The notification's second line, or empty.
 * @property version The app version a `shipped` job went out in, or null.
 */
@Serializable
data class FeedbackStatus(
    val id: String = "",
    val state: String,
    val title: String = "",
    val detail: String = "",
    val version: String? = null,
)

/** The states after which a job changes no more: the phone stops asking about it. */
val FINAL_FEEDBACK_STATES = setOf("shipped", "merged_server", "merged_not_released", "no_change", "failed")

/** True when a job in this state has ended. */
val FeedbackStatus.isFinal: Boolean get() = state in FINAL_FEEDBACK_STATES

/** What the phone asks of the feedback routes, so the page, the worker and the tests can use another answerer. */
interface FeedbackApi {
    /**
     * Sends a feedback for the runner to implement.
     *
     * @param text The feedback, 1 to 4000 characters.
     * @param choice The signed-in engine, model and effort to implement it with.
     * @param clientId Random id of this send attempt; a retried send with the same id returns the same job.
     * @return The new (or already known) job: its id and state.
     * @throws FeedbackProblem When the server refuses or cannot be reached.
     */
    suspend fun send(text: String, choice: AiChoice, clientId: String): FeedbackStatus

    /**
     * @param id A job id from [send].
     * @return The job's state with the notification's title and detail.
     * @throws FeedbackProblem When the server refuses or cannot be reached.
     */
    suspend fun status(id: String): FeedbackStatus
}

/**
 * Client for the server's feedback routes (`/app/v1/feedback`), built like [AiClient]: the library's client adds the
 * session. A refusal (403) or an unknown job (404) becomes a [FeedbackProblem] with the server's sentence, and is
 * never a sign-out.
 *
 * @param client The library's client, whose interceptor adds the session.
 * @param server The library server's address.
 */
class FeedbackClient(private val client: OkHttpClient, private val server: String) : FeedbackApi {
    override suspend fun send(text: String, choice: AiChoice, clientId: String): FeedbackStatus {
        val body = buildJsonObject {
            put("text", text)
            put("engine", choice.engine)
            put("model", choice.model)
            put("effort", choice.effort)
            put("client_id", clientId)
        }
        val sent = call(Request.Builder().url(url()).post(body.toString().toRequestBody(JSON_TYPE)).build())
        return AppJson.decodeFromJsonElement(FeedbackStatus.serializer(), sent)
    }

    override suspend fun status(id: String): FeedbackStatus {
        val url = server.trimEnd('/').toHttpUrl().newBuilder().addPathSegments("app/v1/feedback").addPathSegment(id).build()
        return AppJson.decodeFromJsonElement(FeedbackStatus.serializer(), call(Request.Builder().url(url).get().build())).copy(id = id)
    }

    private fun url() = server.trimEnd('/').toHttpUrl().newBuilder().addPathSegments("app/v1/feedback").build()

    private suspend fun call(request: Request): JsonObject = withContext(Dispatchers.IO) {
        val (code, text) = try {
            client.newCall(request).execute().use { it.code to it.body.string() }
        } catch (_: IOException) {
            throw FeedbackProblem("No connection to Inventory. Try again when you are online.")
        }
        val json = runCatching { AppJson.parseToJsonElement(text).jsonObject }.getOrNull()
        if (code !in 200..299) {
            val said = (json?.get("error") as? JsonPrimitive)?.contentOrNull?.takeIf { it.isNotBlank() }
            throw FeedbackProblem(said ?: "Inventory could not answer ($code). Try again.", notFound = code == 404)
        }
        json ?: throw FeedbackProblem("Inventory sent an answer the app cannot read.")
    }

    private companion object {
        val JSON_TYPE = "application/json".toMediaType()
    }
}

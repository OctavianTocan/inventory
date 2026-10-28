package com.octaviantocan.mymind.data

import com.octaviantocan.aikit.AiCli
import com.octaviantocan.aikit.AiSignInStart
import com.octaviantocan.aikit.AiSignIns
import com.octaviantocan.aikit.CliCatalog
import com.octaviantocan.aikit.asAiSignInFinished
import com.octaviantocan.aikit.asAiSignInStart
import com.octaviantocan.aikit.asAiSignIns
import com.octaviantocan.aikit.asCatalog
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
import java.time.Duration

/** A failed AI call, in the server's own words when it gave some. */
class AiProblem(message: String) : Exception(message)

/**
 * The cards an AI search answered: the ones the model ranked first, best first, then the rest of the candidates.
 *
 * @property cards The cards in that order.
 * @property ranked How many of them the model ranked.
 */
@Serializable
data class AiSearchAnswer(val cards: List<Card> = emptyList(), val ranked: Int = 0)

/**
 * The engine, model and effort an AI search runs with.
 *
 * @property engine `claude-code` or `codex`.
 * @property model A model id from the CLI's list, or empty for the CLI's own default.
 * @property effort An effort the model lists, or empty for the model's default.
 */
data class AiChoice(val engine: String = CLAUDE_CODE, val model: String = "", val effort: String = "")

/** Engine id of Claude Code. */
const val CLAUDE_CODE = "claude-code"

/** Engine id of Codex. */
const val CODEX = "codex"

/** The engines AI search can use, in the order the model page shows them. */
val AI_ENGINES = listOf(CLAUDE_CODE, CODEX)

/** The CLI an engine id runs. */
val String.cli: AiCli get() = if (this == CODEX) AiCli.Codex else AiCli.Claude

/**
 * Client for the server's Claude Code and Codex routes (`/app/v1/ai/`): the account's own sign-ins, the CLIs' model
 * lists and AI search. Each call throws [AiProblem] or [IOException] when it fails, as the AI kit's pages expect.
 *
 * @param client The library's client, whose interceptor adds the session.
 * @param server The library server's address.
 */
class AiClient(client: OkHttpClient, private val server: String) {
    // A Codex sign-in waits up to 25 s for the person and a search waits for the model, so answers take far longer
    // than the library's.
    private val http = client.newBuilder().readTimeout(ANSWER_TIMEOUT).build()

    /** @return Which CLIs the account has signed in. */
    suspend fun signIns(): AiSignIns = call(get("sign-ins")).asAiSignIns()

    /** @return The sign-in page to open, and Codex's code to enter there. */
    suspend fun start(cli: AiCli): AiSignInStart = call(post("sign-ins/${cli.engine}/start", "{}")).asAiSignInStart()

    /**
     * @param code The code Claude Code's page showed, or null for Codex.
     * @return True once signed in; false while Codex is still waiting for the person.
     */
    suspend fun finish(cli: AiCli, code: String?): Boolean =
        call(post("sign-ins/${cli.engine}/finish", buildJsonObject { if (code != null) put("code", code) }.toString())).asAiSignInFinished()

    /** Stops a started sign-in. */
    suspend fun cancel(cli: AiCli) {
        call(post("sign-ins/${cli.engine}/cancel", "{}"))
    }

    /** Signs the CLI out of the account. */
    suspend fun signOut(cli: AiCli) {
        call(post("sign-ins/${cli.engine}/sign-out", "{}"))
    }

    /**
     * @param engine An id from [AI_ENGINES].
     * @return The models the account's CLI offers.
     */
    suspend fun models(engine: String): CliCatalog =
        call(Request.Builder().url(url("models").newBuilder().addQueryParameter("engine", engine).build()).get().build()).asCatalog()

    /**
     * @param query Search text in mymind's query syntax.
     * @param choice The engine, model and effort to rank with.
     * @return The library's matches for [query], the ones the model ranked first.
     */
    suspend fun search(query: String, choice: AiChoice): AiSearchAnswer {
        val body = buildJsonObject {
            put("q", query)
            put("engine", choice.engine)
            if (choice.model.isNotEmpty()) put("model", choice.model)
            if (choice.effort.isNotEmpty()) put("effort", choice.effort)
        }
        return AppJson.decodeFromJsonElement(AiSearchAnswer.serializer(), call(post("search", body.toString())))
    }

    private fun url(path: String) = server.trimEnd('/').toHttpUrl().newBuilder().addPathSegments("app/v1/ai/$path").build()

    private fun get(path: String) = Request.Builder().url(url(path)).get().build()

    private fun post(path: String, json: String) = Request.Builder().url(url(path)).post(json.toRequestBody(JSON_TYPE)).build()

    private suspend fun call(request: Request): JsonObject = withContext(Dispatchers.IO) {
        val (code, text) = try {
            http.newCall(request).execute().use { it.code to it.body.string() }
        } catch (_: IOException) {
            throw AiProblem("No connection to ThyMind. Try again when you are online.")
        }
        val json = runCatching { AppJson.parseToJsonElement(text).jsonObject }.getOrNull()
        if (code !in 200..299) {
            val said = (json?.get("error") as? JsonPrimitive)?.contentOrNull
            throw AiProblem(said?.takeIf { it.isNotBlank() && it != "not-found" } ?: "ThyMind could not answer ($code). Try again.")
        }
        json ?: throw AiProblem("ThyMind sent an answer the app cannot read.")
    }

    private companion object {
        val JSON_TYPE = "application/json".toMediaType()
        val ANSWER_TIMEOUT: Duration = Duration.ofMinutes(3)
    }
}

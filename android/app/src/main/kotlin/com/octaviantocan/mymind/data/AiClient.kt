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
import kotlinx.serialization.json.JsonObjectBuilder
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
 * One of the person's spaces that an AI search found matching.
 *
 * @property id The space's id.
 * @property name Its name.
 */
@Serializable
data class AiSpace(val id: String, val name: String)

/**
 * One part of the likeliest reading of a search, such as its card type or time range.
 *
 * @property kind `type`, `about`, `time`, `site`, `tag` or `other`.
 * @property label The part as a chip shows it, such as "March to May".
 * @property query The query text that applies it, so dropping the part can search again without the engine.
 */
@Serializable
data class AiPart(val kind: String = "other", val label: String, val query: String)

/**
 * What Claude Code or Codex read a search as.
 *
 * @property label What it is looking for, in a few plain words; empty when it gave none.
 * @property parts The likeliest reading in parts, whose queries joined make the first of [searches].
 * @property searches The searches, in mymind's query syntax, the library runs for it.
 * @property tags The person's own tags that match the search.
 * @property spaces The person's own spaces that match the search.
 * @property question True when the search asks something to be answered.
 */
@Serializable
data class AiUnderstood(
    val label: String = "",
    val parts: List<AiPart> = emptyList(),
    val searches: List<String> = emptyList(),
    val tags: List<String> = emptyList(),
    val spaces: List<AiSpace> = emptyList(),
    val question: Boolean = false,
)

/**
 * The engine's answer to a search that asks a question.
 *
 * @property text The answer, drawn from the found cards alone.
 * @property ids Ids of the cards it draws from.
 */
@Serializable
data class AiAnswer(val text: String, val ids: List<String> = emptyList())

/**
 * The cards an AI search answered: the ones the model ranked first, best first, then the rest of the candidates.
 *
 * @property cards The cards in that order.
 * @property ranked How many of them the model ranked.
 * @property understood What the search was read as, when the server says.
 * @property answer The answer to a question, or null.
 */
@Serializable
data class AiSearchAnswer(
    val cards: List<Card> = emptyList(),
    val ranked: Int = 0,
    val understood: AiUnderstood? = null,
    val answer: AiAnswer? = null,
)

@Serializable
private data class AiUnderstanding(val understood: AiUnderstood = AiUnderstood())

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
     * Has the engine read what a search means.
     *
     * @param query Search text in mymind's query syntax.
     * @param choice The engine, model and effort to read it with.
     * @param today The phone's date, `YYYY-MM-DD`, so "last week" means the person's last week.
     * @return The reading.
     */
    suspend fun understand(query: String, choice: AiChoice, today: String): AiUnderstood {
        val body = buildJsonObject {
            choose(query, choice)
            put("today", today)
        }
        return AppJson.decodeFromJsonElement(AiUnderstanding.serializer(), call(post("search/understand", body.toString()))).understood
    }

    /**
     * @param query Search text in mymind's query syntax.
     * @param choice The engine, model and effort to rank with.
     * @param understood What [understand] read the search as, or null to rank the query's own matches.
     * @return The cards the reading's searches find, the ones the model ranked first, and the answer to a question.
     */
    suspend fun search(query: String, choice: AiChoice, understood: AiUnderstood? = null): AiSearchAnswer {
        val body = buildJsonObject {
            choose(query, choice)
            if (understood != null) put("understood", AppJson.encodeToJsonElement(AiUnderstood.serializer(), understood))
        }
        return AppJson.decodeFromJsonElement(AiSearchAnswer.serializer(), call(post("search", body.toString())))
    }

    private fun JsonObjectBuilder.choose(query: String, choice: AiChoice) {
        put("q", query)
        put("engine", choice.engine)
        if (choice.model.isNotEmpty()) put("model", choice.model)
        if (choice.effort.isNotEmpty()) put("effort", choice.effort)
    }

    private fun url(path: String) = server.trimEnd('/').toHttpUrl().newBuilder().addPathSegments("app/v1/ai/$path").build()

    private fun get(path: String) = Request.Builder().url(url(path)).get().build()

    private fun post(path: String, json: String) = Request.Builder().url(url(path)).post(json.toRequestBody(JSON_TYPE)).build()

    private suspend fun call(request: Request): JsonObject = withContext(Dispatchers.IO) {
        val (code, text) = try {
            http.newCall(request).execute().use { it.code to it.body.string() }
        } catch (_: IOException) {
            throw AiProblem("No connection to Inventory. Try again when you are online.")
        }
        val json = runCatching { AppJson.parseToJsonElement(text).jsonObject }.getOrNull()
        if (code !in 200..299) {
            val said = (json?.get("error") as? JsonPrimitive)?.contentOrNull
            throw AiProblem(said?.takeIf { it.isNotBlank() && it != "not-found" } ?: "Inventory could not answer ($code). Try again.")
        }
        json ?: throw AiProblem("Inventory sent an answer the app cannot read.")
    }

    private companion object {
        val JSON_TYPE = "application/json".toMediaType()
        val ANSWER_TIMEOUT: Duration = Duration.ofMinutes(3)
    }
}

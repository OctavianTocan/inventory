package com.octaviantocan.mymind.data

import android.content.Context
import android.content.SharedPreferences
import com.octaviantocan.aikit.AiCli
import com.octaviantocan.aikit.AiSettingsLogic
import com.octaviantocan.aikit.AiSignInServices
import com.octaviantocan.aikit.AiSignInStart
import com.octaviantocan.aikit.AiSignIns
import com.octaviantocan.aikit.CliCatalog
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/** The engine, model and effort AI search runs with, kept on this phone, and the model last chosen for each engine. */
class AiPrefs(private val prefs: SharedPreferences) {
    constructor(context: Context) : this(context.applicationContext.getSharedPreferences(NAME, Context.MODE_PRIVATE))

    private val mutableChoice = MutableStateFlow(read())

    /** The current choice. */
    val choice: StateFlow<AiChoice> = mutableChoice

    /**
     * Switches to [engine] with the model last chosen for it, or the CLI's default, and that model's default effort.
     *
     * @param engine An id from [AI_ENGINES].
     */
    fun chooseEngine(engine: String) {
        val now = mutableChoice.value
        if (engine == now.engine || engine !in AI_ENGINES) return
        write(AiChoice(engine, prefs.getString(modelKey(engine), "").orEmpty(), ""))
    }

    /**
     * Uses [model] of the current engine, keeping the effort only when [catalog] says the model has it.
     *
     * @param model A model id from the catalog.
     * @param catalog The engine's model list.
     */
    fun chooseModel(model: String, catalog: CliCatalog) {
        val now = mutableChoice.value
        write(now.copy(model = model, effort = AiSettingsLogic.keptEffort(catalog, model, now.effort)))
    }

    /**
     * Uses [effort] with the current model; empty means the model's own default.
     *
     * @param effort An effort the model lists, or empty.
     */
    fun chooseEffort(effort: String) = write(mutableChoice.value.copy(effort = effort))

    private fun read(): AiChoice {
        val engine = prefs.getString(ENGINE, CLAUDE_CODE).takeIf { it in AI_ENGINES } ?: CLAUDE_CODE
        return AiChoice(engine, prefs.getString(modelKey(engine), "").orEmpty(), prefs.getString(EFFORT, "").orEmpty())
    }

    private fun write(choice: AiChoice) {
        prefs.edit().putString(ENGINE, choice.engine).putString(modelKey(choice.engine), choice.model).putString(EFFORT, choice.effort).apply()
        mutableChoice.value = choice
    }

    private fun modelKey(engine: String) = "model.$engine"

    private companion object {
        const val NAME = "ai"
        const val ENGINE = "engine"
        const val EFFORT = "effort"
    }
}

/** Ranks a pressed search with Claude Code or Codex. */
interface AiRanking {
    /** @return The chosen engine's name, such as "Claude Code", when it is signed in and can rank; otherwise null. */
    suspend fun engine(): String?

    /**
     * @param query Search text in mymind's query syntax.
     * @return The library's matches for [query], the ones the engine ranked first.
     */
    suspend fun search(query: String): AiSearchAnswer
}

/**
 * Claude Code and Codex for the signed-in account: its sign-ins and model lists as the server last answered them,
 * remembered for this run of the app so a row or page shows them at once, and AI search. Everything remembered
 * belongs to one session token and is dropped when it changes.
 *
 * @param client The server's AI routes.
 * @param prefs The phone's engine, model and effort.
 * @param session The session the answers belong to.
 */
class AiAccount(private val client: AiClient, val prefs: AiPrefs, private val session: Session) : AiRanking {
    private var owner: String? = null
    private val mutableSignIns = MutableStateFlow<AiSignIns?>(null)
    private val catalogs = mutableMapOf<String, CliCatalog>()

    /** The account's sign-ins as last answered, or null before the first answer of this session. */
    val signIns: StateFlow<AiSignIns?> = mutableSignIns

    /** Asks the server again which CLIs are signed in. */
    suspend fun refreshSignIns(): AiSignIns {
        val token = current() ?: throw CancellationException("Signed out.")
        return client.signIns().also { if (session.token == token) mutableSignIns.value = it }
    }

    /**
     * @param engine An id from [AI_ENGINES].
     * @return The engine's model list as last read this session, or null.
     */
    fun cachedModels(engine: String): CliCatalog? = if (current() != null) catalogs[engine] else null

    /**
     * Reads the engine's model list from the server and remembers it.
     *
     * @param engine An id from [AI_ENGINES].
     * @return The list.
     */
    suspend fun models(engine: String): CliCatalog {
        val token = current() ?: throw CancellationException("Signed out.")
        return client.models(engine).also { if (session.token == token) catalogs[engine] = it }
    }

    // The first search of a session asks which CLIs are signed in; later ones use that answer.
    override suspend fun engine(): String? {
        if (current() == null) return null
        val known = mutableSignIns.value ?: try {
            refreshSignIns()
        } catch (e: CancellationException) {
            throw e
        } catch (_: Exception) {
            return null
        }
        val cli = prefs.choice.value.engine.cli
        return cli.title.takeIf { known.account && known[cli].signedIn }
    }

    override suspend fun search(query: String): AiSearchAnswer {
        current() ?: throw CancellationException("Signed out.")
        return client.search(query, prefs.choice.value)
    }

    /**
     * The sign-in pages' way to the server, the browser and the clipboard, valid while this session lasts.
     *
     * @param foreground True while the app is in front, so the clipboard is read only then.
     * @param open Opens a provider's sign-in page.
     * @param copy Puts text on the clipboard.
     * @param copied Reads the clipboard's text.
     * @return The services.
     */
    fun services(foreground: () -> Boolean, open: (String) -> Unit, copy: (String) -> Unit, copied: () -> String?): AiSignInServices {
        val token = session.token
        return object : AiSignInServices {
            override fun current() = token != null && session.token == token
            override val dispatchReady: Boolean get() = current() && foreground()
            override suspend fun status(): AiSignIns = refreshSignIns()
            override fun cached(): AiSignIns? = if (current()) mutableSignIns.value else null
            override fun copied(): String? = if (dispatchReady) copied() else null
            override suspend fun start(cli: AiCli): AiSignInStart = client.start(cli)
            override suspend fun finish(cli: AiCli, code: String?): Boolean = client.finish(cli, code)
            override suspend fun cancel(cli: AiCli) = client.cancel(cli)
            override suspend fun signOut(cli: AiCli) = client.signOut(cli)
            override fun openPage(url: String) { if (dispatchReady) open(url) }
            override fun copy(text: String) { if (dispatchReady) copy(text) }
        }
    }

    /** The current session's token, dropping what another session's answers left behind. */
    private fun current(): String? {
        val token = session.token
        if (token != owner) {
            owner = token
            mutableSignIns.value = null
            catalogs.clear()
        }
        return token
    }
}

package com.octaviantocan.mymind.ui

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import com.octaviantocan.mymind.data.AI_ENGINES
import com.octaviantocan.mymind.data.AiAccount
import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.AiClient
import com.octaviantocan.mymind.data.AiPrefs
import com.octaviantocan.mymind.data.FeedbackApi
import com.octaviantocan.mymind.data.FeedbackProblem
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.feedback.FeedbackNotifier
import com.octaviantocan.mymind.feedback.FeedbackStore
import com.octaviantocan.mymind.feedback.FeedbackTracker
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okhttp3.mockwebserver.RecordedRequest
import java.util.UUID

private class MemoryTokens : TokenStore {
    private var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

private const val CLAUDE_MODELS = """{"models":[{"id":"opus","name":"Opus","efforts":["low","medium","high"],"defaultEffort":"high","isDefault":true},""" +
    """{"id":"haiku","name":"Haiku","efforts":[],"isDefault":false}]}"""
private const val CODEX_MODELS = """{"models":[{"id":"gpt-6-sol","name":"Sol","efforts":["low","medium","high"],"defaultEffort":"medium","isDefault":true}]}"""

/**
 * A signed-in [AiAccount] whose server (a local mock) says which engines are signed in and lists their models, with both
 * answers already read so a page draws at once. The search choice and the feedback choice live in a preferences file of
 * their own.
 *
 * @param claude Whether Claude Code is signed in.
 * @param codex Whether Codex is signed in.
 */
class AiFixture(claude: Boolean, codex: Boolean) : AutoCloseable {
    private val server = MockWebServer()
    val sharedPrefs = ApplicationProvider.getApplicationContext<Context>().getSharedPreferences("ai-${UUID.randomUUID()}", Context.MODE_PRIVATE)
    val prefs = AiPrefs(sharedPrefs)
    val account: AiAccount

    init {
        server.dispatcher = object : Dispatcher() {
            override fun dispatch(request: RecordedRequest): MockResponse {
                val path = request.path.orEmpty()
                val body = when {
                    path.startsWith("/app/v1/ai/sign-ins") -> """{"account":true,"claude":{"signedIn":$claude},"codex":{"signedIn":$codex}}"""
                    path.startsWith("/app/v1/ai/models") -> if (path.contains("codex")) CODEX_MODELS else CLAUDE_MODELS
                    else -> return MockResponse().setResponseCode(404).setBody("""{"error":"not-found"}""")
                }
                return MockResponse().setBody(body)
            }
        }
        server.start()
        val session = Session(MemoryTokens())
        runBlocking { session.signIn("token") }
        account = AiAccount(AiClient(OkHttpClient(), server.url("/").toString()), prefs, session)
        runBlocking {
            account.refreshSignIns()
            AI_ENGINES.forEach { account.models(it) }
        }
    }

    override fun close() = server.shutdown()
}

/** A [FeedbackApi] that answers from memory and remembers what it was asked. */
class FakeFeedbackApi : FeedbackApi {
    val sent = mutableListOf<Triple<String, AiChoice, String>>()
    val asked = mutableListOf<String>()
    val answers = mutableMapOf<String, FeedbackStatus>()
    var refuse: FeedbackProblem? = null
    var jobs = 0

    override suspend fun send(text: String, choice: AiChoice, clientId: String): FeedbackStatus {
        sent += Triple(text, choice, clientId)
        refuse?.let { throw it }
        val id = "job${++jobs}"
        return FeedbackStatus(id, "queued").also { answers[id] = it.copy(title = "Your feedback is waiting its turn") }
    }

    override suspend fun status(id: String): FeedbackStatus {
        asked += id
        return answers[id] ?: throw FeedbackProblem("No such feedback.", notFound = true)
    }
}

/**
 * A tracker over [api] with its store in memory, a notifier that shows nothing, and counters for the update check and the
 * background check it would have started.
 */
class TrackerFixture(val api: FakeFeedbackApi = FakeFeedbackApi(), allowed: Boolean = true) {
    val context: Context = ApplicationProvider.getApplicationContext()
    val store = FeedbackStore(context.getSharedPreferences("feedback-${UUID.randomUUID()}", Context.MODE_PRIVATE))
    val notifier = FeedbackNotifier(context) { allowed }
    var updateChecks = 0
    var scheduled = 0
    val tracker = FeedbackTracker(api, store, notifier, onShipped = { updateChecks++ }, schedule = { scheduled++ })
}

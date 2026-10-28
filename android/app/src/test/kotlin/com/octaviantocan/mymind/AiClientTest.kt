package com.octaviantocan.mymind

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.aikit.AiCli
import com.octaviantocan.aikit.AiSignIn
import com.octaviantocan.aikit.AiSignIns
import com.octaviantocan.aikit.CliCatalog
import com.octaviantocan.aikit.CliModel
import com.octaviantocan.mymind.data.AiAnswer
import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.AiClient
import com.octaviantocan.mymind.data.AiPrefs
import com.octaviantocan.mymind.data.AiPart
import com.octaviantocan.mymind.data.AiProblem
import com.octaviantocan.mymind.data.AiSpace
import com.octaviantocan.mymind.data.AiUnderstood
import com.octaviantocan.mymind.data.AppJson
import com.octaviantocan.mymind.data.CODEX
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.jsonObject
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AiClientTest {
    private val server = MockWebServer()
    private lateinit var ai: AiClient

    @Before
    fun setUp() {
        server.start()
        ai = AiClient(OkHttpClient(), server.url("/").toString())
    }

    @After
    fun tearDown() = server.shutdown()

    private fun answer(body: String, code: Int = 200) = server.enqueue(MockResponse().setResponseCode(code).setBody(body))

    @Test
    fun signInsAndTheirStepsUseTheAccountsOwnRoutes() = runBlocking {
        answer("""{"account":true,"claude":{"signedIn":true,"email":"a@example.com","plan":"max"},"codex":{"signedIn":false}}""")
        assertEquals(AiSignIns(true, AiSignIn(true, "a@example.com", "max"), AiSignIn(false)), ai.signIns())
        assertEquals("/app/v1/ai/sign-ins", server.takeRequest().path)

        answer("""{"url":"https://auth.openai.com/codex/device","code":"ABCD-EFGH","paste":false}""")
        assertEquals("ABCD-EFGH", ai.start(AiCli.Codex).code)
        assertEquals("POST /app/v1/ai/sign-ins/codex/start", server.takeRequest().let { "${it.method} ${it.path}" })

        answer("""{"signedIn":false,"pending":true}""")
        assertFalse(ai.finish(AiCli.Codex, null))
        answer("""{"signedIn":true,"pending":false}""")
        assertTrue(ai.finish(AiCli.Claude, "code#state"))
        server.takeRequest()
        assertEquals("""{"code":"code#state"}""", server.takeRequest().body.readUtf8())
    }

    @Test
    fun searchSendsTheChoiceAndReadsTheRankedCards() = runBlocking {
        answer("""{"cards":[{"id":"c2","type":"Note","flags":0,"created":"2026-10-01T00:00:00Z","modified":"2026-10-01T00:00:00Z","bumped":"2026-10-01T00:00:00Z"},{"id":"c1","type":"WebPage","flags":0,"created":"2026-10-01T00:00:00Z","modified":"2026-10-01T00:00:00Z","bumped":"2026-10-01T00:00:00Z"}],"next":null,"ranked":1}""")
        val found = ai.search("calm lake", AiChoice(CODEX, "gpt-6-sol", "high"))
        assertEquals(listOf("c2", "c1"), found.cards.map { it.id })
        assertEquals(1, found.ranked)
        val sent = server.takeRequest()
        assertEquals("/app/v1/ai/search", sent.path)
        assertEquals("""{"q":"calm lake","engine":"codex","model":"gpt-6-sol","effort":"high"}""", sent.body.readUtf8())
    }

    @Test
    fun understandingSendsTheDateAndTheSearchSendsTheReadingBack() = runBlocking {
        answer("""{"understood":{"label":"Walking essays","parts":[{"kind":"type","label":"Articles","query":"type:article"}],"searches":["type:article walking"],"tags":["walks"],"spaces":[{"id":"s1","name":"Reading"}],"question":true}}""")
        val understood = ai.understand("the walking essay", AiChoice(), "2026-10-28")
        assertEquals(AiUnderstood("Walking essays", listOf(AiPart("type", "Articles", "type:article")), listOf("type:article walking"),
            listOf("walks"), listOf(AiSpace("s1", "Reading")), question = true), understood)
        val asked = server.takeRequest()
        assertEquals("/app/v1/ai/search/understand", asked.path)
        assertEquals("""{"q":"the walking essay","engine":"claude-code","today":"2026-10-28"}""", asked.body.readUtf8())

        answer("""{"cards":[{"id":"c1","type":"Article","flags":0,"created":"2026-10-01T00:00:00Z","modified":"2026-10-01T00:00:00Z","bumped":"2026-10-01T00:00:00Z"}],"next":null,"ranked":1,"understood":{"label":"Walking essays"},"answer":{"text":"You saved one, in spring.","ids":["c1"]}}""")
        val found = ai.search("the walking essay", AiChoice(), understood)
        assertEquals(AiAnswer("You saved one, in spring.", listOf("c1")), found.answer)
        assertEquals("Walking essays", found.understood?.label)
        val sent = AppJson.parseToJsonElement(server.takeRequest().body.readUtf8()).jsonObject
        assertEquals(understood, AppJson.decodeFromJsonElement(AiUnderstood.serializer(), sent.getValue("understood")))
    }

    @Test
    fun failuresCarryTheServersWords() {
        answer("""{"error":"Sign in to Codex first: Profile, AI, Codex."}""", code = 502)
        val problem = assertThrows(AiProblem::class.java) { runBlocking { ai.models(CODEX) } }
        assertEquals("Sign in to Codex first: Profile, AI, Codex.", problem.message)
        assertEquals("/app/v1/ai/models?engine=codex", server.takeRequest().path)

        answer("""{"error":"not-found"}""", code = 404)
        assertEquals("ThyMind could not answer (404). Try again.",
            assertThrows(AiProblem::class.java) { runBlocking { ai.cancel(AiCli.Claude) } }.message)
    }

    @Test
    fun eachEngineKeepsItsOwnModelAndAnEffortOnlyItsModelHas() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val prefs = AiPrefs(context.getSharedPreferences("ai-test", Context.MODE_PRIVATE).also { it.edit().clear().commit() })
        assertEquals(AiChoice(), prefs.choice.value)
        val claude = CliCatalog(listOf(CliModel("opus", "Opus", null, listOf("low", "high"), "high", true), CliModel("haiku", "Haiku", null, emptyList(), null, false)), null, null)
        prefs.chooseModel("opus", claude)
        prefs.chooseEffort("low")
        assertEquals(AiChoice(model = "opus", effort = "low"), prefs.choice.value)
        prefs.chooseModel("haiku", claude)
        assertEquals(AiChoice(model = "haiku"), prefs.choice.value)
        prefs.chooseEngine(CODEX)
        assertEquals(AiChoice(CODEX), prefs.choice.value)
        prefs.chooseEngine("claude-code")
        assertEquals(AiChoice(model = "haiku"), prefs.choice.value)
        // A new instance reads what was kept.
        assertEquals(AiChoice(model = "haiku"), AiPrefs(context.getSharedPreferences("ai-test", Context.MODE_PRIVATE)).choice.value)
    }
}

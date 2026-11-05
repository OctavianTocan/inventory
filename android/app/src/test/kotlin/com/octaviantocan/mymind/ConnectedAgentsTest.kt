package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.ConnectedAgent
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.settings.agentDates
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withTimeout
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.time.LocalDate
import java.time.ZoneId

class ConnectedAgentsTest {
    private val server = MockWebServer()
    private lateinit var session: Session
    private lateinit var repository: Repository

    private val list = """{"agents":[
        {"id":"aa11","name":"Claude","connected":"2026-11-01T09:00:00Z","used":"2026-11-05T09:00:00Z"},
        {"id":"bb22","name":"ChatGPT","connected":"2026-11-04T09:00:00Z","used":"2026-11-04T10:00:00Z"}]}"""

    @Before
    fun setUp() = runBlocking {
        server.start()
        session = Session(MemoryTokens())
        session.signIn("phone-token")
        val client = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session, server.url("/").toString())).build()
        repository = Repository(MymindApi(client, server.url("/").toString()), session)
        Dispatchers.setMain(Dispatchers.Unconfined)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        server.shutdown()
    }

    private fun answer(body: String, code: Int = 200) = server.enqueue(MockResponse().setResponseCode(code).setBody(body))

    private suspend fun until(check: () -> Boolean) = withTimeout(5_000) { while (!check()) delay(10) }

    @Test
    fun listsAndRemovesAgentsWithThePhonesOwnToken() = runBlocking {
        answer(list)
        val agents = repository.agents() as ApiResult.Ok
        assertEquals(listOf("Claude", "ChatGPT"), agents.value.agents.map { it.name })
        val read = server.takeRequest()
        assertEquals("GET", read.method)
        assertEquals("/app/v1/agents", read.path)
        assertEquals("Bearer phone-token", read.getHeader("Authorization"))

        answer("", 204)
        assertTrue(repository.removeAgent("aa11") is ApiResult.Ok)
        val removed = server.takeRequest()
        assertEquals("DELETE", removed.method)
        assertEquals("/app/v1/agents/aa11", removed.path)
        assertEquals("phone-token", session.token)
    }

    @Test
    fun theListDropsARemovedAgentAtOnceAndComesBackWhenTheServerRefuses() = runBlocking {
        val settings = SettingsViewModel(repository)
        answer(list)
        settings.loadAgents()
        until { settings.agents.value.agents != null }
        assertEquals(listOf("aa11", "bb22"), settings.agents.value.agents!!.map { it.id })

        answer("", 204)
        settings.removeAgent("aa11")
        assertEquals(listOf("bb22"), settings.agents.value.agents!!.map { it.id })
        until { server.requestCount == 2 }

        answer("""{"error":"not-found"}""", 404)
        answer(list)
        settings.removeAgent("bb22")
        until { settings.agents.value.agents?.size == 2 }
        assertEquals(listOf("aa11", "bb22"), settings.agents.value.agents!!.map { it.id })
    }

    @Test
    fun datesReadAsDaysInThePhonesZone() {
        val zone = ZoneId.systemDefault()
        val today = LocalDate.of(2026, 11, 5)
        fun at(day: LocalDate) = day.atTime(12, 0).atZone(zone).toInstant().toString()
        val agent = ConnectedAgent("x", "Claude", at(LocalDate.of(2026, 9, 1)), at(today))
        assertEquals("Connected 1 Sep · used today", agentDates(agent, today))
        assertEquals("Connected yesterday", agentDates(agent.copy(connected = at(today.minusDays(1)), used = at(today.minusDays(1))), today))
        assertEquals("Connected 3 Dec 2025 · used yesterday", agentDates(agent.copy(connected = at(LocalDate.of(2025, 12, 3)), used = at(today.minusDays(1))), today))
    }
}

private class MemoryTokens : TokenStore {
    var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

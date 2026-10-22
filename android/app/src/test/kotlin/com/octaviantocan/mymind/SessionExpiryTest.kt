package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.data.TokenStore
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

private class MemoryStore(var value: String? = null) : TokenStore {
    override suspend fun read() = value
    override suspend fun write(token: String) { value = token }
    override suspend fun clear() { value = null }
}

class SessionExpiryTest {
    private val server = MockWebServer()
    private val store = MemoryStore("tok-123")
    private lateinit var session: Session
    private lateinit var client: OkHttpClient

    @Before
    fun setUp() {
        server.start()
        session = Session(store)
        runBlocking { session.restore() }
        client = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session, server.url("/").toString())).build()
    }

    @After
    fun tearDown() = server.shutdown()

    private fun get() = client.newCall(Request.Builder().url(server.url("/app/v1/cards")).build()).execute().close()

    @Test
    fun sendsBearerToken() {
        server.enqueue(MockResponse().setResponseCode(200).setBody("{}"))
        get()
        assertEquals("Bearer tok-123", server.takeRequest().getHeader("Authorization"))
        assertEquals(SessionState.SignedIn, session.state.value)
    }

    @Test
    fun otherHostsNeverSeeTheToken() {
        val other = MockWebServer().apply { start() }
        other.enqueue(MockResponse().setResponseCode(401))
        client.newCall(Request.Builder().url(other.url("/cover.jpg")).build()).execute().close()
        assertNull(other.takeRequest().getHeader("Authorization"))
        assertEquals(SessionState.SignedIn, session.state.value)
        other.shutdown()
    }

    @Test
    fun unauthorizedResponseSignsOut() {
        server.enqueue(MockResponse().setResponseCode(401).setBody("""{"error":"unauthorized"}"""))
        get()
        assertEquals(SessionState.SignedOut, session.state.value)
        assertNull(session.token)
        assertNull(store.value)
    }

    @Test
    fun staleUnauthorizedKeepsNewerSignIn() {
        session.invalidate("tok-old")
        assertEquals(SessionState.SignedIn, session.state.value)
        assertEquals("tok-123", store.value)
    }

    @Test
    fun aRefusedUploadPieceKeepsTheSession() {
        server.enqueue(MockResponse().setResponseCode(401).setBody("Invalid upload authorization"))
        val piece = Request.Builder().url(server.url("/uploads/u1")).header("x-upload-token", "upload-tok").build()
        client.newCall(piece).execute().close()
        assertEquals("Bearer tok-123", server.takeRequest().getHeader("Authorization"))
        assertEquals(SessionState.SignedIn, session.state.value)
        assertEquals("tok-123", store.value)
    }

    @Test
    fun otherErrorsKeepTheSession() {
        server.enqueue(MockResponse().setResponseCode(500))
        get()
        assertEquals(SessionState.SignedIn, session.state.value)
    }
}

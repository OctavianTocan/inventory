package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.HubListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import okhttp3.OkHttpClient
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.TimeUnit

class HubListenerTest {
    private val server = MockWebServer()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private lateinit var hub: HubListener
    private val opened = CopyOnWriteArrayList<WebSocket>()

    @Before
    fun setUp() {
        server.start()
        hub = HubListener(OkHttpClient(), server.url("/").toString(), scope)
    }

    @After
    fun tearDown() {
        hub.stop()
        scope.cancel()
        opened.forEach { it.close(1000, null) }
        server.shutdown()
    }

    // A hub connection that hands over its socket once open and the first message the phone sends.
    private fun hubAnswer(socket: CompletableDeferred<WebSocket>, first: CompletableDeferred<String> = CompletableDeferred()) =
        MockResponse().withWebSocketUpgrade(object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                opened += webSocket
                socket.complete(webSocket)
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                first.complete(text)
            }

            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                webSocket.close(1000, null)
            }
        })

    @Test
    fun aChangeTheHubAnnouncesReachesTheApp() = runBlocking {
        val socket = CompletableDeferred<WebSocket>()
        val handshake = CompletableDeferred<String>()
        server.enqueue(hubAnswer(socket, handshake))
        val heard = async(start = CoroutineStart.UNDISPATCHED) { withTimeout(5_000) { hub.changes.first() } }
        hub.start()
        assertEquals("/hub", server.takeRequest(5, TimeUnit.SECONDS)!!.path)
        assertEquals("{\"protocol\":\"json\",\"version\":1}\u001e", withTimeout(5_000) { handshake.await() })
        val open = withTimeout(5_000) { socket.await() }
        open.send("{}\u001e{\"type\":6}\u001e")
        open.send("{\"type\":1,\"target\":\"object:updated\",\"arguments\":[{\"id\":\"c1\"}]}\u001e")
        heard.await()
    }

    @Test
    fun aDroppedHubIsTriedAgain() = runBlocking {
        val first = CompletableDeferred<WebSocket>()
        val second = CompletableDeferred<WebSocket>()
        server.enqueue(hubAnswer(first))
        server.enqueue(hubAnswer(second))
        hub.start()
        withTimeout(5_000) { first.await() }.close(1001, "going away")
        withTimeout(5_000) { second.await() }
        assertEquals(2, server.requestCount)
    }

    @Test
    fun aHubTheServerRefusesIsLeftAlone() = runBlocking {
        server.enqueue(MockResponse().setResponseCode(401))
        hub.start()
        server.takeRequest(5, TimeUnit.SECONDS)
        assertNull("signed out: no second try", server.takeRequest(2_500, TimeUnit.MILLISECONDS))
    }
}

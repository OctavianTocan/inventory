package com.octaviantocan.mymind.data

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener

// SignalR's record separator, which ends each message on the hub, and the handshake that asks for JSON messages.
private const val RECORD_END = '\u001e'
private const val HANDSHAKE = "{\"protocol\":\"json\",\"version\":1}$RECORD_END"

// The wait before connecting again after the hub dropped, doubling from one second up to a minute.
private const val FIRST_RETRY_MS = 1_000L
private const val LAST_RETRY_MS = 60_000L

// Answers to the connection that mean trying again is pointless: signed out, or a server without the hub.
private val REFUSED = setOf(401, 403, 404)

/**
 * Listens on the server's `/hub` WebSocket, the SignalR hub the web app uses, while started, and says so on [changes]
 * whenever the library changes on the server: from another device, the web, or the server's own reading of a saved
 * link. As in Zero, the phone hears of a change as it happens instead of asking for it. A dropped connection is tried
 * again, waiting longer each time up to a minute; one the server refuses is left alone until the next [start].
 *
 * @param client Sends the session's token with the connection.
 * @param server The server's address.
 * @param scope Scope the waits before trying again run in.
 */
class HubListener(private val client: OkHttpClient, server: String, private val scope: CoroutineScope) {
    private val address = server.trimEnd('/') + "/hub"
    private val changed = MutableSharedFlow<Unit>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)
    private var socket: WebSocket? = null
    private var retry: Job? = null
    private var wait = FIRST_RETRY_MS
    private var started = false

    /** One signal for each change the server announces; a collector still busy gets one for all it missed. */
    val changes: SharedFlow<Unit> = changed

    /** Connects now, unless already connected; a wait before trying again is cut short. */
    @Synchronized
    fun start() {
        started = true
        if (socket != null) return
        retry?.cancel()
        retry = null
        connect()
    }

    /** Closes the connection and stops trying again. */
    @Synchronized
    fun stop() {
        started = false
        retry?.cancel()
        retry = null
        socket?.close(1000, null)
        socket = null
    }

    private fun connect() {
        socket = client.newWebSocket(Request.Builder().url(address).build(), Listener())
    }

    @Synchronized
    private fun heard(webSocket: WebSocket, text: String) {
        if (socket !== webSocket) return
        wait = FIRST_RETRY_MS
        if (text.split(RECORD_END).any(::invocation)) changed.tryEmit(Unit)
    }

    @Synchronized
    private fun dropped(webSocket: WebSocket, again: Boolean) {
        if (socket !== webSocket) return
        socket = null
        if (!started || !again) return
        val pause = wait
        wait = (wait * 2).coerceAtMost(LAST_RETRY_MS)
        retry = scope.launch {
            delay(pause)
            synchronized(this@HubListener) {
                retry = null
                if (started && socket == null) connect()
            }
        }
    }

    private inner class Listener : WebSocketListener() {
        override fun onOpen(webSocket: WebSocket, response: Response) {
            webSocket.send(HANDSHAKE)
        }

        override fun onMessage(webSocket: WebSocket, text: String) = heard(webSocket, text)

        override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
            webSocket.close(1000, null)
        }

        override fun onClosed(webSocket: WebSocket, code: Int, reason: String) = dropped(webSocket, again = true)

        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) =
            dropped(webSocket, again = response?.code !in REFUSED)
    }
}

// True for a SignalR invocation, which the hub sends for each change; not for its pings or the handshake's answer.
private fun invocation(record: String): Boolean = record.isNotBlank() &&
    runCatching { AppJson.parseToJsonElement(record).jsonObject["type"]?.jsonPrimitive?.intOrNull == 1 }.getOrDefault(false)

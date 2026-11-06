package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.awaitLoopbackCallback
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import java.net.HttpURLConnection
import java.net.ServerSocket
import java.net.URL

class LoopbackCallbackTest {
    private fun freePort(): Int = ServerSocket(0).use { it.localPort }

    /** GETs [url] once the listener is up, answering its status and body. */
    private suspend fun get(url: String): Pair<Int, String> {
        repeat(50) {
            try {
                val connection = URL(url).openConnection() as HttpURLConnection
                val status = connection.responseCode
                val body = (if (status < 400) connection.inputStream else connection.errorStream).bufferedReader().readText()
                return status to body
            } catch (_: java.net.ConnectException) {
                delay(50)
            }
        }
        error("the listener never came up")
    }

    @Test
    fun theBrowsersReturnIsTakenAndOtherRequestsAreNot() = runBlocking {
        val port = freePort()
        val callback = "http://127.0.0.1:$port/auth/callback"
        val waiting = async { awaitLoopbackCallback(callback, "intent:#Intent;package=x;end") }
        assertEquals(404, get("http://127.0.0.1:$port/favicon.ico").first)
        assertEquals(404, get("http://127.0.0.1:$port/auth/callback").first)
        val (status, page) = get("$callback?code=c1&scope=openid&state=s1&client_id=app_1")
        assertEquals(200, status)
        assertTrue(page.contains("Back to Inventory") && page.contains("intent:#Intent;package=x;end"))
        assertEquals("$callback?code=c1&scope=openid&state=s1&client_id=app_1", withTimeout(5_000) { waiting.await() })
    }

    @Test
    fun onlyTheLoopbackAddressIsListenedOn() {
        assertThrows(IllegalArgumentException::class.java) {
            runBlocking { awaitLoopbackCallback("http://example.com:1455/auth/callback", "") }
        }
        assertThrows(IllegalArgumentException::class.java) {
            runBlocking { awaitLoopbackCallback("https://127.0.0.1:1455/auth/callback", "") }
        }
    }

    @Test
    fun aWaitThatRunsOutFails() {
        val port = freePort()
        assertThrows(java.io.IOException::class.java) {
            runBlocking { awaitLoopbackCallback("http://127.0.0.1:$port/auth/callback", "", lifetimeMs = 600) }
        }
    }
}

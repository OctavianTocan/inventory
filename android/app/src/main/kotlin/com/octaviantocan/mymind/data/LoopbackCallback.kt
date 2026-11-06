package com.octaviantocan.mymind.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.withContext
import java.net.InetAddress
import java.net.ServerSocket
import java.net.SocketTimeoutException
import java.net.URI
import kotlin.coroutines.coroutineContext

/**
 * Waits on this phone's own loopback address for the browser to come back from OpenAI's Sign in with ChatGPT page,
 * which sends it to `http://127.0.0.1:1455/auth/callback?code=…&state=…` (developers.openai.com/siwc). Only that path
 * is answered with the "go back to Inventory" page; anything else the browser asks for gets a 404 and the wait goes on.
 *
 * @param callback The callback address the server gave, `http://127.0.0.1:<port>/<path>`.
 * @param backToApp The link the page's button opens to bring the app to the front.
 * @param lifetimeMs How long to wait before giving up.
 * @return The whole address the browser asked for.
 * @throws IllegalArgumentException When [callback] is not on the loopback address.
 * @throws java.io.IOException When the port is taken or the wait ran out.
 */
suspend fun awaitLoopbackCallback(callback: String, backToApp: String, lifetimeMs: Long = 15 * 60_000L): String = withContext(Dispatchers.IO) {
    val uri = URI(callback)
    require(uri.scheme == "http" && uri.host == "127.0.0.1" && uri.port in 1024..65535) { "Not a loopback callback." }
    ServerSocket(uri.port, 4, InetAddress.getByName("127.0.0.1")).use { server ->
        server.soTimeout = 500
        val until = System.currentTimeMillis() + lifetimeMs
        while (System.currentTimeMillis() < until) {
            coroutineContext.ensureActive()
            val socket = try { server.accept() } catch (_: SocketTimeoutException) { continue }
            socket.use {
                it.soTimeout = 5_000
                val line = runCatching { it.getInputStream().bufferedReader().readLine() }.getOrNull().orEmpty()
                val target = line.split(' ').getOrNull(1).orEmpty()
                val ours = line.startsWith("GET ") && target.substringBefore('?') == uri.rawPath && '?' in target
                val body = if (ours) donePage(backToApp) else "Not found"
                val bytes = body.toByteArray()
                val head = "HTTP/1.1 ${if (ours) "200 OK" else "404 Not Found"}\r\nContent-Type: text/html; charset=utf-8\r\n" +
                    "Content-Length: ${bytes.size}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n"
                runCatching { it.getOutputStream().apply { write(head.toByteArray()); write(bytes); flush() } }
                if (ours) return@withContext "http://127.0.0.1:${uri.port}$target"
            }
        }
        throw java.io.IOException("The ChatGPT sign-in took too long. Start again.")
    }
}

/** The page the browser shows once it came back: a button that opens the app again. */
private fun donePage(backToApp: String): String = """<!doctype html><html><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Inventory</title>
<style>body{font:17px/1.5 system-ui,sans-serif;margin:0;display:flex;min-height:100vh;align-items:center;justify-content:center;
background:#f7f5f2;color:#222}main{max-width:320px;padding:24px;text-align:center}a{display:inline-block;margin-top:16px;
padding:12px 22px;border-radius:999px;background:#222;color:#fff;text-decoration:none}
@media(prefers-color-scheme:dark){body{background:#151413;color:#eee}a{background:#eee;color:#151413}}</style></head>
<body><main><p>ChatGPT is connected. Go back to Inventory to finish.</p><a href="$backToApp">Back to Inventory</a></main></body></html>"""

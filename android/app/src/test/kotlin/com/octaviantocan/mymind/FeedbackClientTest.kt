package com.octaviantocan.mymind

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.CODEX
import com.octaviantocan.mymind.data.FeedbackClient
import com.octaviantocan.mymind.data.FeedbackProblem
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.data.isFinal
import kotlinx.coroutines.runBlocking
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
class FeedbackClientTest {
    private val server = MockWebServer()
    private lateinit var feedback: FeedbackClient

    @Before
    fun setUp() {
        server.start()
        feedback = FeedbackClient(OkHttpClient(), server.url("/").toString())
    }

    @After
    fun tearDown() = server.shutdown()

    private fun answer(body: String, code: Int = 200) = server.enqueue(MockResponse().setResponseCode(code).setBody(body))

    @Test
    fun sendPostsTheWholeChoiceAndReadsTheJob() = runBlocking {
        answer("""{"id":"f1","state":"queued"}""", code = 202)
        val job = feedback.send("Rounder cards", AiChoice(CODEX, "gpt-6-sol", "high"), "client-7")
        assertEquals(FeedbackStatus("f1", "queued"), job)
        val sent = server.takeRequest()
        assertEquals("POST /app/v1/feedback", "${sent.method} ${sent.path}")
        assertEquals("""{"text":"Rounder cards","engine":"codex","model":"gpt-6-sol","effort":"high","client_id":"client-7"}""", sent.body.readUtf8())
    }

    @Test
    fun statusReadsTitleDetailAndVersionAndEndsOnFinalStates() = runBlocking {
        answer("""{"state":"shipped","title":"Shipped in 0.6.56. Tap to update","detail":"","version":"0.6.56"}""")
        val job = feedback.status("f1")
        assertEquals(FeedbackStatus("f1", "shipped", "Shipped in 0.6.56. Tap to update", "", "0.6.56"), job)
        assertTrue(job.isFinal)
        assertFalse(FeedbackStatus("f1", "testing").isFinal)
        assertEquals("/app/v1/feedback/f1", server.takeRequest().path)
    }

    @Test
    fun aRefusalOrAnUnknownJobIsTheServersSentence() {
        answer("""{"error":"Only Octavian can send feedback for now."}""", code = 403)
        val refused = assertThrows(FeedbackProblem::class.java) { runBlocking { feedback.send("x", AiChoice(), "c") } }
        assertEquals("Only Octavian can send feedback for now.", refused.message)
        assertFalse(refused.notFound)

        answer("""{"error":"No such feedback."}""", code = 404)
        val missing = assertThrows(FeedbackProblem::class.java) { runBlocking { feedback.status("nope") } }
        assertEquals("No such feedback.", missing.message)
        assertTrue(missing.notFound)

        answer("oops", code = 500)
        assertEquals("Inventory could not answer (500). Try again.",
            assertThrows(FeedbackProblem::class.java) { runBlocking { feedback.status("f1") } }.message)
    }
}

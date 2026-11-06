package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.Analytics
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.AppEvent
import com.octaviantocan.mymind.data.EventBatch
import com.octaviantocan.mymind.data.EventsAnswer
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class AnalyticsTest {
    @get:Rule
    val folder = TemporaryFolder()

    private fun analytics() = Analytics(folder.root.resolve("events.json")) { 1_000L }

    @Test
    fun eventsAreStampedKeptAcrossLaunchesAndSentInBatchesOfAHundred() = runBlocking {
        val first = analytics()
        repeat(250) { first.record(AppEvent("screen", screen = "all")) }
        first.count("pinch", detail = "2")
        first.count("pinch", detail = "2")
        first.fold()

        val again = analytics()
        assertEquals(251, again.size)
        val sent = mutableListOf<EventBatch>()
        assertTrue(again.send("0.6.75") { batch -> sent += batch; ApiResult.Ok(EventsAnswer(kept = batch.events.size)) })
        assertEquals(listOf(100, 100, 51), sent.map { it.events.size })
        assertEquals(3, sent.map { it.batch }.toSet().size)
        assertTrue(sent.all { it.version == "0.6.75" })
        assertEquals(1_000L, sent[0].events[0].at)
        assertEquals(AppEvent("pinch", detail = "2", count = 2.0, at = 1_000L), sent[2].events.last())
        assertEquals(0, analytics().size)
    }

    @Test
    fun aFailedSendKeepsTheEventsAndTheQueueIsCapped() = runBlocking {
        val analytics = analytics()
        repeat(Analytics.KEPT + 5) { analytics.record(AppEvent("open", card = "c$it")) }
        assertEquals(Analytics.KEPT, analytics.size)
        assertFalse(analytics.send("v") { ApiResult.Failed(ApiFailure.Network) })
        assertEquals(Analytics.KEPT, analytics.size)
        val first = mutableListOf<String>()
        analytics.send("v") { batch -> first += batch.events.first().card; ApiResult.Ok(EventsAnswer()) }
        assertEquals("c5", first.first())
    }

    @Test
    fun stopDropsEverythingAndSignOutClears() = runBlocking {
        val analytics = analytics()
        repeat(3) { analytics.record(AppEvent("open")) }
        analytics.send("v") { ApiResult.Ok(EventsAnswer(stop = true)) }
        analytics.record(AppEvent("open"))
        assertEquals(0, analytics.size)

        val other = Analytics(folder.root.resolve("other.json"))
        other.record(AppEvent("search", words = "harbour"))
        other.fold()
        other.clear()
        assertEquals(0, other.size)
        assertEquals(0, Analytics(folder.root.resolve("other.json")).size)
    }
}

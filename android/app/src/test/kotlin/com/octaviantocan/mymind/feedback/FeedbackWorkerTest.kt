package com.octaviantocan.mymind.feedback

import android.app.Notification
import android.app.NotificationManager
import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.ui.FakeFeedbackApi
import com.octaviantocan.mymind.ui.TrackerFixture
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config

/** The background check of feedback jobs, and what following a job does. */
@RunWith(AndroidJUnit4::class)
@Config(sdk = [36])
class FeedbackWorkerTest {
    private val api = FakeFeedbackApi()
    private val fixture = TrackerFixture(api)
    private var next = 0

    private fun check() = runBlocking { FeedbackWorker.checkAndContinue(fixture.tracker) { next++ } }

    @Test
    fun withNoActiveJobTheCheckEndsWithoutAnotherOne() {
        assertFalse(check())
        assertEquals(0, next)
        assertEquals(emptyList<String>(), api.asked)
    }

    @Test
    fun whileAJobIsActiveEachCheckSchedulesTheNextAndAFinalStateEndsTheChain() {
        api.answers["job1"] = FeedbackStatus("job1", "working", "Working on your feedback")
        fixture.store.add("job1")
        assertTrue(check())
        assertTrue(check())
        assertEquals(2, next)
        assertEquals(listOf("job1"), fixture.store.active())

        api.answers["job1"] = FeedbackStatus("job1", "merged_server", "Merged. It goes live with the next server update")
        assertFalse(check())
        assertEquals(2, next)
        assertTrue(fixture.store.active().isEmpty())
        assertFalse(fixture.tracker.hasActive())
        // A job that ended is not asked about again.
        val asked = api.asked.size
        assertFalse(check())
        assertEquals(asked, api.asked.size)
    }

    @Test
    fun aJobThePhoneCannotReachStaysActiveButOneTheServerForgotIsDropped() {
        fixture.store.add("gone")
        // The fake answers "not found" for ids it has not been given.
        assertFalse(check())
        assertTrue(fixture.store.active().isEmpty())
    }

    @Test
    fun shippingStartsTheUpdateCheckOnceAndNotificationsOnlyShowChanges() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val manager = context.getSystemService(NotificationManager::class.java)
        api.answers["job1"] = FeedbackStatus("job1", "testing", "Testing your change")
        fixture.store.add("job1")
        check()
        check()
        assertEquals(1, shadowOf(manager).allNotifications.size)
        assertEquals(0, fixture.updateChecks)

        api.answers["job1"] = FeedbackStatus("job1", "shipped", "Shipped in 0.6.56. Tap to update", version = "0.6.56")
        check()
        check()
        assertEquals(1, fixture.updateChecks)
        val shown = shadowOf(manager).allNotifications.single()
        assertEquals("Shipped in 0.6.56. Tap to update", shown.extras.getString(Notification.EXTRA_TITLE))
    }

    @Test
    fun sendingKeepsTheJobAndStartsTheBackgroundCheck() {
        val sent = runBlocking { fixture.tracker.send("Rounder cards", com.octaviantocan.mymind.data.AiChoice(model = "opus", effort = "medium"), "client-1") }
        assertEquals("job1", sent.id)
        assertEquals(listOf("job1"), fixture.store.active())
        assertEquals("job1", fixture.tracker.activeId())
        assertEquals(1, fixture.scheduled)
    }
}

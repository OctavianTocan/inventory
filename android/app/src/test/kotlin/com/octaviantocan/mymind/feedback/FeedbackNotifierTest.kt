package com.octaviantocan.mymind.feedback

import android.app.Notification
import android.app.NotificationManager
import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.MainActivity
import com.octaviantocan.mymind.data.FINAL_FEEDBACK_STATES
import com.octaviantocan.mymind.data.FeedbackStatus
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config

/** The Feedback notification: one per job, updated in place, in the server's words. */
@RunWith(AndroidJUnit4::class)
@Config(sdk = [36])
class FeedbackNotifierTest {
    private val context: Context = ApplicationProvider.getApplicationContext()
    private val manager = context.getSystemService(NotificationManager::class.java)

    /** Every state of the server's table with the words it sends, and whether it is final. */
    private val table = listOf(
        Triple("queued", "Your feedback is waiting its turn", false),
        Triple("working", "Working on your feedback", false),
        Triple("testing", "Testing your change", false),
        Triple("fixing", "Fixing a failed test", false),
        Triple("merging", "Getting ready to ship", false),
        Triple("shipped", "Shipped in 0.6.56. Tap to update", true),
        Triple("merged_server", "Merged. It goes live with the next server update", true),
        Triple("merged_not_released", "Merged, but the release didn't go out. Tavi can see why", true),
        Triple("no_change", "No change needed: it already works that way", true),
        Triple("failed", "Couldn't ship this: the tests kept failing", true),
    )

    private fun posted(id: String): Notification? = shadowOf(manager).getNotification(FeedbackNotifier.id(id))

    @Test
    fun everyStateShowsTheServersTitleAndDetail() {
        val notifier = FeedbackNotifier(context) { true }
        table.forEachIndexed { index, (state, title, final) ->
            val id = "job-$index"
            notifier.show(FeedbackStatus(id, state, title, "Detail of $state"))
            val shown = checkNotNull(posted(id)) { state }
            assertEquals(state, title, shown.extras.getString(Notification.EXTRA_TITLE))
            assertEquals(state, "Detail of $state", shown.extras.getString(Notification.EXTRA_TEXT))
            assertEquals(state, FeedbackNotifier.CHANNEL, shown.channelId)
            assertEquals(state, final, shown.flags and Notification.FLAG_AUTO_CANCEL != 0)
            assertEquals(state, !final, shown.flags and Notification.FLAG_ONGOING_EVENT != 0)
        }
        assertEquals(FINAL_FEEDBACK_STATES, table.filter { it.third }.map { it.first }.toSet())
    }

    @Test
    fun theChannelIsCalledFeedbackAndAJobsNotificationIsUpdatedInPlaceQuietly() {
        val notifier = FeedbackNotifier(context) { true }
        notifier.show(FeedbackStatus("job-1", "queued", "Your feedback is waiting its turn"))
        assertEquals("Feedback", manager.getNotificationChannel("feedback").name.toString())
        notifier.show(FeedbackStatus("job-1", "working", "Working on your feedback"))
        val all = shadowOf(manager).allNotifications
        assertEquals(1, all.size)
        val shown = all.single()
        assertEquals("Working on your feedback", shown.extras.getString(Notification.EXTRA_TITLE))
        assertTrue(shown.flags and Notification.FLAG_ONLY_ALERT_ONCE != 0)
        assertNotNull(posted("job-1"))

        notifier.show(FeedbackStatus("job-2", "queued", "Your feedback is waiting its turn"))
        assertEquals(2, shadowOf(manager).allNotifications.size)
    }

    @Test
    fun theTapOpensTheAppWhereTheUpdateShows() {
        FeedbackNotifier(context) { true }.show(FeedbackStatus("job-1", "shipped", "Shipped in 0.6.56. Tap to update"))
        val tap = shadowOf(checkNotNull(posted("job-1")).contentIntent).savedIntent
        assertEquals(MainActivity::class.java.name, tap.component?.className)
    }

    @Test
    fun aDeniedNotificationPermissionPostsNothingAndDoesNotCrash() {
        FeedbackNotifier(context) { false }.show(FeedbackStatus("job-1", "working", "Working on your feedback"))
        assertNull(posted("job-1"))
        assertEquals(0, shadowOf(manager).allNotifications.size)
    }
}

package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.BoxScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.QueuedUpload
import com.octaviantocan.mymind.ui.library.SpaceContent
import com.octaviantocan.mymind.ui.library.SpaceListContent
import com.octaviantocan.mymind.ui.library.UploadQueueState
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.upload.AddMediaPopup
import com.octaviantocan.mymind.ui.upload.UploadQueueStrip
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.time.Instant

/** The + popup (S21), the upload queue strip, pull to refresh and Top of Mind, in the system mode of the parameter. */
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class UploadScreenshotTest(private val dark: Boolean) {
    @get:Rule
    val compose = createComposeRule()

    private val mode = if (dark) "dark" else "light"
    private val scheme = if (dark) PageScheme.Dark else PageScheme.Light

    private fun shell(tab: Tab = Tab.Everything, overlay: @Composable BoxScope.() -> Unit = {}, page: @Composable () -> Unit) {
        compose.setPhoneContent(dark) { ShellFrame(tab, onSelectTab = {}, scheme = scheme, overlay = overlay, page = page) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun capture(name: String) = compose.onRoot().captureRoboImage("screenshots/${name}_$mode.png")

    private fun strip(state: UploadQueueState): @Composable BoxScope.() -> Unit = {
        UploadQueueStrip(state, onToggle = {}, onDismiss = {}, modifier = Modifier.align(Alignment.BottomCenter))
    }

    @Test
    fun addPopup() {
        shell(overlay = { AddMediaPopup(onPhotoLibrary = {}, onTakePhoto = {}, onDismiss = {}) }) { SampleEverything() }
        capture("S21_add_popup")
    }

    @Test
    fun addPopupWithoutCamera() {
        shell(overlay = { AddMediaPopup(onPhotoLibrary = {}, onTakePhoto = null, onDismiss = {}) }) { SampleEverything() }
        capture("S21_add_popup_no_camera")
    }

    @Test
    fun uploadStrip() {
        shell(overlay = strip(UploadQueueState(queue.take(2)))) { SampleEverything() }
        capture("S06_upload_strip")
    }

    @Test
    fun uploadStripDetails() {
        shell(overlay = strip(UploadQueueState(queue, details = true))) { SampleEverything() }
        capture("S06_upload_strip_details")
    }

    @Test
    fun uploadStripInASpace() {
        val space = Space("s1", "Reading", "#ff5924")
        shell(Tab.Spaces, overlay = strip(UploadQueueState(queue.take(1)))) {
            SpaceContent(space, FeedState(LibrarySamples.grid, endReached = true), onBack = {}, onLoadMore = {}, onOpenCard = {}, onAdd = {})
        }
        capture("S08_upload_strip_space")
    }

    @Test
    fun everythingRefreshing() {
        shell { SampleEverything(feed = LibrarySamples.feed.copy(refreshing = true)) }
        capture("S06_pull_refresh")
    }

    @Test
    fun spacesRefreshing() {
        shell(Tab.Spaces) { SpaceListContent(LibrarySamples.spaces.copy(loaded = true, refreshing = true), onOpenSpace = {}, onRetry = {}, onAdd = {}) }
        capture("S07_pull_refresh")
    }

    @Test
    fun topOfMind() {
        val pins = LibrarySamples.topOfMind + listOf(
            testCard("t4", type = "Note", flags = 512, pinPosition = 4, title = null).copy(note = "A pinned note"),
            testCard("t5", type = "Article", flags = 512, pinPosition = 5, title = "Another pinned article"),
        )
        shell { SampleEverything(feed = FeedState(LibrarySamples.grid), topOfMind = pins) }
        capture("S06_top_of_mind")
    }

    private val added = Instant.parse("2026-10-17T12:00:00Z").toEpochMilli()

    private val queue = listOf(
        QueuedUpload(1, "photo.jpg", "image/jpeg", added, running = true),
        QueuedUpload(2, "clip.mp4", "video/mp4", added),
        QueuedUpload(3, "paper.pdf", "application/pdf", added, error = "The server returned an error (413)."),
    )

    companion object {
        private const val SETTLE_MS = 1_500L

        /** System light and system dark. */
        @JvmStatic
        @ParameterizedRobolectricTestRunner.Parameters(name = "dark={0}")
        fun modes() = listOf(arrayOf<Any>(false), arrayOf<Any>(true))
    }
}

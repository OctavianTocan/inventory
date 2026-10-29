package com.octaviantocan.mymind.ui

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.share.ShareState
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareProblem
import com.octaviantocan.mymind.ui.share.ShareSheet
import com.octaviantocan.mymind.ui.share.ShareSheetState
import com.octaviantocan.mymind.ui.share.SharedFile
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The share sheet (S22), version 10 One field for everything, and its problem sheets, in the system mode of the parameter. */
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ShareScreenshotTest(private val dark: Boolean) {
    @get:Rule
    val compose = createComposeRule()

    private val mode = if (dark) "dark" else "light"

    private val reading = Space("s1", "Reading", "#ff5924", count = 214)
    private val walks = Space("s2", "Walks", "#5e8f4a", count = 38)
    private val spaces = listOf(reading, walks, Space("s3", "Ideas", "#4080df", count = 12))
    private val link = "https://www.theatlantic.com/magazine/walking/"

    private fun show(name: String, keyboardPx: Int = 0, state: ShareSheetState) {
        compose.setPhoneContent(dark, keyboardPx) { ShareSheet(state, onClose = {}) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/S22_${name}_$mode.png")
    }

    private fun linkSheet(phase: SavePhase, read: Boolean) = ShareSheetState(listOf(ShareItem.Text(link)), phase).apply {
        this.spaces = this@ShareScreenshotTest.spaces
        allTags = listOf("walking", "walks", "essay", "nature", "to read", "woods")
        recentTags = listOf("to read", "essay", "recipes")
        suggestion = SpaceSuggestion(reading, 12)
        val page = testCard("c1").copy(type = "Placeholder", title = null, url = link, domain = "theatlantic.com")
        cards = listOf(
            if (read) page.copy(type = "Article", title = "Walking", siteName = "The Atlantic", author = "Henry David Thoreau",
                tags = listOf(Tag("t1", "essay", 2), Tag("t2", "nature", 2))) else page,
        )
        saved = if (phase == SavePhase.Saving) 0 else 1
    }

    @Test
    fun saving() = show("saving", state = linkSheet(SavePhase.Saving, read = false))

    @Test
    fun saved() = show("saved", state = linkSheet(SavePhase.Saved, read = false))

    @Test
    fun read() = show("read", state = linkSheet(SavePhase.Saved, read = true))

    @Test
    fun typing() = show("typing", keyboardPx = 900, state = linkSheet(SavePhase.Saved, read = true).apply { touched = true; typed = "wal" })

    @Test
    fun added() = show("added", state = linkSheet(SavePhase.Saved, read = true).apply {
        touched = true
        spaceIds = listOf("s2")
        tags = listOf("walking", "to read")
        note = "Read before Sunday’s walk"
        typed = "Read it twice"
    })

    @Test
    fun kept() = show("kept", state = linkSheet(SavePhase.Kept, read = false))

    @Test
    fun words() = show("words", state = ShareSheetState(listOf(ShareItem.Text("In Wildness is the preservation of the World.")), SavePhase.Saved).apply {
        recentTags = listOf("quote", "thoreau")
        saved = 1
    })

    @Test
    fun severalPhotos() = show("photos", state = ShareSheetState(List(4) { ShareItem.Stream("content://photos/$it") }, SavePhase.Saving).apply {
        files = List(4) { SharedFile("content://photos/$it", "IMG_204$it.jpg", "image/jpeg", 2_400_000L) }
        this.spaces = this@ShareScreenshotTest.spaces
        recentTags = listOf("landscape", "trip")
        saved = 2
    })

    @Test
    fun failed() = show("failed", state = ShareSheetState(listOf(ShareItem.Stream("content://videos/1")), SavePhase.Failed).apply {
        files = listOf(SharedFile("content://videos/1", "Hike.mp4", "video/mp4", 24_000_000L))
        problem = "Cannot reach the server. Check your connection."
    })

    private fun problem(name: String, state: ShareState) {
        compose.setPhoneContent(dark) { ShareProblem(state, onAction = {}, onClose = {}) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/S22_${name}_$mode.png")
    }

    @Test
    fun malformed() = problem("malformed", ShareState.Malformed)

    @Test
    fun tooManyItems() = problem("too_many", ShareState.TooManyItems)

    @Test
    fun fileTooBig() = problem("too_big", ShareState.FileTooBig)

    @Test
    fun pdfHasPassword() = problem("pdf_password", ShareState.PdfHasPassword)

    @Test
    fun noLogin() = problem("no_login", ShareState.NoLogin)

    companion object {
        private const val SETTLE_MS = 1_500L

        /** System light and system dark. */
        @JvmStatic
        @ParameterizedRobolectricTestRunner.Parameters(name = "dark={0}")
        fun modes() = listOf(arrayOf<Any>(false), arrayOf<Any>(true))
    }
}

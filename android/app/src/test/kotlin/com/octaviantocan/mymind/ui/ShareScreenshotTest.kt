package com.octaviantocan.mymind.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.share.ShareState
import com.octaviantocan.mymind.ui.library.QueuedUpload
import com.octaviantocan.mymind.ui.library.UploadQueueState
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareProblem
import com.octaviantocan.mymind.ui.share.ShareSheet
import com.octaviantocan.mymind.ui.share.ShareSheetState
import com.octaviantocan.mymind.ui.share.ShareTab
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The share sheet (S22) and its problem cards, in the system mode of the parameter. */
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ShareScreenshotTest(private val dark: Boolean) {
    @get:Rule
    val compose = createComposeRule()

    private val mode = if (dark) "dark" else "light"

    private val spaces = listOf(Space("s1", "Reading", "#ff5924"), Space("s2", "Recipes", "#75ffc0"), Space("s3", "Travel", "#4a99e9"))

    private fun show(name: String, keyboardPx: Int = 0, content: @Composable () -> Unit) {
        compose.setPhoneContent(dark, keyboardPx, content)
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        compose.onRoot().captureRoboImage("screenshots/S22_${name}_$mode.png")
    }

    private fun sheet(state: ShareSheetState) = @Composable { ShareSheet(state, onSave = {}, onToggleUploads = {}, onDismissUpload = {}) }

    private fun filled(tab: ShareTab, phase: SavePhase) = ShareSheetState(
        tab = tab,
        allTags = listOf("recipes", "reading", "travel"),
        recentTags = listOf("reading", "travel"),
        spaces = spaces,
        phase = phase,
    ).apply {
        tags = listOf("recipes", "dinner")
        note = "Try this on Friday.\nDouble the garlic."
        selectedSpaces = setOf("s2")
    }

    private val one = UploadQueueState(listOf(QueuedUpload(1, "photo.jpg", "image/jpeg", 0L, running = true)))

    @Test
    fun saving() = show("saving") {
        sheet(ShareSheetState(phase = SavePhase.Saving).apply { uploads = one })()
    }

    @Test
    fun savedTags() = show("saved_tags") { sheet(filled(ShareTab.Tags, SavePhase.Saved))() }

    @Test
    fun savedNotes() = show("saved_notes") { sheet(filled(ShareTab.Notes, SavePhase.Saved))() }

    @Test
    fun savedSpaces() = show("saved_spaces") { sheet(filled(ShareTab.Spaces, SavePhase.Saved))() }

    @Test
    fun tagsWithKeyboard() = show("tags_keyboard", keyboardPx = 900) {
        sheet(filled(ShareTab.Tags, SavePhase.Editing).apply { tagText = "re" })()
    }

    @Test
    fun saveFailed() = show("failed") { sheet(filled(ShareTab.Spaces, SavePhase.Failed))() }

    @Test
    fun severalItemsUploading() = show("several") {
        val queue = UploadQueueState(
            listOf(
                QueuedUpload(1, "photo.jpg", "image/jpeg", 0L, running = true),
                QueuedUpload(2, "clip.mp4", "video/mp4", 0L),
                QueuedUpload(3, "paper.pdf", "application/pdf", 0L, error = "The server returned an error (413)."),
            ),
            details = true,
        )
        sheet(ShareSheetState(phase = SavePhase.Saving).apply { uploads = queue })()
    }

    private fun problem(name: String, state: ShareState) = show(name) { ShareProblem(state, onAction = {}, onClose = {}) }

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

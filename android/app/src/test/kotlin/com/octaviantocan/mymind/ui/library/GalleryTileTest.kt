package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.GalleryImage
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.theme.MymindTheme
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(ParameterizedRobolectricTestRunner::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class GalleryTileTest(private val width: Int) {
    @get:Rule val compose = createComposeRule()
    private val source = "https://gallery.example.test/forest.jpg"

    private fun render(card: Card) {
        compose.setContent {
            MymindTheme(dark = true) {
                Box(Modifier.width(width.dp).height((width * 1.35f).dp)) { GalleryTile(card, {}) }
            }
        }
    }

    private fun assertBoth(body: String) {
        val text = compose.onNodeWithText(body, useUnmergedTree = true).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val image = compose.onNodeWithContentDescription("Image preview", useUnmergedTree = true)
            .assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        assertTrue("Text and image need separate visible areas", text.bottom <= image.top)
        assertTrue(image.height > 0f && image.width > 0f)
    }

    @Test fun aNoteKeepsItsBodyAndCover() {
        val body = "A quiet place to think."
        render(testCard("mixed-note", type = "Note", title = "Weekend", note = body)
            .copy(cover = Cover(source, 1200, 800)))
        assertBoth(body)
    }

    @Test fun aSocialPostKeepsItsBodyAndGallery() {
        val body = "Keep a little space for new ideas."
        render(testCard("mixed-post", type = "XPost", title = "A saved post")
            .copy(post = Post(bodyHtml = "<p>$body</p>", gallery = listOf(GalleryImage(source, 1200, 800)))))
        assertBoth(body)
    }

    @Test fun anArticleKeepsItsHeadlineAndCover() {
        val body = "Small spaces, better ideas"
        render(testCard("mixed-article", type = "Article", title = body).copy(cover = Cover(source, 1200, 800)))
        assertBoth(body)
    }

    @Test fun aSavedWebPageKeepsItsHeadlineAndCover() {
        val body = "Collect the things that move you"
        render(testCard("mixed-link", type = "WebPage", title = body).copy(cover = Cover(source, 1200, 800)))
        assertBoth(body)
    }

    @Test fun anAnnotatedPhotoKeepsItsWrittenNoteAndImage() {
        val body = "A place worth returning to."
        render(testCard("annotated-photo", type = "Image", title = "Mountain light", note = body)
            .copy(cover = Cover(source, 1200, 800)))
        assertBoth(body)
    }

    @Test fun anAnnotatedVideoKeepsItsNoteCoverAndVideoIndicator() {
        val body = "A short film worth watching again."
        render(testCard("annotated-video", type = "YouTubeVideo", title = "A saved film", note = body)
            .copy(cover = Cover(source, 1200, 800)))
        assertBoth(body)
        compose.onNodeWithContentDescription("Video", useUnmergedTree = true).assertIsDisplayed()
    }

    @Test fun aTextOnlyNoteKeepsItsBody() {
        val body = "Remember this thought."
        render(testCard("text-note", type = "Note", title = "A thought", note = body))
        compose.onNodeWithText(body, useUnmergedTree = true).assertIsDisplayed()
        compose.onNodeWithContentDescription("Image preview", useUnmergedTree = true).assertDoesNotExist()
    }

    companion object {
        @JvmStatic @ParameterizedRobolectricTestRunner.Parameters(name = "width={0}")
        fun sizes() = listOf(arrayOf(88), arrayOf(184))
    }
}

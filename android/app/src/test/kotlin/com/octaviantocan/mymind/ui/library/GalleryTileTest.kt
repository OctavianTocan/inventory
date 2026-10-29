package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.unit.dp
import androidx.test.core.app.ApplicationProvider
import com.octaviantocan.mymind.data.AppInfo
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.GalleryImage
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.data.Product
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.theme.MymindTheme
import okhttp3.OkHttpClient
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

    private fun render(card: Card, previews: VideoPreviews? = null) {
        compose.setContent {
            CompositionLocalProvider(LocalVideoPreviews provides previews) {
                MymindTheme(dark = true) {
                    Box(Modifier.width(width.dp).height((width * 1.35f).dp)) { GalleryTile(card, {}, playVideo = false) }
                }
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

    @Test fun anAnnotatedVideoKeepsItsNoteCoverAndBadge() {
        val body = "A short film worth watching again."
        render(testCard("annotated-video", type = "YouTubeVideo", title = "A saved film", note = body)
            .copy(cover = Cover(source, 1200, 800)))
        assertBoth(body)
        compose.onNodeWithContentDescription("YouTube video", useUnmergedTree = true).assertIsDisplayed()
    }

    @Test fun aVideoFileShowsItsLengthWhereItFits() {
        val card = testCard("clip", type = "Video", title = "clip.mp4")
            .copy(cover = Cover("/media/u/clip;600x1000-t.webp", 600, 1000), blob = Blob("u/clip", "video/mp4", 1080, 1800))
        val previews = VideoPreviews(ApplicationProvider.getApplicationContext(), OkHttpClient())
        previews.rememberLength(checkNotNull(previewVideo(card)), 65_000)
        render(card, previews)
        val badge = if (width >= 96) "Video, 1:05" else "Video"
        compose.onNodeWithContentDescription(badge, useUnmergedTree = true).assertIsDisplayed()
    }

    @Test fun aVideoKeptWithoutACoverShowsAStillAndItsBadge() {
        render(testCard("raw", type = "Video", title = "raw.mp4").copy(blob = Blob("u/raw", "video/mp4", 1080, 1920)))
        compose.onNodeWithContentDescription("Video", useUnmergedTree = true).assertIsDisplayed()
    }

    @Test fun aLinkWhoseOnlyTextIsItsAddressShowsItOnce() {
        render(testCard("bare-link", type = "WebPage", url = "https://example.test/", title = "example.test").copy(domain = "example.test"))
        compose.onAllNodesWithText("example.test", useUnmergedTree = true).assertCountEquals(1)
    }

    @Test fun aTextOnlyNoteKeepsItsBody() {
        val body = "Remember this thought."
        render(testCard("text-note", type = "Note", title = "A thought", note = body))
        compose.onNodeWithText(body, useUnmergedTree = true).assertIsDisplayed()
        compose.onNodeWithContentDescription("Image preview", useUnmergedTree = true).assertDoesNotExist()
    }

    @Test fun aProductIsAPinWithItsPriceOnThePhoto() {
        render(testCard("pin", type = "Product", title = "Paper lamp")
            .copy(cover = Cover(source, 900, 1200), product = Product("GBP 24.00", "Norrland"), siteName = "Hollow & Pine"))
        val price = compose.onNodeWithText("£24", useUnmergedTree = true).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val photo = compose.onNodeWithContentDescription("Image preview", useUnmergedTree = true).fetchSemanticsNode().boundsInRoot
        val name = compose.onNodeWithText("Paper lamp", useUnmergedTree = true).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        assertTrue("The price sits on the photo", photo.contains(price.center))
        assertTrue("The name is under the photo", name.top >= photo.bottom)
    }

    @Test fun anAppShowsItsIconAndName() {
        render(testCard("app", type = "SoftwareApplication", title = "Trailhead")
            .copy(cover = Cover(source, 250, 250), app = AppInfo("play-store")))
        compose.onNodeWithContentDescription("App icon", useUnmergedTree = true).assertIsDisplayed()
        compose.onNodeWithText("Trailhead", useUnmergedTree = true).assertIsDisplayed()
    }

    @Test fun aYouTubeVideoShowsItsTitleUnderItsStill() {
        render(testCard("yt-caption", type = "YouTubeVideo", title = "Folding a paper lamp")
            .copy(cover = Cover(source, 1280, 720), author = "by Maya Walks"))
        val title = compose.onNodeWithText("Folding a paper lamp", useUnmergedTree = true).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        val badge = compose.onNodeWithContentDescription("YouTube video", useUnmergedTree = true).assertIsDisplayed().fetchSemanticsNode().boundsInRoot
        assertTrue("The badge stays on the still, above the title", badge.bottom <= title.top)
    }

    companion object {
        @JvmStatic @ParameterizedRobolectricTestRunner.Parameters(name = "width={0}")
        fun sizes() = listOf(arrayOf(88), arrayOf(184))
    }
}

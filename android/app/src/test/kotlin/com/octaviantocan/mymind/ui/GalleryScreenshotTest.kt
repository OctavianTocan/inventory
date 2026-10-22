package com.octaviantocan.mymind.ui

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.remember
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.test.core.app.ApplicationProvider
import coil3.EventListener
import coil3.ImageLoader
import coil3.SingletonImageLoader
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.GalleryImage
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.MosaicState
import com.octaviantocan.mymind.ui.library.SearchType
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.shell.LibraryFootBar
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.util.Collections

/** Actual Compose gallery and Coil photo decoding, with public fixture bytes and no network requests. */
@OptIn(coil3.annotation.DelicateCoilApi::class)
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class GalleryScreenshotTest(private val dark: Boolean) {
    @get:Rule val compose = createComposeRule()
    private val loaded = Collections.synchronizedSet(mutableSetOf<String>())
    private lateinit var loader: ImageLoader
    private lateinit var client: OkHttpClient

    @Before fun photos() {
        val pictures = listOf("mountains", "forest", "coast", "hills").associateWith { name ->
            checkNotNull(javaClass.getResourceAsStream("/gallery/$name.jpg")).use { it.readBytes() }
        }
        client = OkHttpClient.Builder().addInterceptor { chain ->
            val request = chain.request()
            check(request.url.host == "gallery.example.test")
            check(request.header("Authorization") == null)
            val bytes = pictures.getValue(request.url.encodedPath.removePrefix("/").removeSuffix(".jpg"))
            Response.Builder().request(request).protocol(Protocol.HTTP_1_1).code(200).message("Fixture")
                .body(bytes.toResponseBody("image/jpeg".toMediaType())).build()
        }.build()
        loader = ImageLoader.Builder(ApplicationProvider.getApplicationContext())
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { client })) }
            .eventListener(object : EventListener() {
                override fun onSuccess(request: ImageRequest, result: SuccessResult) { loaded += request.data.toString() }
            }).build()
        SingletonImageLoader.setUnsafe(loader)
    }

    @After fun close() {
        SingletonImageLoader.reset()
        loader.shutdown()
        client.connectionPool.evictAll()
        client.dispatcher.executorService.shutdown()
    }

    private fun photo(id: String, picture: String, width: Int, height: Int, title: String) =
        testCard(id, type = "Image", title = title).copy(cover = Cover("https://gallery.example.test/$picture.jpg", width, height, dominantColor = "#4c6254"))

    private val pins: List<Card> get() = listOf(
        photo("pin-landscape", "mountains", 1200, 900, "Places to return to").copy(flags = 512, pinPosition = 1),
        photo("pin-coast", "coast", 900, 1200, "Along the coast").copy(flags = 512, pinPosition = 2),
    )

    private val cards: List<Card> get() = listOf(
        photo("forest", "forest", 800, 1200, "A quiet trail"),
        photo("mountains", "mountains", 1200, 800, "Mountain light"),
        testCard("note", type = "Note", title = "A thought to keep", note = "Leave a little room for things you haven't found yet."),
        photo("coast", "coast", 1200, 900, "On the water"),
        photo("article", "hills", 1200, 800, "Small spaces, better ideas").copy(type = "Article"),
        testCard("post", type = "XPost", title = "Designing for attention").copy(post = Post(bodyHtml = "The best tools give you space to think. Save what moves you, and find it again when it matters.")),
        photo("forest-again", "forest", 1000, 900, "Somewhere green"),
        testCard("reading", type = "Article", title = "Good tools should get out of your way"),
        photo("hills", "hills", 1200, 800, "Evening hills"),
        photo("coast-again", "coast", 900, 1200, "Ocean blue"),
        photo("mountains-again", "mountains", 1100, 900, "A place worth saving"),
    )

    private val mixedNotes: List<Card> get() = listOf(
        testCard("thought", type = "Note", title = "A thought to keep", note = "Leave a little room for things you haven't found yet."),
        photo("walk", "forest", 800, 1200, "Weekend walk").copy(type = "Note", note = "Take the long way home. There is always something worth noticing."),
        photo("reading", "hills", 1200, 800, "Small spaces, better ideas").copy(type = "Article"),
        testCard("post-picture", type = "XPost", title = "Designing for attention").copy(post = Post(
            bodyHtml = "<p>The best tools leave you room to think. Keep the words and the picture together.</p>",
            gallery = listOf(GalleryImage("https://gallery.example.test/coast.jpg", 1200, 900)),
        )),
        photo("saved-link", "hills", 1200, 800, "Collect the things that move you").copy(type = "WebPage", domain = "reading.example"),
        testCard("quote", type = "Quote", title = "An idea to keep", note = "Make room for things that make you curious."),
        photo("mountain-note", "mountains", 1200, 800, "Back to the mountains").copy(type = "Content", note = "A slower morning. A clearer mind. Save this for the next trip."),
        testCard("repository", type = "Repository", title = "tools-for-thought").copy(domain = "github.com"),
        photo("image-only", "forest", 800, 1200, "Somewhere green"),
        photo("annotated-photo", "mountains", 1200, 800, "Morning light").copy(note = "A small reminder: take a breath before rushing into the day."),
        photo("saved-video", "coast", 1200, 900, "A saved film").copy(type = "YouTubeVideo", note = "A short film worth watching again."),
        testCard("snippet", type = "Content", title = "A useful reminder", note = "Good tools should get out of your way."),
    )

    private fun capture(columns: Int, name: String, searching: Boolean = false,
        sample: List<Card> = cards, highlights: List<Card> = pins) {
        compose.setPhoneContent(dark) {
            ShellFrame(Tab.Everything, {}, PageScheme.System, footer = {
                LibraryFootBar(Tab.Everything, {}, searching, "", {}, {}, {})
            }) {
                EverythingContent(FeedState(sample, endReached = true), searching, null,
                    SearchType.entries.map { it.suggestion }, null, emptyList(), {}, {}, {}, {}, {}, {}, {}, {},
                    topOfMind = highlights, gridState = remember { LazyListState() }, mosaic = remember { MosaicState(columns) })
            }
        }
        compose.mainClock.autoAdvance = true
        if (!searching) compose.waitUntil(10_000) { loaded.size >= 3 }
        compose.mainClock.advanceTimeBy(1200)
        compose.waitForIdle()
        compose.onRoot().captureRoboImage("screenshots/${name}_${if (dark) "dark" else "light"}.png")
    }

    @Test fun gallery() = capture(2, "gallery")
    @Test fun compactGallery() = capture(4, "gallery_compact")
    @Test fun search() = capture(2, "gallery_search", searching = true)
    @Test fun differentNoteTypes() = capture(2, "gallery_notes", sample = mixedNotes, highlights = emptyList())
    @Test fun compactNoteTypes() = capture(4, "gallery_notes_compact", sample = mixedNotes, highlights = emptyList())

    companion object {
        @JvmStatic @ParameterizedRobolectricTestRunner.Parameters(name = "dark={0}") fun modes() = listOf(arrayOf(false), arrayOf(true))
    }
}

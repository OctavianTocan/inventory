package com.octaviantocan.mymind.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.captureRoboImage
import com.github.takahirom.roborazzi.captureScreenRoboImage
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.shell.MainShell
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.theme.WindowShape
import com.octaviantocan.mymind.ui.theme.listPaneWidth
import com.octaviantocan.mymind.ui.theme.windowShape
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okhttp3.mockwebserver.RecordedRequest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import org.robolectric.shadows.ShadowDialog

/**
 * The window's three shapes, ported from Pully's foldables and sideways phones (pulls.review PR #137 to #141): a phone
 * held upright as before, a sideways phone with one page in a readable column, and an unfolded foldable with the gallery
 * and the opened card side by side.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class WindowShapesTest {
    @get:Rule val compose = createComposeRule()
    private var server: MockWebServer? = null

    @After fun closeServer() { server?.shutdown() }

    private fun showShell() {
        val cards = (1..12).map { testCard("n$it", type = "Note", title = "Note $it", note = "Words of note $it") }
        val model = LibraryViewModel(FakeLibrary(pages = mapOf(null to CardPage(cards))))
        val local = MockWebServer().apply {
            dispatcher = object : Dispatcher() {
                override fun dispatch(request: RecordedRequest) = MockResponse().setBody("""{"email":"reader@example.test","kind":"password"}""")
            }
            start()
        }
        server = local
        val store = object : TokenStore {
            override suspend fun read(): String? = null
            override suspend fun write(token: String) = Unit
            override suspend fun clear() = Unit
        }
        val settings = SettingsViewModel(Repository(MymindApi(OkHttpClient(), local.url("/").toString()), Session(store)))
        compose.setContent { MymindTheme(dark = false) { MainShell(model, settings) } }
        compose.waitForIdle()
    }

    private fun reader() = compose.onNode(SemanticsMatcher.expectValue(SemanticsProperties.PaneTitle, "Card details"))

    @Test fun shapesFollowPullysThresholds() {
        assertEquals(WindowShape.Phone, windowShape(384.dp, 896.dp))
        assertEquals(WindowShape.Phone, windowShape(599.dp, 400.dp))
        assertEquals(WindowShape.Sideways, windowShape(891.dp, 411.dp))
        assertEquals(WindowShape.Unfolded, windowShape(841.dp, 701.dp))
        assertEquals(WindowShape.Unfolded, windowShape(707.dp, 841.dp))
        assertEquals(387.dp.value, listPaneWidth(841.dp).value, 0.5f)
        assertEquals(340.dp, listPaneWidth(600.dp))
        assertEquals(420.dp, listPaneWidth(1280.dp))
    }

    @Test
    @Config(sdk = [36], qualifiers = "w841dp-h701dp-xhdpi")
    fun unfoldedShowsTheCardBesideTheGallery() {
        showShell()
        compose.onNodeWithText("A card you open shows here.").assertIsDisplayed()
        compose.onNodeWithContentDescription("Note 1").performClick()
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        // Beside the gallery, not a sheet in a window of its own over it.
        assertNull(ShadowDialog.getLatestDialog())
        reader().assertIsDisplayed()
        compose.onNodeWithContentDescription("Card gallery").assertIsDisplayed()
        compose.onNodeWithText("A card you open shows here.").assertDoesNotExist()
        // The gallery is in sight, so the reader leaves Back out (Pully's rule 210).
        compose.onNodeWithContentDescription("Back").assertDoesNotExist()
        val gallery = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode().boundsInRoot
        val pane = reader().fetchSemanticsNode().boundsInRoot
        assertTrue("the card sits right of the gallery: $gallery $pane", pane.left >= gallery.right)
        compose.onRoot().captureRoboImage("screenshots/window_unfolded.png")
        // Opening another card replaces the one beside the gallery.
        compose.onNodeWithContentDescription("Note 2").performClick()
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        // The tile shows its words too, so the reader adds a second node only for the card it shows.
        compose.onAllNodesWithText("Words of note 2").assertCountEquals(2)
        compose.onAllNodesWithText("Words of note 1").assertCountEquals(1)
    }

    @Test
    @Config(sdk = [36], qualifiers = "w891dp-h411dp-xhdpi")
    fun sidewaysKeepsTheGalleryAndTheReaderInAReadableColumn() {
        showShell()
        val gallery = compose.onNodeWithContentDescription("Card gallery").fetchSemanticsNode().boundsInRoot
        val density = compose.onRoot().fetchSemanticsNode().layoutInfo.density.density
        assertTrue("gallery ${gallery.width / density}dp", gallery.width / density <= 600f + 1f)
        assertEquals("gallery is centred", 891f * density / 2f, gallery.center.x, density * 2)
        compose.onRoot().captureRoboImage("screenshots/window_sideways.png")
        compose.onNodeWithContentDescription("Note 1").performClick()
        compose.mainClock.advanceTimeBy(1000)
        compose.waitForIdle()
        val sheet = reader().fetchSemanticsNode().boundsInRoot
        assertTrue("reader ${sheet.width / density}dp", sheet.width / density <= 600f + 1f)
        compose.onNodeWithContentDescription("Back").assertIsDisplayed()
        captureScreenRoboImage("screenshots/window_sideways_reader.png")
    }
}

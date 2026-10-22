package com.octaviantocan.mymind.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
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
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.theme.PageScheme
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okhttp3.mockwebserver.RecordedRequest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class GalleryIntegrationTest {
    @get:Rule val compose = createComposeRule()
    private var server: MockWebServer? = null
    @After fun closeServer() { server?.shutdown() }

    private fun showShell(): LibraryViewModel {
        val library = FakeLibrary(pages = mapOf(null to CardPage((1..100).map { testCard("p$it", type = "Image", title = "Photo $it") })))
        val model = LibraryViewModel(library)
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
        compose.setContent { MymindTheme(dark = true) { MainShell(model, settings) } }
        compose.waitForIdle()
        return model
    }

    @Test fun theLibraryPlacesTheProfileAndHighlightsAboveThePinchGallery() {
        compose.setPhoneContent(true) {
            ShellFrame(Tab.Everything, {}, PageScheme.Dark) { SampleEverything() }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithContentDescription("Account settings").assertIsDisplayed()
        compose.onNodeWithContentDescription("Pinned cards").assertIsDisplayed()
        compose.onNodeWithContentDescription("Card gallery").assertExists()
        compose.onNodeWithContentDescription("Rediscover cards").assertIsDisplayed()
        compose.onNodeWithText("Create").assertDoesNotExist()
    }

    @Test fun searchFromSpacesUsesTheSharedFieldAndClosesBackToTheGallery() {
        val model = showShell()
        compose.onNodeWithText("Spaces").performClick()
        compose.onNodeWithContentDescription("Search").performClick()
        val field = compose.onNodeWithContentDescription("Search your mind")
        field.assertIsFocused().performTextInput("forest")
        compose.runOnIdle { assertEquals("forest", model.search.query.value) }
        compose.onNodeWithContentDescription("Close search").performClick()
        compose.onNodeWithContentDescription("Account settings").assertIsDisplayed()
        compose.onNodeWithContentDescription("Card gallery").assertExists()
        compose.onNodeWithContentDescription("Search your mind").assertDoesNotExist()
        compose.runOnIdle { assertEquals("", model.search.query.value) }
    }

    @Test fun changingTabsRetainsGalleryDensityAndScrollPosition() {
        showShell()
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        val smaller = gallery.fetchSemanticsNode().config[SemanticsActions.CustomActions].first { it.label == "Smaller cards" }
        compose.runOnIdle { assertTrue(smaller.action()) }
        gallery.performScrollToIndex(10)
        val before = gallery.fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value()
        assertTrue(before > 0f)
        compose.onNodeWithText("Spaces").performClick()
        compose.onNodeWithText("Everything").performClick()
        compose.runOnIdle { }
        assertEquals("3 columns", gallery.fetchSemanticsNode().config[SemanticsProperties.StateDescription])
        assertEquals(before, gallery.fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value(), 1f)
    }
}

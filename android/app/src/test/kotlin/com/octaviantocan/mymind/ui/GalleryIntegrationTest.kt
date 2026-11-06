package com.octaviantocan.mymind.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.swipeLeft
import androidx.compose.ui.test.swipeRight
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.geometry.Offset
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.Space
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
    private lateinit var library: FakeLibrary
    @After fun closeServer() { server?.shutdown() }

    private fun showShell(searchHits: List<Card> = listOf(testCard("hit")), spaces: List<Space> = emptyList()): LibraryViewModel {
        library = FakeLibrary(pages = mapOf(null to CardPage((1..100).map { testCard("p$it", type = "Image", title = "Photo $it") })),
            searchHits = searchHits, spaceList = spaces)
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
        compose.onNodeWithContentDescription("Rediscover cards").assertDoesNotExist()
        compose.onNodeWithText("Create").assertDoesNotExist()
    }

    @Test fun searchFromSpacesUsesTheSharedFieldAndClosesBackToSpaces() {
        val model = showShell()
        compose.onNodeWithText("Spaces").performClick()
        compose.onNodeWithContentDescription("Search").performClick()
        val field = compose.onNodeWithContentDescription("Search your mind")
        field.assertIsFocused().performTextInput("forest")
        compose.runOnIdle { assertEquals("forest", model.search.query.value) }
        compose.onNodeWithContentDescription("Close search").performClick()
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onNodeWithContentDescription("Search your mind").assertDoesNotExist()
        compose.runOnIdle { assertEquals("", model.search.query.value) }
        compose.onNodeWithText("All").performClick()
        compose.onNodeWithContentDescription("Card gallery").assertExists()
    }

    @Test fun aSwipeMovesBetweenEverythingSpacesAndAsk() {
        // "I want to be able to swipe between the everything, spaces, and ask pages ... Just as it does in Pully" (Tavi,
        // 5 Nov 2026).
        showShell()
        compose.onRoot().performTouchInput { swipeLeft(startX = width * 0.9f, endX = width * 0.1f) }
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onNodeWithContentDescription("Card gallery").assertDoesNotExist()
        compose.onRoot().performTouchInput { swipeLeft(startX = width * 0.9f, endX = width * 0.1f) }
        compose.onNodeWithText("Find something").assertIsDisplayed()
        // "When I swipe over to the search screen, the search input should be automatically focused" (Tavi, 5 Nov 2026).
        compose.onNodeWithContentDescription("Search your mind").assertIsFocused()
        compose.onRoot().performTouchInput { swipeRight(startX = width * 0.1f, endX = width * 0.9f) }
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onNodeWithContentDescription("Search your mind").assertDoesNotExist()
        compose.onRoot().performTouchInput { swipeRight(startX = width * 0.1f, endX = width * 0.9f) }
        compose.onNodeWithText("All").assertIsSelected()
        compose.onNodeWithContentDescription("Card gallery").assertIsDisplayed()
    }

    @Test fun aShortSlowDragSettlesBackOnTheSamePage() {
        showShell()
        compose.onRoot().performTouchInput {
            down(Offset(width * 0.6f, height * 0.5f))
            moveBy(Offset(-width * 0.2f, 0f), delayMillis = 400)
            moveBy(Offset(0f, 0f), delayMillis = 400)
            up()
        }
        compose.onNodeWithText("All").assertIsSelected()
        compose.onNodeWithContentDescription("Card gallery").assertIsDisplayed()
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
        compose.onNodeWithText("All").performClick()
        compose.runOnIdle { }
        assertEquals("3 columns", gallery.fetchSemanticsNode().config[SemanticsProperties.StateDescription])
        assertEquals(before, gallery.fetchSemanticsNode().config[SemanticsProperties.VerticalScrollAxisRange].value(), 1f)
    }

    @Test fun searchResultsPinchLikeTheGallery() {
        // "The zoom in and stuff should work on the results too" (Tavi, 28 Oct 2026): the results open before their cards.
        showShell(searchHits = (1..40).map { testCard("s$it", type = "Image", title = "Found $it") })
        compose.onNodeWithContentDescription("Search").performClick()
        compose.onNodeWithContentDescription("Search your mind").performTextInput("forest")
        compose.onNodeWithContentDescription("Search your mind").performImeAction()
        compose.waitForIdle()
        compose.onNodeWithContentDescription("Found 1").assertExists()
        val gallery = compose.onNodeWithContentDescription("Card gallery")
        gallery.performTouchInput {
            down(0, Offset(width * 0.18f, height * 0.5f))
            down(1, Offset(width * 0.82f, height * 0.5f))
            moveTo(0, Offset(width * 0.4f, height * 0.5f), delayMillis = 120)
            moveTo(1, Offset(width * 0.6f, height * 0.5f), delayMillis = 120)
            up(0); up(1)
        }
        compose.waitForIdle()
        assertEquals("4 columns", gallery.fetchSemanticsNode().config[SemanticsProperties.StateDescription])
        compose.onNodeWithContentDescription("Search your mind").assertExists()
    }

    @Test fun theSpacesListPinchesLikeTheGalleryAndKeepsItsSizeAcrossASpace() {
        // "for spaces we also need the pinching stuff" (Tavi, 28 Oct 2026).
        showShell(spaces = (1..12).map { Space("s$it", "Space $it", "#ff5924") })
        compose.onNodeWithText("Spaces").performClick()
        compose.waitForIdle()
        val list = compose.onNodeWithContentDescription("All spaces")
        list.performTouchInput {
            down(0, Offset(width * 0.18f, height * 0.5f))
            down(1, Offset(width * 0.82f, height * 0.5f))
            moveTo(0, Offset(width * 0.4f, height * 0.5f), delayMillis = 120)
            moveTo(1, Offset(width * 0.6f, height * 0.5f), delayMillis = 120)
            up(0); up(1)
        }
        compose.waitForIdle()
        assertEquals("4 columns", list.fetchSemanticsNode().config[SemanticsProperties.StateDescription])
        // Two fingers drawing together sideways are a pinch, never a swipe to the next page.
        compose.onNodeWithText("Spaces").assertIsSelected()
        compose.onNodeWithText("Space 1").performClick()
        compose.onNodeWithContentDescription("Back").performClick()
        compose.waitForIdle()
        assertEquals("4 columns", compose.onNodeWithContentDescription("All spaces").fetchSemanticsNode().config[SemanticsProperties.StateDescription])
    }

    @Test fun thePlusOnSpacesAndTheNewSpaceTileOpenTheNewSpaceSheet() {
        // "On the Spaces screen ... there should be a space that has a little plus in the middle ... if you press the
        // plus, we already assume that you're trying to add a space" (Tavi, 5 Nov 2026).
        showShell(spaces = listOf(Space("s1", "Walks", "#ff5924")))
        compose.onNodeWithText("Spaces").performClick()
        compose.waitForIdle()
        compose.onNodeWithContentDescription("Add").performClick()
        compose.onNodeWithText("Create space").assertIsDisplayed()
        compose.onNodeWithText("Photos").assertDoesNotExist()
        compose.onNodeWithContentDescription("Space name").performTextInput("Recipes")
        compose.onNodeWithText("Create space").performClick()
        compose.waitForIdle()
        assertTrue(library.calls.any { it.startsWith("newSpace:Recipes:") })
        compose.onNodeWithText("Create space").assertDoesNotExist()
        compose.onNodeWithText("New space").performClick()
        compose.onNodeWithText("Create space").assertIsDisplayed()
    }

    @Test fun aSpaceOptionsAreASheetAndDeletingIsHeld() {
        showShell(spaces = listOf(Space("s1", "Walks", "#ff5924")))
        compose.onNodeWithText("Spaces").performClick()
        compose.waitForIdle()
        compose.onNodeWithText("Walks").performClick()
        compose.onNodeWithContentDescription("Space options").performClick()
        compose.onNodeWithText("Add to this space").assertIsDisplayed()
        compose.onNodeWithText("Delete space").performSemanticsAction(SemanticsActions.OnClick)
        compose.waitForIdle()
        assertTrue("deleteSpace:s1" in library.calls)
        compose.onNodeWithText("Add to this space").assertDoesNotExist()
    }

    @Test fun aSpaceIsRenamedFromItsDotMenu() {
        // "We need a way to rename a space in its dot menu" (Tavi, 5 Nov 2026).
        showShell(spaces = listOf(Space("s1", "Walks", "#ff5924")))
        compose.onNodeWithText("Spaces").performClick()
        compose.waitForIdle()
        compose.onNodeWithText("Walks").performClick()
        compose.onNodeWithContentDescription("Space options").performClick()
        compose.onNodeWithText("Rename space").performClick()
        compose.onNodeWithContentDescription("Space name").performTextReplacement("Long walks")
        compose.onNodeWithText("Save name").performClick()
        compose.waitForIdle()
        assertTrue("renameSpace:s1:Long walks" in library.calls)
        compose.onNodeWithText("Save name").assertDoesNotExist()
        compose.onAllNodesWithText("Long walks").onFirst().assertIsDisplayed()
    }

    @Test fun holdingASpaceOnTheListOpensItsOptions() {
        // "also show that dot menu when holding over it in the spaces page" (Tavi, 5 Nov 2026).
        showShell(spaces = listOf(Space("s1", "Walks", "#ff5924")))
        compose.onNodeWithText("Spaces").performClick()
        compose.waitForIdle()
        compose.onNodeWithText("Walks").performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithText("Add to this space").assertIsDisplayed()
        compose.onNodeWithText("Rename space").performClick()
        compose.onNodeWithContentDescription("Space name").performTextReplacement("Hikes")
        compose.onNodeWithText("Save name").performClick()
        compose.waitForIdle()
        assertTrue("renameSpace:s1:Hikes" in library.calls)
        compose.onNodeWithText("Hikes").assertIsDisplayed()
        compose.onNodeWithText("Hikes").performTouchInput { longClick() }
        compose.onNodeWithText("Delete space").performSemanticsAction(SemanticsActions.OnClick)
        compose.waitForIdle()
        assertTrue("deleteSpace:s1" in library.calls)
        compose.onNodeWithText("Hikes").assertDoesNotExist()
    }
}

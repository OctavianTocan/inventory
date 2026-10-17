package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuItem
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class FeedRefreshTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun anUnchangedFirstPageKeepsEverythingLoaded() = runTest(dispatcher) {
        val library = FakeLibrary(
            pages = mapOf(
                null to CardPage(listOf(testCard("a"), testCard("b")), next = "p2"),
                "p2" to CardPage(listOf(testCard("c"))),
            ),
        )
        val feed = CardFeed(this) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        feed.loadMore()
        advanceUntilIdle()
        val before = feed.state.value
        feed.refresh()
        assertTrue(feed.state.value.refreshing)
        advanceUntilIdle()
        assertFalse(feed.state.value.refreshing)
        assertSame(before.cards, feed.state.value.cards)
        assertTrue(feed.state.value.endReached)
    }

    @Test
    fun aChangedFirstPageReplacesTheListAndPagingGoesOn() = runTest(dispatcher) {
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("b")))))
        val feed = CardFeed(this) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        library.pages = mapOf(
            null to CardPage(listOf(testCard("new"), testCard("a")), next = "p2"),
            "p2" to CardPage(listOf(testCard("b"))),
        )
        feed.refresh()
        advanceUntilIdle()
        assertEquals(listOf("new", "a"), feed.state.value.cards.map { it.id })
        assertFalse(feed.state.value.endReached)
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("new", "a", "b"), feed.state.value.cards.map { it.id })
    }

    @Test
    fun aFailedRefreshKeepsTheList() = runTest(dispatcher) {
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a")))))
        val feed = CardFeed(this) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        library.pages = emptyMap()
        feed.refresh()
        advanceUntilIdle()
        assertEquals(listOf("a"), feed.state.value.cards.map { it.id })
        assertFalse(feed.state.value.refreshing)
        assertEquals(null, feed.state.value.error)
    }

    @Test
    fun prependPutsTheCardFirstOnce() = runTest(dispatcher) {
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("b")))))
        val feed = CardFeed(this) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        feed.prepend(testCard("n"))
        feed.prepend(testCard("b"))
        assertEquals(listOf("b", "n", "a"), feed.state.value.cards.map { it.id })
    }

    @Test
    fun topOfMindIsReadFromItsRouteAndAgainAfterAPin() = runTest(dispatcher) {
        val library = FakeLibrary(
            pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("p1", pinPosition = 1, flags = 512)))),
            pinnedPages = mapOf(null to CardPage(listOf(testCard("p1", pinPosition = 1, flags = 512)))),
        )
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()
        assertEquals(listOf("p1"), viewModel.topOfMind.state.value.cards.map { it.id })

        library.pinnedPages = mapOf(null to CardPage(listOf(testCard("p1", pinPosition = 1, flags = 512), testCard("a", pinPosition = 2, flags = 512))))
        viewModel.perform(CardMenuItem(CardMenuAction.AddToTopOfMind), testCard("a"), null)
        advanceUntilIdle()
        assertEquals(listOf("p1", "a"), viewModel.topOfMind.state.value.cards.map { it.id })
        assertEquals(2, library.calls.count { it == "topOfMind:null" })
    }

    @Test
    fun pullingEverythingReadsTheLibraryAndTopOfMindAgain() = runTest(dispatcher) {
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a")))), pinnedPages = mapOf(null to CardPage(listOf(testCard("p")))))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()
        library.calls.clear()
        viewModel.refreshEverything()
        advanceUntilIdle()
        assertEquals(listOf("cards:null", "topOfMind:null"), library.calls.filter { it.startsWith("cards:") || it.startsWith("topOfMind:") })
    }

    @Test
    fun pullingTheSpacesListKeepsItOnShowWhileItReads() = runTest(dispatcher) {
        val library = FakeLibrary(spaceList = listOf(Space("s1", "One")), spacePages = mapOf("s1" to CardPage(listOf(testCard("a")))))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        viewModel.loadSpaces()
        advanceUntilIdle()
        library.spaceList = listOf(Space("s1", "One"), Space("s2", "Two"))
        viewModel.refreshSpaces()
        assertTrue(viewModel.spaces.value.refreshing)
        assertEquals(listOf("s1"), viewModel.spaces.value.spaces.map { it.id })
        advanceUntilIdle()
        assertFalse(viewModel.spaces.value.refreshing)
        assertEquals(listOf("s1", "s2"), viewModel.spaces.value.spaces.map { it.id })
        assertEquals(setOf("s1"), viewModel.spaces.value.covers.keys)
    }
}

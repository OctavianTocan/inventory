package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.FeedMemory
import com.octaviantocan.mymind.data.LibraryMemory
import com.octaviantocan.mymind.data.PageConfig
import com.octaviantocan.mymind.ui.theme.PageScheme
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class FeedMemoryTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun keptCardsShowBeforeTheServerAnswers() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("a"), testCard("b")))
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("b")))))
        val answer = CompletableDeferred<Unit>()
        val feed = CardFeed(this, memory) { answer.await(); library.cards(it) }
        feed.loadMore()
        runCurrent()
        assertEquals(listOf("a", "b"), feed.state.value.cards.map { it.id })
        assertTrue(feed.state.value.loading)
        answer.complete(Unit)
        advanceUntilIdle()
        assertEquals(listOf("a", "b"), feed.state.value.cards.map { it.id })
        assertFalse(feed.state.value.loading)
        assertTrue(feed.state.value.endReached)
        assertEquals(emptyList<List<Card>>(), memory.writes)
    }

    @Test
    fun anUnchangedFirstPageKeepsTheKeptCardsAndPagingGoesOn() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("a"), testCard("b"), testCard("c")))
        val library = FakeLibrary(
            pages = mapOf(
                null to CardPage(listOf(testCard("a"), testCard("b")), next = "p2"),
                "p2" to CardPage(listOf(testCard("c"), testCard("d"))),
            ),
        )
        val feed = CardFeed(this, memory) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("a", "b", "c"), feed.state.value.cards.map { it.id })
        assertFalse(feed.state.value.endReached)
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("a", "b", "c", "d"), feed.state.value.cards.map { it.id })
        assertEquals(listOf("cards:null", "cards:p2"), library.calls)
        assertEquals(listOf(listOf("a", "b", "c", "d")), memory.writes.map { cards -> cards.map { it.id } })
    }

    @Test
    fun aChangedFirstPageReplacesTheKeptCards() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("old"), testCard("a")))
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("new"), testCard("a")))))
        val feed = CardFeed(this, memory) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("new", "a"), feed.state.value.cards.map { it.id })
        assertEquals(listOf(listOf("new", "a")), memory.writes.map { cards -> cards.map { it.id } })
    }

    @Test
    fun aChangedCardIsNotTheSamePage() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("a", title = "Before")))
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a", title = "After")))))
        val feed = CardFeed(this, memory) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("After"), feed.state.value.cards.map { it.title })
    }

    @Test
    fun offlineTheKeptCardsStayWithTheErrorUntilARetryAnswers() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("a")))
        val library = FakeLibrary(pages = emptyMap())
        val feed = CardFeed(this, memory) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("a"), feed.state.value.cards.map { it.id })
        assertNotNull(feed.state.value.error)
        assertFalse(feed.state.value.loading)
        library.pages = mapOf(null to CardPage(listOf(testCard("b"))))
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("b"), feed.state.value.cards.map { it.id })
        assertEquals(null, feed.state.value.error)
        assertEquals(listOf("cards:null", "cards:null"), library.calls)
    }

    @Test
    fun pullToRefreshWhileTheKeptCardsAreUncheckedConfirmsThem() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("a")))
        val library = FakeLibrary(pages = emptyMap())
        val feed = CardFeed(this, memory) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        library.pages = mapOf(null to CardPage(listOf(testCard("a")), next = "p2"), "p2" to CardPage(listOf(testCard("b"))))
        feed.refresh()
        advanceUntilIdle()
        assertEquals(null, feed.state.value.error)
        assertFalse(feed.state.value.endReached)
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("a", "b"), feed.state.value.cards.map { it.id })
    }

    @Test
    fun theListIsKeptAfterItChangesAndOnlyThen() = runTest(dispatcher) {
        val memory = Kept()
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("b")))))
        val feed = CardFeed(this, memory) { library.cards(it) }
        feed.loadMore()
        advanceUntilIdle()
        feed.refresh()
        advanceUntilIdle()
        feed.prepend(testCard("n"))
        advanceUntilIdle()
        feed.remove("a")
        feed.replace(testCard("b", title = "Renamed"))
        advanceUntilIdle()
        assertEquals(
            listOf(listOf("a", "b"), listOf("n", "a", "b"), listOf("n", "b")),
            memory.writes.map { cards -> cards.map { it.id } },
        )
        assertEquals("Renamed", memory.writes.last().last().title)
    }

    @Test
    fun aReloadDoesNotShowTheKeptCardsAgain() = runTest(dispatcher) {
        val memory = Kept(listOf(testCard("a")))
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a")))))
        val answer = CompletableDeferred<Unit>()
        val feed = CardFeed(this, memory) { answer.await(); library.cards(it) }
        feed.loadMore()
        runCurrent()
        feed.reload()
        runCurrent()
        assertEquals(emptyList<Card>(), feed.state.value.cards)
        answer.complete(Unit)
        advanceUntilIdle()
        assertEquals(listOf("a"), feed.state.value.cards.map { it.id })
        assertEquals(1, memory.reads)
    }

    @Test
    fun theGalleryOpensOnTheKeptSchemeAndKeepsTheServersAnswer() = runTest(dispatcher) {
        val offline = Memory(scheme = "Dark")
        val stale = LibraryViewModel(FakeLibrary(config = null), io = dispatcher, memory = offline)
        advanceUntilIdle()
        assertEquals(PageScheme.Dark, stale.scheme.value)
        assertEquals("Dark", offline.scheme)

        val online = Memory(scheme = "Dark")
        val fresh = LibraryViewModel(FakeLibrary(config = PageConfig(colorScheme = "light")), io = dispatcher, memory = online)
        advanceUntilIdle()
        assertEquals(PageScheme.Light, fresh.scheme.value)
        assertEquals("Light", online.scheme)
    }

    @Test
    fun theGalleryAndTopOfMindAreKeptApart() = runTest(dispatcher) {
        val memory = Memory()
        val library = FakeLibrary(
            pages = mapOf(null to CardPage(listOf(testCard("a")))),
            pinnedPages = mapOf(null to CardPage(listOf(testCard("p", pinPosition = 1, flags = 512)))),
        )
        LibraryViewModel(library, io = dispatcher, memory = memory)
        advanceUntilIdle()
        assertEquals(listOf("a"), memory.feeds.getValue("everything").kept.map { it.id })
        assertEquals(listOf("p"), memory.feeds.getValue("top-of-mind").kept.map { it.id })
    }
}

// A list's kept cards in memory, recording every write.
private class Kept(var kept: List<Card> = emptyList()) : FeedMemory {
    val writes = mutableListOf<List<Card>>()
    var reads = 0

    override suspend fun read(): List<Card> = kept.also { reads++ }

    override suspend fun write(cards: List<Card>) {
        writes += cards
        kept = cards
    }
}

private class Memory(var scheme: String? = null) : LibraryMemory {
    val feeds = mutableMapOf<String, Kept>()

    override fun feed(name: String): FeedMemory = feeds.getOrPut(name) { Kept() }

    override suspend fun scheme(): String? = scheme

    override suspend fun saveScheme(name: String) {
        scheme = name
    }
}

package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.ChangesPage
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.LibrarySync
import com.octaviantocan.mymind.data.LocalFirstLibrary
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.SpaceList
import com.octaviantocan.mymind.data.SyncOutcome
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class LibrarySyncTest {
    private val tokens = SyncTokens()
    private val session = Session(tokens)
    private val copy = LibraryCopy(RuntimeEnvironment.getApplication(), session, Dispatchers.Unconfined)
    private val asked = mutableListOf<String?>()
    private var feed: Map<String?, ApiResult<ChangesPage>> = emptyMap()
    private var gate: CompletableDeferred<Unit>? = null
    private val sync = LibrarySync(copy, { since ->
        asked += since
        gate?.await()
        feed[since] ?: ApiResult.Failed(ApiFailure.Network)
    }, CoroutineScope(Dispatchers.Unconfined))

    @Before
    fun setUp() = runBlocking {
        tokens.token = "session-a"
        session.restore()
    }

    private fun answer(cards: List<Card>, next: String, more: Boolean = false, removed: List<String> = emptyList()) =
        ApiResult.Ok(ChangesPage(cards, removed, next, more, spaces = if (more) null else listOf(Space("s1", "One"))))

    @Test
    fun readsEveryAnswerOnceThenOnlyWhatChanged() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("a"), testCard("b")), "t1", more = true), "t1" to answer(listOf(testCard("c")), "t2"), "t2" to answer(emptyList(), "t2"))
        assertEquals(SyncOutcome.Done(changed = true), sync.sync())
        assertEquals(listOf(null, "t1"), asked)
        assertTrue(copy.complete())
        assertEquals(SyncOutcome.Done(changed = false), sync.sync())
        assertEquals(listOf(null, "t1", "t2"), asked)
    }

    @Test
    fun aServerWithoutTheFeedIsUnavailableUntilItAnswers() = runBlocking {
        feed = mapOf(null to ApiResult.Failed(ApiFailure.Server(404)))
        assertEquals(SyncOutcome.Unavailable, sync.sync())
        assertFalse(sync.available)
        feed = emptyMap()
        assertEquals(SyncOutcome.Failed(ApiFailure.Network), sync.sync())
        feed = mapOf(null to answer(listOf(testCard("a")), "t1"))
        assertEquals(SyncOutcome.Done(changed = true), sync.sync())
        assertTrue(sync.available)
    }

    @Test
    fun anInterruptedSyncResumesWhereItStopped() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("a")), "t1", more = true))
        assertEquals(SyncOutcome.Failed(ApiFailure.Network), sync.sync())
        assertFalse(copy.complete())
        feed = mapOf("t1" to answer(listOf(testCard("b")), "t2"))
        assertEquals(SyncOutcome.Done(changed = true), sync.sync())
        assertEquals(listOf(null, "t1", "t1"), asked)
        assertEquals(listOf("b", "a"), copy.page(null)!!.cards.map { it.id })
    }

    @Test
    fun aPositionTheServerNoLongerReadsStartsTheCopyOver() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("a"), testCard("gone")), "t1"))
        sync.sync()
        feed = mapOf("t1" to ApiResult.Failed(ApiFailure.Server(400)), null to answer(listOf(testCard("a")), "n1"))
        assertEquals(SyncOutcome.Done(changed = true), sync.sync())
        assertEquals(listOf(null, "t1", null), asked)
        assertEquals("n1", copy.syncToken())
        assertEquals(listOf("a"), copy.page(null)!!.cards.map { it.id })
    }

    @Test
    fun aServerThatDropsTheFeedStopsTheGalleryReadingTheCopy() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("kept")), "t1"))
        sync.sync()
        val library = LocalFirstLibrary(FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("server"))))), copy, sync)
        assertEquals(listOf("kept"), (library.cards(null) as ApiResult.Ok).value.cards.map { it.id })
        feed = mapOf("t1" to ApiResult.Failed(ApiFailure.Server(404)))
        sync.sync()
        assertEquals(listOf("server"), (library.cards(null) as ApiResult.Ok).value.cards.map { it.id })
    }

    @Test
    fun aSecondCallJoinsTheRunningSyncUnlessItMustFollowIt() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("a")), "t1"), "t1" to answer(emptyList(), "t1"))
        gate = CompletableDeferred()
        val first = sync.start()
        assertSame(first, sync.start())
        val after = sync.start(afterCurrent = true)
        assertNotSame(first, after)
        assertEquals(listOf<String?>(null), asked)
        gate!!.complete(Unit)
        assertEquals(SyncOutcome.Done(changed = true), first.await())
        assertEquals(SyncOutcome.Done(changed = false), after.await())
        assertEquals(listOf(null, "t1"), asked)
    }

    @Test
    fun theGalleryComesFromTheServerUntilTheCopyIsWhole() = runBlocking {
        val server = FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("server"))), "s2" to CardPage()))
        val library = LocalFirstLibrary(server, copy, sync)
        assertEquals(listOf("server"), (library.cards(null) as ApiResult.Ok).value.cards.map { it.id })
        assertEquals(listOf("server"), (library.cards(null) as ApiResult.Ok).value.cards.map { it.id })
        val many = (1..55).map { testCard("c%02d".format(it)).copy(bumped = "2026-01-01T00:%02d:00Z".format(it)) }
        feed = mapOf(null to answer(many + testCard("p", flags = 512, pinPosition = 0), "t1"))
        sync.sync()
        server.calls.clear()
        val first = (library.cards(null) as ApiResult.Ok).value
        assertEquals("c55", first.cards.first().id)
        val second = (library.cards(first.next) as ApiResult.Ok).value
        assertEquals(6, second.cards.size)
        assertEquals(listOf("p"), (library.topOfMind(null) as ApiResult.Ok).value.cards.map { it.id })
        library.cards("s2")
        assertEquals("a server cursor still reads the server", listOf("cards:s2"), server.calls.filter { it.startsWith("cards") })
    }

    @Test
    fun withoutNetworkSearchCardsAndSpacesComeFromTheCopy() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("a", title = "Calm technology", spaces = listOf("s1"))), "t1"))
        sync.sync()
        var failure: ApiFailure = ApiFailure.Network
        val server = object : Library by FakeLibrary() {
            override suspend fun search(query: String, cursor: String?) = ApiResult.Failed(failure)
            override suspend fun card(cardId: String) = ApiResult.Failed(failure)
            override suspend fun spaces() = ApiResult.Failed(failure)
            override suspend fun spaceCards(spaceId: String, cursor: String?) = ApiResult.Failed(failure)
        }
        val library = LocalFirstLibrary(server, copy, sync)
        assertEquals(listOf("a"), (library.search("calm", null) as ApiResult.Ok).value.cards.map { it.id })
        assertEquals("Calm technology", (library.card("a") as ApiResult.Ok).value.title)
        assertEquals(SpaceList(listOf(Space("s1", "One"))), (library.spaces() as ApiResult.Ok).value)
        assertEquals(listOf("a"), (library.spaceCards("s1", null) as ApiResult.Ok).value.cards.map { it.id })
        assertEquals(ApiResult.Failed(ApiFailure.Network), library.card("missing"))
        failure = ApiFailure.Server(500)
        assertEquals(ApiResult.Failed(ApiFailure.Server(500)), library.search("calm", null))
    }

    @Test
    fun aWriteBringsTheCopyUpToDateBeforeItReturns() = runBlocking {
        feed = mapOf(null to answer(listOf(testCard("a")), "t1"))
        sync.sync()
        feed = mapOf("t1" to answer(listOf(testCard("a", flags = 512, pinPosition = 0)), "t2"))
        val server = FakeLibrary()
        val library = LocalFirstLibrary(server, copy, sync)
        assertEquals(ApiResult.Ok(Unit), library.setPinned("a", true))
        assertEquals(listOf("pin:a:true"), server.calls)
        assertEquals(listOf("a"), copy.pinned()!!.cards.map { it.id })
        server.writeFailure = ApiFailure.Network
        asked.clear()
        library.trash("a")
        assertEquals(emptyList<String?>(), asked)
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun theGalleryShowsWhatASyncBrought() = runBlocking {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        try {
            feed = mapOf(null to answer(listOf(testCard("a")), "t1"), "t1" to answer(emptyList(), "t1"))
            sync.sync()
            val viewModel = LibraryViewModel(LocalFirstLibrary(FakeLibrary(), copy, sync), sync = sync)
            assertEquals(listOf("a"), viewModel.everything.state.value.cards.map { it.id })
            feed = mapOf("t1" to answer(listOf(testCard("b").copy(bumped = "2026-02-01T00:00:00Z"), testCard("p", flags = 512)), "t2"))
            viewModel.syncNow()
            assertEquals(listOf("b", "p", "a"), viewModel.everything.state.value.cards.map { it.id })
            assertEquals(listOf("p"), viewModel.topOfMind.state.value.cards.map { it.id })
        } finally {
            Dispatchers.resetMain()
        }
    }
}

private class SyncTokens : TokenStore {
    var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

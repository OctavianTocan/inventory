package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.ChangesPage
import com.octaviantocan.mymind.data.Edit
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.LibrarySync
import com.octaviantocan.mymind.data.LocalFirstLibrary
import com.octaviantocan.mymind.data.Outbox
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.SyncOutcome
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.data.refusedNotice
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class OutboxTest {
    private val tokens = OutboxTokens()
    private val session = Session(tokens)
    private val copy = LibraryCopy(RuntimeEnvironment.getApplication(), session, Dispatchers.Unconfined)
    private val server = FakeLibrary()
    private var queued = 0
    private val refused = mutableListOf<Pair<Edit, Card?>>()
    private val outbox = Outbox(copy, server, onQueued = { queued++ }, onRefused = { edit, card -> refused += edit to card })
    private var feed: Map<String?, ApiResult<ChangesPage>> = emptyMap()
    private val sync = LibrarySync(copy, { since -> feed[since] ?: ApiResult.Failed(ApiFailure.Network) }, outbox::send, scope = CoroutineScope(Dispatchers.Unconfined))
    private val library = LocalFirstLibrary(server, copy, sync, outbox)

    @Before
    fun setUp() = runBlocking {
        tokens.token = "session-a"
        session.restore()
        feed = mapOf(null to ApiResult.Ok(ChangesPage(listOf(testCard("a"), testCard("b", tags = listOf(Tag("t1", "old")), spaces = listOf("s1"))), next = "t1")))
        sync.sync()
        feed = mapOf("t1" to ApiResult.Ok(ChangesPage(next = "t1")))
    }

    @Test
    fun offlineChangesShowAtOnceAndGoOutInOrderOnceOnline() = runBlocking {
        server.writeFailure = ApiFailure.Network
        assertEquals(ApiResult.Ok(Unit), library.setPinned("a", true))
        assertEquals(ApiResult.Ok(Unit), library.trash("b"))
        assertEquals(ApiResult.Ok(Unit), library.addToSpace("s2", "a"))
        assertEquals(ApiResult.Ok(Unit), library.setNote("a", "later"))
        assertEquals(listOf("a"), copy.pinned()!!.cards.map { it.id })
        assertEquals(listOf("a"), copy.page(null)!!.cards.map { it.id })
        assertEquals("later", copy.card("a")!!.note)
        assertEquals(listOf("s2"), copy.card("a")!!.spaces)
        assertEquals(4, queued)
        server.writeFailure = null
        server.calls.clear()
        assertEquals(SyncOutcome.Done(changed = false), sync.sync())
        assertEquals(listOf("pin:a:true", "trash:b", "space:s2:a", "note:a:later"), server.calls)
        assertTrue(outbox.isEmpty())
    }

    @Test
    fun anOnlineChangeShowsAtOnceAndReachesTheServerInTheBackground() = runBlocking {
        val answer = CompletableDeferred<Unit>()
        val slow = object : Library by server {
            override suspend fun setPinned(cardId: String, pinned: Boolean): ApiResult<Unit> {
                answer.await()
                return server.setPinned(cardId, pinned)
            }
        }
        val slowOutbox = Outbox(copy, slow)
        val slowSync = LibrarySync(copy, { since -> feed[since] ?: ApiResult.Failed(ApiFailure.Network) }, slowOutbox::send, scope = CoroutineScope(Dispatchers.Unconfined))
        val library = LocalFirstLibrary(slow, copy, slowSync, slowOutbox)
        assertEquals(ApiResult.Ok(Unit), library.setPinned("a", true))
        assertEquals(listOf("a"), copy.pinned()!!.cards.map { it.id })
        assertEquals("the server has not answered yet", emptyList<String>(), server.calls)
        answer.complete(Unit)
        slowSync.sync()
        assertEquals(listOf("pin:a:true"), server.calls)
        assertTrue(slowOutbox.isEmpty())
    }

    @Test
    fun aCardAddedToASpaceGoesToItsEnd() = runBlocking {
        copy.apply(ChangesPage(listOf(testCard("b", spaces = listOf("s1")).copy(spaceRanks = mapOf("s1" to 4.0))), next = "t1"))
        server.writeFailure = ApiFailure.Network
        assertEquals(ApiResult.Ok(Unit), library.addToSpace("s1", "a"))
        assertEquals(mapOf("s1" to 5.0), copy.card("a")!!.spaceRanks)
        assertEquals(listOf("b", "a"), copy.spaceCards("s1")!!.cards.map { it.id })
        assertEquals(ApiResult.Ok(Unit), library.removeFromSpace("s1", "a"))
        assertNull(copy.card("a")!!.spaceRanks?.get("s1"))
    }

    @Test
    fun withoutTheFeedAnOnlineChangeGoesStraightToTheServer() = runBlocking {
        feed = mapOf("t1" to ApiResult.Failed(ApiFailure.Server(404)))
        assertEquals(SyncOutcome.Unavailable, sync.sync())
        server.writeFailure = ApiFailure.NotAllowed
        assertEquals(ApiResult.Failed(ApiFailure.NotAllowed), library.setPinned("a", true))
        assertTrue(outbox.isEmpty())
        assertEquals(emptyList<String>(), copy.pinned()!!.cards.map { it.id })
    }

    @Test
    fun aChangeMadeOnlineWaitsBehindTheOnesQueuedBeforeIt() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", true)
        server.writeFailure = null
        server.calls.clear()
        assertEquals(ApiResult.Ok(Unit), library.setPinned("a", false))
        assertEquals(listOf("pin:a:true", "pin:a:false"), server.calls)
        assertTrue(outbox.isEmpty())
    }

    @Test
    fun aTagAddedOfflineCanBeTakenBackBeforeItIsSent() = runBlocking {
        server.writeFailure = ApiFailure.Network
        val tag = (library.addTag("a", "later") as ApiResult.Ok).value
        assertEquals(listOf("later"), copy.card("a")!!.tags!!.map { it.name })
        assertEquals(ApiResult.Ok(Unit), library.removeTag(tag.id!!))
        assertEquals(emptyList<String>(), copy.card("a")!!.tags!!.map { it.name })
        assertTrue(outbox.isEmpty())
    }

    @Test
    fun aTagSentFromTheOutboxIsRemovedByTheServersId() = runBlocking {
        server.writeFailure = ApiFailure.Network
        val tag = (library.addTag("a", "later") as ApiResult.Ok).value
        server.writeFailure = null
        sync.sync()
        server.calls.clear()
        assertEquals(ApiResult.Ok(Unit), library.removeTag(tag.id!!))
        assertEquals(listOf("untag:t-later"), server.calls)
    }

    @Test
    fun aTagRemovedOfflineLeavesItsCardAtOnce() = runBlocking {
        server.writeFailure = ApiFailure.Network
        assertEquals(ApiResult.Ok(Unit), library.removeTag("t1"))
        assertEquals(emptyList<Tag>(), copy.card("b")!!.tags)
        assertEquals(Edit.RemoveTag("b", "t1"), copy.firstQueued()!!.second)
    }

    @Test
    fun aRefusedChangeIsDroppedAndItsCardReadAgain() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", true)
        server.writeFailure = ApiFailure.NotAllowed
        server.cardsById = mapOf("a" to testCard("a", title = "As the server has it"))
        assertNull(outbox.send())
        assertEquals(listOf(Edit.Pin("a", true)), refused.map { it.first })
        assertEquals("Your change to “Card a” could not be saved.", refusedNotice(refused.single().second))
        assertEquals(emptyList<String>(), copy.pinned()!!.cards.map { it.id })
        assertEquals("As the server has it", copy.card("a")!!.title)
        assertTrue(outbox.isEmpty())
    }

    @Test
    fun aRefusedChangeDropsItsCardOnlyWhenTheServerNoLongerHasIt() = runBlocking {
        val unreadable = Outbox(copy, object : Library by server {
            override suspend fun card(cardId: String): ApiResult<Card> = ApiResult.Failed(ApiFailure.Network)
        })
        server.writeFailure = ApiFailure.Network
        library.setNote("a", "later")
        server.writeFailure = ApiFailure.Invalid
        assertNull(unreadable.send())
        assertEquals("Card a", copy.card("a")!!.title)
        server.writeFailure = ApiFailure.Network
        library.setNote("b", "later")
        server.writeFailure = ApiFailure.Invalid
        assertNull(outbox.send())
        assertNull(copy.card("b"))
    }

    @Test
    fun waitingChangesStayOnCardsTheServerSendsAgain() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", true)
        copy.apply(ChangesPage(listOf(testCard("a", title = "Renamed")), next = "t2"))
        assertEquals("Renamed", copy.card("a")!!.title)
        assertEquals(listOf("a"), copy.pinned()!!.cards.map { it.id })
    }

    @Test
    fun anotherSessionNeverSendsTheChanges() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.trash("a")
        assertFalse(outbox.isEmpty())
        tokens.token = "session-b"
        session.restore()
        assertTrue(outbox.isEmpty())
    }
}

private class OutboxTokens : TokenStore {
    var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

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
import com.octaviantocan.mymind.data.PushAnswer
import com.octaviantocan.mymind.data.PushRequest
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.SyncOutcome
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.data.refusedNotice
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
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
    private val pushed = mutableListOf<PushRequest>()
    private var pushAnswer: (PushRequest) -> ApiResult<PushAnswer> = { request -> ApiResult.Ok(PushAnswer(request.changes.last().id)) }
    private val pushing = Outbox(copy, server, onRefused = { edit, card -> refused += edit to card }, push = { request -> pushed += request; pushAnswer(request) })

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
    fun waitingChangesGoOutInOneBatchUnderTheirPlaceInTheQueue() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", true)
        library.setNote("b", "later")
        library.removeFromSpace("s1", "b")
        server.writeFailure = null
        server.calls.clear()
        assertNull(pushing.send())
        val changes = pushed.single().changes
        assertEquals(listOf("pin", "note", "remove-from-space"), changes.map { it.name })
        assertEquals(changes.map { it.id }.sorted(), changes.map { it.id })
        assertEquals(JsonObject(mapOf("cardId" to JsonPrimitive("a"), "pinned" to JsonPrimitive(true))), changes[0].args)
        assertEquals("no change went out on its own", emptyList<String>(), server.calls)
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun aBatchSentAgainCarriesTheSameIdsFromTheSamePhone() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.trash("a")
        pushAnswer = { ApiResult.Failed(ApiFailure.Network) }
        assertEquals(ApiFailure.Network, pushing.send())
        assertFalse(pushing.isEmpty())
        pushAnswer = { request -> ApiResult.Ok(PushAnswer(request.changes.last().id)) }
        assertNull(pushing.send())
        assertEquals(pushed[0], pushed[1])
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun aChangeThePushTurnsDownIsDroppedAndItsCardReadAgain() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", true)
        library.setNote("b", "later")
        server.cardsById = mapOf("a" to testCard("a", title = "As the server has it"))
        pushAnswer = { request -> ApiResult.Ok(PushAnswer(request.changes.last().id, refused = listOf(request.changes.first().id))) }
        assertNull(pushing.send())
        assertEquals(listOf(Edit.Pin("a", true)), refused.map { it.first })
        assertEquals(emptyList<String>(), copy.pinned()!!.cards.map { it.id })
        assertEquals("As the server has it", copy.card("a")!!.title)
        assertEquals("later", copy.card("b")!!.note)
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun aServerWithoutThePushGetsEachChangeOnItsOwn() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", true)
        library.trash("b")
        server.writeFailure = null
        server.calls.clear()
        pushAnswer = { ApiResult.Ok(PushAnswer()) }
        assertNull(pushing.send())
        assertEquals(listOf("pin:a:true", "trash:b"), server.calls)
        server.writeFailure = ApiFailure.Network
        library.setPinned("a", false)
        server.writeFailure = null
        assertNull(pushing.send())
        assertEquals("the push is not asked again", 1, pushed.size)
        assertEquals("pin:a:false", server.calls.last())
    }

    @Test
    fun aPushAnswerTheAppCannotReadSendsTheChangesOneByOne() = runBlocking {
        server.writeFailure = ApiFailure.Network
        library.setNote("a", "later")
        server.writeFailure = null
        server.calls.clear()
        pushAnswer = { ApiResult.Failed(ApiFailure.Server(0)) }
        assertNull(pushing.send())
        assertEquals(listOf("note:a:later"), server.calls)
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun aTagAddedThroughThePushIsRemovedByTheServersId() = runBlocking {
        val pushingSync = LibrarySync(copy, { since -> feed[since] ?: ApiResult.Failed(ApiFailure.Network) }, pushing::send, scope = CoroutineScope(Dispatchers.Unconfined))
        val library = LocalFirstLibrary(server, copy, pushingSync, pushing)
        pushAnswer = { request -> ApiResult.Ok(PushAnswer(request.changes.last().id, tagIds = request.changes.filter { it.name == "add-tag" }.associate { it.id.toString() to "t-server" })) }
        val tag = (library.addTag("a", "later") as ApiResult.Ok).value
        assertEquals("add-tag", pushed.single().changes.single().name)
        assertEquals(ApiResult.Ok(Unit), library.removeTag(tag.id!!))
        assertEquals(JsonObject(mapOf("tagId" to JsonPrimitive("t-server"))), pushed.last().changes.single().args)
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun onceThePushWorksASpaceMadeOnThePhoneShowsAtOnceUnderItsOwnId() = runBlocking {
        copy.apply(ChangesPage(next = "t1", spaces = listOf(Space("s1", "One", position = 4))))
        val pushingSync = LibrarySync(copy, { since -> feed[since] ?: ApiResult.Failed(ApiFailure.Network) }, pushing::send, scope = CoroutineScope(Dispatchers.Unconfined))
        val library = LocalFirstLibrary(server, copy, pushingSync, pushing)
        assertEquals(ApiResult.Ok("s-new"), library.createSpace("First", "#f96"))
        assertEquals("before a push has worked, the server makes the space", listOf("newSpace:First:#f96"), server.calls)
        assertTrue(library.setPinned("a", true) is ApiResult.Ok)
        server.calls.clear()
        val id = (library.createSpace("Reading", "#75ffc0") as ApiResult.Ok).value
        assertTrue(id.matches(Regex("[0-9a-f]{32}")))
        assertEquals(Space(id, "Reading", "#75ffc0", 5, smart = false, count = 0), copy.spaces()!!.last())
        val made = pushed.last().changes.single()
        assertEquals("create-space", made.name)
        assertEquals(JsonObject(mapOf("spaceId" to JsonPrimitive(id), "name" to JsonPrimitive("Reading"), "color" to JsonPrimitive("#75ffc0"))), made.args)
        assertEquals(emptyList<String>(), server.calls)
    }

    @Test
    fun aDeletedSpaceLeavesTheListAndItsCardsAtOnceAndStaysGone() = runBlocking {
        copy.apply(ChangesPage(next = "t1", spaces = listOf(Space("s1", "One"), Space("s2", "Two"))))
        server.writeFailure = ApiFailure.Network
        assertEquals(ApiResult.Ok(Unit), library.deleteSpace("s1"))
        assertEquals(listOf("s2"), copy.spaces()!!.map { it.id })
        assertEquals(emptyList<String>(), copy.card("b")!!.spaces)
        copy.apply(ChangesPage(listOf(testCard("b", spaces = listOf("s1", "s2"))), next = "t2", spaces = listOf(Space("s1", "One"), Space("s2", "Two"))))
        assertEquals("the server's answer from before the delete does not bring it back", listOf("s2"), copy.spaces()!!.map { it.id })
        assertEquals(listOf("s2"), copy.card("b")!!.spaces)
        server.writeFailure = null
        assertNull(outbox.send())
        assertEquals("deleteSpace:s1", server.calls.last())
        assertTrue(outbox.isEmpty())
    }

    @Test
    fun aSpaceTheServerCannotTakeIsDroppedWithANotice() = runBlocking {
        copy.apply(ChangesPage(next = "t1", spaces = listOf(Space("s1", "One"))))
        copy.queue(Edit.CreateSpace("0a1b2c3d4e5f4a6b8c7d9e0f1a2b3c4d", "Reading", "#75ffc0"))
        assertEquals(listOf("s1", "0a1b2c3d4e5f4a6b8c7d9e0f1a2b3c4d"), copy.spaces()!!.map { it.id })
        assertNull(outbox.send())
        assertEquals("Your new space “Reading” could not be saved.", refusedNotice(refused.single().first, refused.single().second))
        assertTrue(outbox.isEmpty())
    }

    // The library over the pushing outbox, once the server has answered a push and so keeps the ids the phone makes.
    private suspend fun pushedLibrary(): LocalFirstLibrary {
        val pushingSync = LibrarySync(copy, { since -> feed[since] ?: ApiResult.Failed(ApiFailure.Network) }, pushing::send, scope = CoroutineScope(Dispatchers.Unconfined))
        val library = LocalFirstLibrary(server, copy, pushingSync, pushing)
        assertTrue(library.setPinned("a", true) is ApiResult.Ok)
        assertTrue(pushing.pushWorks())
        return library
    }

    @Test
    fun onceThePushWorksASharedLinkShowsAtOnceUnderThePhonesId() = runBlocking {
        val library = pushedLibrary()
        pushAnswer = { ApiResult.Failed(ApiFailure.Network) }
        val id = (library.createFromContent("https://example.com/a", "clipboard", null, "Café Lumen") as ApiResult.Ok).value
        assertTrue(id.matches(Regex("[0-9a-f]{32}")))
        val shown = copy.card(id)!!
        assertEquals(listOf("Placeholder", "https://example.com/a", "example.com", "Café Lumen"), listOf(shown.type, shown.url, shown.domain, shown.note))
        assertEquals("it leads the gallery", id, copy.page(null)!!.cards.first().id)
        assertEquals("its tile shows the address until the server reads it", "https://example.com/a", cardHeading(shown))
        val made = pushed.last().changes.single()
        assertEquals("create-card", made.name)
        assertEquals(listOf(id, "https://example.com/a", "clipboard", "Café Lumen"), listOf("cardId", "content", "source", "note").map { (made.args[it] as JsonPrimitive).content })
        pushAnswer = { request -> ApiResult.Ok(PushAnswer(request.changes.last().id, cardIds = request.changes.associate { it.id.toString() to id })) }
        assertNull(pushing.send())
        assertEquals("the server keeps the phone's id", "Placeholder", copy.card(id)?.type)
        assertEquals(emptyList<String>(), server.calls.filter { it.startsWith("content:") })
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun aSharedLinkAlreadySavedMakesWayForTheSavedCard() = runBlocking {
        val library = pushedLibrary()
        pushAnswer = { ApiResult.Failed(ApiFailure.Network) }
        val id = (library.createFromContent("https://example.com/a", "clipboard", null, null) as ApiResult.Ok).value
        assertTrue(library.addTag(id, "later") is ApiResult.Ok)
        assertEquals(listOf("later"), copy.card(id)!!.tags!!.map { it.name })
        pushAnswer = { request -> ApiResult.Ok(PushAnswer(request.changes.last().id, cardIds = mapOf(request.changes.first().id.toString() to "a"))) }
        assertNull(pushing.send())
        assertEquals(listOf("create-card", "add-tag"), pushed.last().changes.map { it.name })
        assertNull("the phone's card makes way for the saved one", copy.card(id))
        assertTrue(refused.isEmpty())
        assertTrue(pushing.isEmpty())
    }

    @Test
    fun sharedTextShowsAsANoteThatSearchFindsOffline() = runBlocking {
        val library = pushedLibrary()
        pushAnswer = { ApiResult.Failed(ApiFailure.Network) }
        val id = (library.createFromContent("Buy oat milk\nand bread", "clipboard", null, null) as ApiResult.Ok).value
        assertEquals("Note", copy.card(id)!!.type)
        assertEquals(listOf(id), copy.search("bread")!!.cards.map { it.id })
        assertEquals("Something you shared could not be saved.", refusedNotice(Edit.CreateCard(id, "x", "clipboard", made = ""), null))
    }

    @Test
    fun beforeThePushWorksASharedLinkGoesToTheServer() = runBlocking {
        val pushingSync = LibrarySync(copy, { since -> feed[since] ?: ApiResult.Failed(ApiFailure.Network) }, pushing::send, scope = CoroutineScope(Dispatchers.Unconfined))
        val library = LocalFirstLibrary(server, copy, pushingSync, pushing)
        assertEquals(ApiResult.Ok("card-${"https://example.com/a".hashCode()}"), library.createFromContent("https://example.com/a", "clipboard", "s1", null))
        assertEquals(listOf("content:https://example.com/a:clipboard:s1"), server.calls)
        assertTrue(pushed.isEmpty())
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
        assertEquals("Your change to “Card a” could not be saved.", refusedNotice(refused.single().first, refused.single().second))
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

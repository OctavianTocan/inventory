package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.AppJson
import com.octaviantocan.mymind.data.ChangesPage
import com.octaviantocan.mymind.data.Edit
import com.octaviantocan.mymind.data.FEED_MEMORY_CARDS
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import java.time.Instant

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class LibraryCopyTest {
    private val tokens = CopyTokens()
    private val session = Session(tokens)
    private val copy = LibraryCopy(RuntimeEnvironment.getApplication(), session, Dispatchers.Unconfined)

    private fun signIn(token: String?) = runBlocking {
        tokens.token = token
        session.restore()
    }

    @Test
    fun keepsTheFirstCardsOfEachListInOrder() = runBlocking {
        signIn("session-a")
        val cards = (1..FEED_MEMORY_CARDS + 50).map { testCard("c$it", note = "note $it", tags = listOf(Tag("t$it", "tag $it"))) }
        copy.feed("everything").write(cards)
        copy.feed("top-of-mind").write(listOf(testCard("p", pinPosition = 1, flags = 512)))
        assertEquals(cards.take(FEED_MEMORY_CARDS), copy.feed("everything").read())
        assertEquals(listOf(testCard("p", pinPosition = 1, flags = 512)), copy.feed("top-of-mind").read())
        copy.feed("everything").write(listOf(testCard("only")))
        assertEquals(listOf("only"), copy.feed("everything").read().map { it.id })
    }

    @Test
    fun theCopySaysWhenWhatItShowsChanged() = runBlocking {
        signIn("session-a")
        var heard = 0
        val listening = launch(Dispatchers.Unconfined) { copy.changes.collect { heard++ } }
        copy.apply(ChangesPage(listOf(testCard("a")), next = "t1", spaces = listOf(Space("s1", "One"))))
        assertEquals("cards, spaces and a copy now whole", 1, heard)
        copy.apply(ChangesPage(next = "t1", spaces = listOf(Space("s1", "One"))))
        assertEquals("an answer that changes nothing is quiet", 1, heard)
        copy.queue(Edit.Pin("a", true))
        assertEquals(2, heard)
        copy.replace("a", testCard("a", title = "Read again"))
        assertEquals(3, heard)
        listening.cancel()
    }

    @Test
    fun anotherSessionNeverSeesTheCopy() = runBlocking {
        signIn("session-a")
        copy.feed("everything").write(listOf(testCard("a")))
        copy.saveScheme("Dark")
        signIn("session-b")
        assertEquals(emptyList<Any>(), copy.feed("everything").read())
        assertNull(copy.scheme())
        signIn("session-a")
        assertEquals(emptyList<Any>(), copy.feed("everything").read())
    }

    @Test
    fun signedOutReadsNothingAndKeepsNothing() = runBlocking {
        signIn(null)
        copy.feed("everything").write(listOf(testCard("a")))
        copy.saveScheme("Dark")
        assertEquals(emptyList<Any>(), copy.feed("everything").read())
        signIn("session-a")
        assertEquals(emptyList<Any>(), copy.feed("everything").read())
        assertNull(copy.scheme())
    }

    @Test
    fun clearForgetsEverything() = runBlocking {
        signIn("session-a")
        copy.feed("everything").write(listOf(testCard("a")))
        copy.saveScheme("Dark")
        assertEquals("Dark", copy.scheme())
        copy.clear()
        assertEquals(emptyList<Any>(), copy.feed("everything").read())
        assertNull(copy.scheme())
    }

    @Test
    fun aSyncedCopyAnswersTheGalleryInTheServersOrder() = runBlocking {
        signIn("session-a")
        val cards = (1..60).map { testCard("c%02d".format(it)).copy(bumped = Instant.parse("2026-01-01T00:00:00Z").plusSeconds(it * 60L).toString()) } +
            listOf(testCard("tie-a"), testCard("tie-b"), testCard("private", flags = 8).copy(bumped = "2026-02-01T00:00:00Z"))
        assertTrue(copy.apply(ChangesPage(cards.take(30), next = "t1", more = true)))
        assertFalse(copy.complete())
        assertTrue(copy.apply(ChangesPage(cards.drop(30), next = "t2", spaces = listOf(Space("s1", "One")))))
        assertTrue(copy.complete())
        assertEquals("t2", copy.syncToken())
        val first = copy.page(null)!!
        assertEquals((60 downTo 11).map { "c%02d".format(it) }, first.cards.map { it.id })
        val second = copy.page(first.next)!!
        assertEquals((10 downTo 1).map { "c%02d".format(it) } + listOf("tie-b", "tie-a"), second.cards.map { it.id })
        assertNull(second.next)
        assertEquals(listOf(Space("s1", "One")), copy.spaces())
    }

    @Test
    fun changesReplaceAndRemoveCardsAndAResetStartsOver() = runBlocking {
        signIn("session-a")
        copy.apply(ChangesPage(listOf(testCard("a"), testCard("b")), next = "t1"))
        copy.apply(ChangesPage(listOf(testCard("a", title = "Renamed")), removed = listOf("b", "never-seen"), next = "t2"))
        assertEquals(listOf("Renamed"), copy.page(null)!!.cards.map { it.title })
        assertNull(copy.card("b"))
        copy.apply(ChangesPage(listOf(testCard("z")), next = "new-1", more = true, reset = true))
        assertFalse(copy.complete())
        assertEquals(listOf("z"), copy.page(null)!!.cards.map { it.id })
    }

    @Test
    fun pinnedKeepsPinOrderThenNewestFirst() = runBlocking {
        signIn("session-a")
        copy.apply(ChangesPage(listOf(
            testCard("second", flags = 512, pinPosition = 1),
            testCard("first", flags = 512, pinPosition = 0),
            testCard("unplaced", flags = 512).copy(bumped = "2026-03-01T00:00:00Z"),
            testCard("older-unplaced", flags = 512),
            testCard("hidden", flags = 512 or 131_072, pinPosition = 2),
            testCard("plain"),
        ), next = "t1"))
        assertEquals(listOf("first", "second", "unplaced", "older-unplaced"), copy.pinned()!!.cards.map { it.id })
    }

    @Test
    fun keywordSearchFindsWordsAnywhereInTheCard() = runBlocking {
        signIn("session-a")
        val prose = AppJson.parseToJsonElement("""{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Peripheral interfaces"}]}]}""")
        copy.apply(ChangesPage(listOf(
            testCard("titled", title = "Calm technology"),
            testCard("noted", note = "Read about café culture"),
            testCard("tagged", tags = listOf(Tag("t1", "design"))),
            testCard("saved").copy(prose = prose),
            testCard("secret", title = "Calm and private", flags = 8),
        ), next = "t1"))
        assertEquals(listOf("titled"), copy.search("CALM tech")!!.cards.map { it.id })
        assertEquals(listOf("noted"), copy.search("café cult")!!.cards.map { it.id })
        assertEquals(listOf("tagged"), copy.search("desig")!!.cards.map { it.id })
        assertEquals(listOf("saved"), copy.search("periph")!!.cards.map { it.id })
        assertEquals(emptyList<String>(), copy.search("\"*:")!!.cards.map { it.id })
    }

    @Test
    fun spaceCardsAreTheCardsThatListTheSpace() = runBlocking {
        signIn("session-a")
        copy.apply(ChangesPage(listOf(testCard("in", spaces = listOf("s1", "s2")), testCard("near", spaces = listOf("s10")), testCard("out")), next = "t1"))
        assertEquals(listOf("in"), copy.spaceCards("s1")!!.cards.map { it.id })
        assertEquals(listOf("in"), copy.spaceCards("s2")!!.cards.map { it.id })
    }

    @Test
    fun theSyncedCopyBelongsToItsSession() = runBlocking {
        signIn("session-a")
        copy.apply(ChangesPage(listOf(testCard("a")), next = "t1"))
        signIn("session-b")
        assertFalse(copy.complete())
        assertNull(copy.syncToken())
        assertEquals(emptyList<String>(), copy.page(null)!!.cards.map { it.id })
        signIn(null)
        assertNull(copy.page(null))
        assertFalse(copy.apply(ChangesPage(listOf(testCard("a")), next = "t1")))
    }
}

private class CopyTokens : TokenStore {
    var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

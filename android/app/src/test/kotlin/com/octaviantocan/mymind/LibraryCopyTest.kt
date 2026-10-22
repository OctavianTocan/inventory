package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.FEED_MEMORY_CARDS
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config

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
}

private class CopyTokens : TokenStore {
    var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

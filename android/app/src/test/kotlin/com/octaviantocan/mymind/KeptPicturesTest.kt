package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.ChangesPage
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.KEPT_PICTURE_CARDS
import com.octaviantocan.mymind.data.KeptPictures
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import java.time.Instant

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class KeptPicturesTest {
    private val tokens = PictureTokens()
    private val session = Session(tokens)
    private val copy = LibraryCopy(RuntimeEnvironment.getApplication(), session, Dispatchers.Unconfined)
    private val fetched = mutableListOf<String>()
    private var onPhone = emptySet<String>()
    private val pictures = KeptPictures(copy, { it.cover?.src }, { it in onPhone }, { fetched += it; true })

    @Before
    fun setUp() = runBlocking {
        tokens.token = "session-a"
        session.restore()
    }

    private fun pictured(id: String, minute: Long, src: String = "/p/$id", flags: Int = 0, pin: Long? = null) =
        testCard(id, flags = flags, pinPosition = pin).copy(
            bumped = Instant.parse("2026-01-01T00:00:00Z").plusSeconds(minute * 60).toString(),
            cover = Cover(src = src),
        )

    @Test
    fun fetchesPinnedThenTheNewestCardsPicturesItDoesNotHave() = runBlocking {
        val newest = (1..KEPT_PICTURE_CARDS + 5L).map { pictured("c%04d".format(it), it) }
        val others = listOf(
            pictured("pinned-old", 0, flags = 512, pin = 0),
            pictured("private", 5_000, flags = 8),
            pictured("same-picture", 4_000, src = "/p/c1004"),
            testCard("no-picture").copy(bumped = "2027-01-01T00:00:00Z"),
        )
        copy.apply(ChangesPage(newest + others, next = "t1"))
        onPhone = setOf("/p/c1005")
        assertEquals(998, pictures.fetchMissing())
        assertEquals("/p/pinned-old", fetched.first())
        assertEquals(listOf("/p/c1004", "/p/c1003"), fetched.drop(1).take(2))
        assertEquals("/p/c0008", fetched.last())
        assertEquals(fetched.distinct(), fetched)
    }

    @Test
    fun waitsUntilThePhoneHoldsTheWholeLibrary() = runBlocking {
        copy.apply(ChangesPage(listOf(pictured("a", 1)), next = "t1", more = true))
        assertNull(pictures.fetchMissing())
        assertEquals(emptyList<String>(), fetched)
    }

    @Test
    fun stopsWhenAsked() = runBlocking {
        copy.apply(ChangesPage((1..10L).map { pictured("c$it", it) }, next = "t1"))
        assertEquals(3, pictures.fetchMissing { fetched.size == 3 })
    }
}

private class PictureTokens : TokenStore {
    var token: String? = null
    override suspend fun read() = token
    override suspend fun write(token: String) { this.token = token }
    override suspend fun clear() { token = null }
}

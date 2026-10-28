package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.AiRanking
import com.octaviantocan.mymind.data.AiSearchAnswer
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AiSearchTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private class Ranker(var engine: String? = "Claude Code", var answer: () -> AiSearchAnswer = { AiSearchAnswer() }) : AiRanking {
        val asked = mutableListOf<String>()
        var gate: CompletableDeferred<Unit>? = null
        override suspend fun engine() = engine
        override suspend fun search(query: String): AiSearchAnswer {
            asked += query
            gate?.await()
            return answer()
        }
    }

    @Test
    fun pressingSearchShowsTheMatchesAtOnceThenTheRankedOrder() = runTest(dispatcher) {
        val library = FakeLibrary()
        val ranker = Ranker(answer = { AiSearchAnswer(listOf(testCard("lake"), testCard("hit")), ranked = 1) }).apply { gate = CompletableDeferred() }
        val search = SearchModel(this, library, ranker)
        search.setQuery("calm places")
        search.submit()
        advanceUntilIdle()
        // The library's own search answers while the engine is still ranking.
        assertEquals(listOf("hit"), search.results.value.state.value.cards.map { it.id })
        assertEquals(AiRank.Ranking("Claude Code"), search.rank.value)
        ranker.gate!!.complete(Unit)
        advanceUntilIdle()
        assertEquals(listOf("calm places"), ranker.asked)
        assertEquals(listOf("lake", "hit"), search.results.value.state.value.cards.map { it.id })
        assertEquals(AiRank.Ranked("Claude Code", 1), search.rank.value)
        assertEquals("Ranked by Claude Code", aiNote(search.rank.value!!))
    }

    @Test
    fun typingAgainDropsTheRanking() = runTest(dispatcher) {
        val ranker = Ranker().apply { gate = CompletableDeferred() }
        val search = SearchModel(this, FakeLibrary(), ranker)
        search.setQuery("calm")
        search.submit()
        advanceUntilIdle()
        search.setQuery("calm lake")
        advanceUntilIdle()
        ranker.gate!!.complete(Unit)
        advanceUntilIdle()
        assertNull(search.rank.value)
        assertEquals(listOf("hit"), search.results.value.state.value.cards.map { it.id })
    }

    @Test
    fun withoutASignedInEngineOrForSameVibeNothingIsRanked() = runTest(dispatcher) {
        val ranker = Ranker(engine = null)
        val search = SearchModel(this, FakeLibrary(), ranker)
        search.setQuery("calm")
        search.submit()
        advanceUntilIdle()
        assertNull(search.rank.value)
        assertEquals(emptyList<String>(), ranker.asked)

        ranker.engine = "Codex"
        search.similarTo(testCard("lake"))
        search.submit()
        advanceUntilIdle()
        assertNull(search.rank.value)
        assertEquals(emptyList<String>(), ranker.asked)
    }

    @Test
    fun aFailedRankingKeepsTheMatchesAndSaysWhy() = runTest(dispatcher) {
        val ranker = Ranker(engine = "Codex", answer = { throw IllegalStateException("Sign in to Codex first: Profile, AI, Codex.") })
        val search = SearchModel(this, FakeLibrary(), ranker)
        search.setQuery("calm")
        search.submit()
        advanceUntilIdle()
        assertEquals(AiRank.Failed("Sign in to Codex first: Profile, AI, Codex."), search.rank.value)
        assertEquals(listOf("hit"), search.results.value.state.value.cards.map { it.id })
    }
}

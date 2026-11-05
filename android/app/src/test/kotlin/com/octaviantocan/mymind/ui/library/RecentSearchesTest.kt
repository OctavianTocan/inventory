package com.octaviantocan.mymind.ui.library

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.AiRanking
import com.octaviantocan.mymind.data.AiSearchAnswer
import com.octaviantocan.mymind.data.AiUnderstood
import com.octaviantocan.mymind.data.RecentSearches
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
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/** Version 10 of the search page (Tavi, 5 November 2026, "Let's build 10"): recent searches, the engine and the ranked-only view. */
@OptIn(ExperimentalCoroutinesApi::class)
@RunWith(AndroidJUnit4::class)
class RecentSearchesTest {
    private val dispatcher = StandardTestDispatcher()
    private val prefs = ApplicationProvider.getApplicationContext<Context>().getSharedPreferences("recent-test", Context.MODE_PRIVATE)

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
        prefs.edit().clear().commit()
    }

    @After
    fun tearDown() = Dispatchers.resetMain()

    private class Ranker(private val name: String?) : AiRanking {
        override suspend fun engine() = name
        override suspend fun understand(query: String) = AiUnderstood()
        override suspend fun search(query: String, understood: AiUnderstood?) =
            AiSearchAnswer(listOf(testCard("lake"), testCard("pond"), testCard("hit")), ranked = 2)
    }

    @Test
    fun keepsTheLastFiveNewestFirstWithoutRepeatsAndOutlivesTheApp() {
        val recent = RecentSearches(prefs)
        listOf("one", "two", "three", "four", "five", "six").forEach(recent::add)
        recent.add("  Three  ")
        recent.add("   ")
        assertEquals(listOf("Three", "six", "five", "four", "two"), recent.searches.value)
        assertEquals(recent.searches.value, RecentSearches(prefs).searches.value)
        recent.clear()
        assertEquals(emptyList<String>(), RecentSearches(prefs).searches.value)
    }

    @Test
    fun pressingSearchKeepsTheTypedWordsButNeverAFilterAlone() = runTest(dispatcher) {
        val search = SearchModel(this, FakeLibrary(), null, RecentSearches(prefs))
        search.setQuery("calm lakes")
        search.submit()
        search.choose(SearchType.Article.suggestion)
        search.submit()
        search.similarTo(testCard("lake"))
        search.submit()
        advanceUntilIdle()
        assertEquals(listOf("calm lakes"), search.recentSearches.value)
        search.forgetSearches()
        assertEquals(emptyList<String>(), search.recentSearches.value)
    }

    @Test
    fun aTappedRecentSearchRunsAgainAndComesFirst() = runTest(dispatcher) {
        val recent = RecentSearches(prefs).apply { add("thoreau"); add("foggy paintings") }
        val search = SearchModel(this, FakeLibrary(), null, recent)
        search.searchFor("thoreau")
        advanceUntilIdle()
        assertEquals("thoreau", search.query.value)
        assertEquals(listOf("hit"), search.results.value.state.value.cards.map { it.id })
        assertEquals(listOf("thoreau", "foggy paintings"), search.recentSearches.value)
    }

    @Test
    fun theEngineIsKnownBeforeASearchIsPressed() = runTest(dispatcher) {
        val search = SearchModel(this, FakeLibrary(), Ranker("Codex"))
        assertNull(search.engine.value)
        search.loadEngine()
        advanceUntilIdle()
        assertEquals("Codex", search.engine.value)
    }

    @Test
    fun onlyTheRankedCardsShowUntilAskedForEveryMatchAgain() = runTest(dispatcher) {
        val search = SearchModel(this, FakeLibrary(), Ranker("Claude Code"))
        search.setQuery("still water")
        search.submit()
        advanceUntilIdle()
        assertEquals("Claude Code ranked 2 first", aiNote(search.rank.value!!))
        search.showOnlyRanked(true)
        advanceUntilIdle()
        assertTrue(search.onlyRanked.value)
        assertEquals(listOf("lake", "pond"), search.results.value.state.value.cards.map { it.id })
        search.showOnlyRanked(false)
        advanceUntilIdle()
        assertEquals(listOf("lake", "pond", "hit"), search.results.value.state.value.cards.map { it.id })
        search.showOnlyRanked(true)
        search.setQuery("still water at dusk")
        advanceUntilIdle()
        assertFalse(search.onlyRanked.value)
    }
}

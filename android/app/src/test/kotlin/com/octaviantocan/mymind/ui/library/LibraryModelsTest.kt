package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.PageConfig
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.User
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuItem
import com.octaviantocan.mymind.ui.menu.CardMenuOutcome
import com.octaviantocan.mymind.ui.menu.CardSubmenu
import com.octaviantocan.mymind.ui.theme.PageScheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryModelsTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private fun TestScope.feedOf(library: FakeLibrary) = CardFeed(this) { library.cards(it) }

    @Test
    fun feedPagesUntilTheEndAndDropsRepeats() = runTest(dispatcher) {
        val library = FakeLibrary(
            pages = mapOf(
                null to CardPage(listOf(testCard("a"), testCard("b")), next = "p2"),
                "p2" to CardPage(listOf(testCard("b"), testCard("c"))),
            ),
        )
        val feed = feedOf(library)
        assertTrue(feed.state.value.empty)
        feed.loadMore()
        assertTrue(feed.state.value.loading)
        advanceUntilIdle()
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("a", "b", "c"), feed.state.value.cards.map { it.id })
        assertTrue(feed.state.value.endReached)
        feed.loadMore()
        advanceUntilIdle()
        assertEquals(listOf("cards:null", "cards:p2"), library.calls)
    }

    @Test
    fun feedReportsFailureAndRetries() = runTest(dispatcher) {
        val library = FakeLibrary(pages = emptyMap())
        val feed = feedOf(library)
        feed.loadMore()
        advanceUntilIdle()
        assertTrue(feed.state.value.error != null)
        assertTrue(feed.state.value.empty)
        library.pages = mapOf(null to CardPage(listOf(testCard("a"))))
        feed.loadMore()
        advanceUntilIdle()
        assertNull(feed.state.value.error)
        assertEquals(listOf("a"), feed.state.value.cards.map { it.id })
    }

    @Test
    fun feedRemovesAndReplacesCards() = runTest(dispatcher) {
        val feed = feedOf(FakeLibrary(pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("b"))))))
        feed.loadMore()
        advanceUntilIdle()
        feed.remove("a")
        feed.replace(testCard("b", title = "changed"))
        feed.replace(testCard("unknown"))
        assertEquals(listOf("b"), feed.state.value.cards.map { it.id })
        assertEquals("changed", feed.state.value.cards.single().title)
    }

    @Test
    fun searchSendsTypeAndTextAfterAPause() = runTest(dispatcher) {
        val library = FakeLibrary()
        val search = SearchModel(this, library)
        search.choose(SearchType.XPost.suggestion)
        advanceUntilIdle()
        assertEquals(SearchFilter("X Posts", "type:x-post"), search.filter.value)
        search.setQuery("ab")
        search.setQuery("abc")
        advanceTimeBy(100)
        assertEquals(listOf("search:type:x-post:null"), library.calls)
        advanceUntilIdle()
        assertEquals(listOf("search:type:x-post:null", "suggest:abc", "search:type:x-post abc:null"), library.calls)
        assertEquals(listOf("hit"), search.results.value.state.value.cards.map { it.id })
        search.clear()
        advanceUntilIdle()
        assertFalse(search.active)
        assertEquals(3, library.calls.size)
    }

    @Test
    fun tagSuggestionsBecomeQuotedTagFiltersAndClearTheText() = runTest(dispatcher) {
        val library = FakeLibrary(
            suggestions = mapOf("ro" to listOf(Suggestion("tag", "Road Trip", "tag:Road Trip"), Suggestion("tag", "Road Trip", "tag:Road Trip"))),
        )
        val search = SearchModel(this, library)
        search.setQuery("ro")
        advanceUntilIdle()
        assertEquals(listOf(Suggestion("tag", "Road Trip", "tag:Road Trip")), search.suggestions.value)
        search.choose(search.suggestions.value.single())
        advanceUntilIdle()
        assertEquals(SearchFilter("road trip", "tag:\"Road Trip\""), search.filter.value)
        assertEquals("", search.query.value)
        assertTrue(search.suggestions.value.isEmpty())
        assertEquals("search:tag:\"Road Trip\":null", library.calls.last())
        search.choose(Suggestion("tag", "Road Trip", "tag:Road Trip"))
        assertNull(search.filter.value)
    }

    @Test
    fun hashTextAsksForTagsAndTypesAreReadOnce() = runTest(dispatcher) {
        val library = FakeLibrary(
            allTags = listOf("recipes", "travel"),
            suggestions = mapOf("" to listOf(Suggestion("type", "Notes", "type:note"), Suggestion("tag", "x", "tag:x"))),
        )
        val search = SearchModel(this, library)
        search.setQuery("#rec")
        advanceUntilIdle()
        assertEquals(listOf(Suggestion("tag", "recipes", "#recipes")), search.suggestions.value)
        search.loadTypes()
        advanceUntilIdle()
        search.loadTypes()
        advanceUntilIdle()
        assertEquals(listOf(Suggestion("type", "Notes", "type:note")), search.types.value)
        assertEquals(1, library.calls.count { it == "suggest:" })
    }

    @Test
    fun sameVibeSearchesOnlyTheSimilarQuery() = runTest(dispatcher) {
        val library = FakeLibrary()
        val search = SearchModel(this, library)
        search.similarTo(testCard("img"))
        search.setQuery("ignored")
        advanceUntilIdle()
        assertEquals(SearchFilter(SAME_VIBE_LABEL, "similar:img"), search.filter.value)
        assertEquals("search:similar:img:null", library.calls.last())
    }

    @Test
    fun searchTypeSlugsMatchTheServerSuggestions() {
        assertEquals("x-post", SearchType.XPost.slug)
        assertEquals("youtube-video", SearchType.YouTubeVideo.slug)
        assertEquals("software-application", SearchType.App.slug)
        assertEquals("content", SearchType.Snippet.slug)
        assertEquals(17, SearchType.entries.size)
    }

    @Test
    fun serendipityDealsThePageDeckInOrderAndCompletesOnce() = runTest(dispatcher) {
        val ids = (1..18).map { "c$it" }
        val cards = ids.associateWith { testCard(it) } + ("gone" to testCard("gone").copy(deleted = "2026-10-16T08:00:00Z"))
        val library = FakeLibrary(cardsById = cards - "c5", deckIds = ids + "gone")
        val model = SerendipityModel(this, library)
        model.deal()
        advanceUntilIdle()
        val deck = model.state.value.deck
        assertEquals(ids - "c5", deck.map { it.id })
        assertEquals("deck", library.calls.first())
        assertEquals(deck.first(), model.state.value.current)

        model.keep()
        assertEquals(1, model.state.value.index)
        assertEquals(deck[1].id, model.forget())
        assertTrue("trash:${deck[1].id}" in library.calls)
        repeat(deck.size - 2) { model.keep() }
        advanceUntilIdle()
        assertTrue(model.state.value.finished)
        model.keep()
        advanceUntilIdle()
        assertEquals(1, library.calls.count { it == "complete" })
    }

    @Test
    fun serendipityReportsAnEmptyDeckAndAFailedPage() = runTest(dispatcher) {
        val library = FakeLibrary(deckIds = emptyList())
        val model = SerendipityModel(this, library)
        model.deal()
        advanceUntilIdle()
        assertTrue(model.state.value.dealt)
        assertTrue(model.state.value.deck.isEmpty())
        assertNull(model.state.value.error)

        library.deckIds = null
        model.deal()
        advanceUntilIdle()
        assertFalse(model.state.value.dealt)
        assertNotNull(model.state.value.error)
    }

    @Test
    fun serendipityForgetKeepsTheCardWhenTheServerRefuses() = runTest(dispatcher) {
        val library = FakeLibrary(cardsById = mapOf("a" to testCard("a"), "b" to testCard("b")), deckIds = listOf("a", "b"))
        val model = SerendipityModel(this, library)
        model.deal()
        advanceUntilIdle()
        library.writeFailure = ApiFailure.Network
        assertNull(model.forget())
        assertEquals(0, model.state.value.index)
    }

    @Test
    fun viewModelReadsSchemeAndUpdatesListsAfterMenuActions() = runTest(dispatcher) {
        val library = FakeLibrary(
            pages = mapOf(null to CardPage(listOf(testCard("a"), testCard("b")))),
            config = PageConfig(colorScheme = "dark", features = setOf("reader", "sameVibe")),
        )
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()
        assertEquals(PageScheme.Dark, viewModel.scheme.value)
        assertEquals(setOf("reader", "sameVibe"), viewModel.features.value)
        assertFalse("me" in library.calls)

        val card = viewModel.everything.state.value.cards.first()
        viewModel.perform(CardMenuItem(CardMenuAction.AddToTopOfMind), card, null)
        assertTrue(viewModel.everything.state.value.cards.first().pinned)
        viewModel.perform(CardMenuItem(CardMenuAction.DeleteCard), card, null)
        assertEquals(listOf("b"), viewModel.everything.state.value.cards.map { it.id })
    }

    @Test
    fun schemeFallsBackToTheAccountWhenThePageConfigFails() = runTest(dispatcher) {
        val library = FakeLibrary(config = null, user = User(colorScheme = "light"))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()
        assertEquals(PageScheme.Light, viewModel.scheme.value)
        assertTrue(viewModel.features.value.isEmpty())
    }

    @Test
    fun tagsAreAddedOneByOneAndRemovedByTheirId() = runTest(dispatcher) {
        val card = testCard("a", tags = listOf(Tag("t-old", "old")))
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(card))))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()

        val tagged = (viewModel.addTags(card, listOf("one", "two")) as ApiResult.Ok).value
        assertEquals(listOf("old", "one", "two"), tagged.tags?.map { it.name })
        assertEquals(tagged, viewModel.everything.state.value.cards.single())
        val untagged = (viewModel.removeTag(tagged, Tag("t-old", "old")) as ApiResult.Ok).value
        assertEquals(listOf("one", "two"), untagged.tags?.map { it.name })
        assertEquals(listOf("tag:a:one", "tag:a:two", "untag:t-old"), library.calls.filter { it.startsWith("tag:") || it.startsWith("untag:") })

        library.writeFailure = ApiFailure.Server(500)
        assertEquals(ApiResult.Failed(ApiFailure.Server(500)), viewModel.addTags(untagged, listOf("three")))
    }

    @Test
    fun spacesToggleCreateAndDelete() = runTest(dispatcher) {
        val card = testCard("a", spaces = listOf("s1"))
        val s1 = Space("s1", "One")
        val s2 = Space("s2", "Two")
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(card))), spaceList = listOf(s1, s2))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        viewModel.loadSpaces()
        advanceUntilIdle()

        val joined = (viewModel.toggleSpace(card, s2) as ApiResult.Ok).value
        assertEquals(listOf("s1", "s2"), joined.spaces)
        val left = (viewModel.toggleSpace(joined, s1) as ApiResult.Ok).value
        assertEquals(listOf("s2"), left.spaces)
        assertEquals(left, viewModel.everything.state.value.cards.single())
        assertEquals(ApiResult.Ok("s-new"), viewModel.createSpace("New", "#f96"))
        advanceUntilIdle()
        assertEquals(ApiResult.Ok(Unit), viewModel.deleteSpace(s2))
        assertEquals(listOf("s1"), viewModel.spaces.value.spaces.map { it.id })
        assertEquals(emptyList<String>(), viewModel.everything.state.value.cards.single().spaces)
        assertEquals(
            listOf("space:s2:a", "unspace:s1:a", "newSpace:New:#f96", "deleteSpace:s2"),
            library.calls.filter { it.startsWith("space:") || it.startsWith("unspace:") || it.startsWith("newSpace:") || it.startsWith("deleteSpace:") },
        )
    }

    @Test
    fun openingTheTagsSubmenuReadsTheTagChoices() = runTest(dispatcher) {
        val library = FakeLibrary(allTags = listOf("b", "a"), recent = listOf("a"))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        val outcome = viewModel.perform(CardMenuItem(CardMenuAction.AddTags), testCard("c"), null)
        advanceUntilIdle()
        assertEquals(CardMenuOutcome.OpenSubmenu(CardSubmenu.Tags), outcome)
        assertEquals(TagChoices(listOf("b", "a"), listOf("a")), viewModel.tagChoices.value)
    }
}

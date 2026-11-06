package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Tag
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class CardChoiceTest {
    private val dispatcher = StandardTestDispatcher()
    private val cards = (0..11).map { testCard("c$it") }

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun aSweepChoosesEveryFindFromTheHeldOneToTheFingerAndDropsThemOnTheWayBack() {
        val choice = CardChoice()
        choice.beginSweep(cards, 2)
        assertEquals(listOf("c2"), choice.cards.map { it.id })
        // Down two rows of three: the whole run, not only the finds the finger passed over.
        choice.sweepTo(cards, 8)
        assertEquals((2..8).map { "c$it" }, choice.cards.map { it.id })
        choice.sweepTo(cards, 5)
        assertEquals((2..5).map { "c$it" }, choice.cards.map { it.id })
        // Up past where it began chooses the other way.
        choice.sweepTo(cards, 0)
        assertEquals(listOf("c2", "c1", "c0"), choice.cards.map { it.id })
        choice.endSweep()
        assertTrue(choice.active)
        assertFalse(choice.sweeping)
    }

    @Test
    fun aSecondSweepKeepsWhatTheFirstChoseAndTapsAddOrDrop() {
        val choice = CardChoice()
        choice.beginSweep(cards, 0)
        choice.sweepTo(cards, 1)
        choice.endSweep()
        choice.beginSweep(cards, 6)
        choice.sweepTo(cards, 7)
        choice.endSweep()
        assertEquals(listOf("c0", "c1", "c6", "c7"), choice.cards.map { it.id })
        choice.toggle(cards[1])
        choice.toggle(cards[10])
        assertEquals(listOf("c0", "c6", "c7", "c10"), choice.cards.map { it.id })
        choice.refresh(gone = setOf("c6"))
        assertEquals(listOf("c0", "c7", "c10"), choice.cards.map { it.id })
        choice.clear()
        assertFalse(choice.active)
    }

    @Test
    fun tagSharesCountTheChosenFindsWithoutRegardToCase() {
        val chosen = listOf(
            testCard("a", tags = listOf(Tag("1", "Kyoto"), Tag("2", "trip"))),
            testCard("b", tags = listOf(Tag("3", "kyoto"))),
            testCard("c"),
        )
        val counts = tagCounts(chosen)
        assertEquals(mapOf("kyoto" to 2, "trip" to 1), counts)
        assertEquals(TagShare.Some, tagShare(counts.getValue("kyoto"), 3))
        assertEquals(TagShare.All, tagShare(3, 3))
        assertEquals(TagShare.None, tagShare(0, 3))
    }

    @Test
    fun taggingManyPutsTagsOnlyWhereMissingAndUndoTakesBackExactlyThat() = runTest(dispatcher) {
        val a = testCard("a", tags = listOf(Tag("t-a-old", "kyoto")))
        val b = testCard("b")
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(a, b))))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()

        val change = viewModel.tagMany(listOf(a, b), put = listOf("Kyoto", "trip"), takeOff = emptyList())
        assertEquals(listOf("tag:a:trip", "tag:b:Kyoto", "tag:b:trip"), library.calls.filter { it.startsWith("tag:") })
        val shown = viewModel.everything.state.value.cards.associate { card -> card.id to card.tags.orEmpty().map { it.name } }
        assertEquals(listOf("kyoto", "trip"), shown["a"])
        assertEquals(listOf("Kyoto", "trip"), shown["b"])

        library.calls.clear()
        viewModel.undoMany(change)
        // a keeps the kyoto it had before; only what the batch added comes off.
        assertEquals(listOf("untag:t-trip", "untag:t-Kyoto", "untag:t-trip"), library.calls.filter { it.startsWith("untag:") })
        val after = viewModel.everything.state.value.cards.associate { card -> card.id to card.tags.orEmpty().map { it.name } }
        assertEquals(listOf("kyoto"), after["a"])
        assertEquals(emptyList<String>(), after["b"])
    }

    @Test
    fun takingATagOffManyAndUndoingPutsItBack() = runTest(dispatcher) {
        val a = testCard("a", tags = listOf(Tag("t1", "Trip")))
        val b = testCard("b", tags = listOf(Tag("t2", "trip"), Tag("t3", "food")))
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(a, b))))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()

        val change = viewModel.tagMany(listOf(a, b), put = emptyList(), takeOff = listOf("trip"))
        assertEquals(listOf("untag:t1", "untag:t2"), library.calls.filter { it.startsWith("untag:") })
        assertEquals(listOf("food"), viewModel.everything.state.value.cards.last().tags?.map { it.name })
        viewModel.undoMany(change)
        assertEquals(listOf("tag:a:Trip", "tag:b:trip"), library.calls.filter { it.startsWith("tag:") })
    }

    @Test
    fun puttingManyInASpaceSkipsThoseAlreadyInItAndUndoTakesOutOnlyTheRest() = runTest(dispatcher) {
        val a = testCard("a", spaces = listOf("s1"))
        val b = testCard("b")
        val library = FakeLibrary(pages = mapOf(null to CardPage(listOf(a, b))), spaceList = listOf(Space("s1", "One")))
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()

        val change = viewModel.putManyInSpace(listOf(a, b), Space("s1", "One"))
        assertEquals(listOf("space:s1:b"), library.calls.filter { it.startsWith("space:") })
        assertEquals(listOf("s1"), viewModel.everything.state.value.cards.last().spaces)
        viewModel.undoMany(change)
        assertEquals(listOf("unspace:s1:b"), library.calls.filter { it.startsWith("unspace:") })
        assertEquals(listOf("s1"), viewModel.everything.state.value.cards.first().spaces)
    }
}

package com.octaviantocan.mymind.ui.menu

import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.library.testCard
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class CardSpacesTest {
    private val spaces = listOf(Space("s1", "One"), Space("s2", "Two"), Space("s3", "Three"), Space("own", "Own"), Space("smart", "Smart", smart = true))
    private val card = testCard("c", url = "https://www.example.com/x", tags = listOf(Tag("t", "Design")), spaces = listOf("own"))
    private val library = listOf(
        testCard("1", url = "https://example.com/y", spaces = listOf("s1")),
        testCard("2", tags = listOf(Tag("u", " design ")), spaces = listOf("s2", "own", "smart")),
        testCard("3", tags = listOf(Tag("u", "DESIGN")), spaces = listOf("s2")),
        testCard("4", tags = listOf(Tag("v", "other")), spaces = listOf("s3", "s3")),
        card.copy(spaces = listOf("s3")),
    )

    @Test
    fun theSuggestedSpaceHoldsTheMostCardsSharingASiteOrTagLeavingOutTheCardsOwnAndSmartSpaces() {
        assertEquals(SpaceSuggestion(spaces[1], 2), suggestSpace(card, library, spaces))
        assertEquals(SpaceSuggestion(spaces[0], 1), suggestSpace(card, library.take(1), spaces))
        assertNull(suggestSpace(testCard("n", type = "Note"), library, spaces))
        assertNull(suggestSpace(card, library.drop(3), spaces))
    }

    @Test
    fun thymindsPickWinsWhenItIsAnOpenSpace() {
        assertEquals(SpaceSuggestion(spaces[2], 0, byThyMind = true), suggestSpace(card.copy(suggestedSpace = "s3"), library.take(3), spaces))
        assertEquals(SpaceSuggestion(spaces[0], 0, byThyMind = true), suggestSpace(testCard("n", type = "Note").copy(suggestedSpace = "s1"), library, spaces))
        assertEquals(SpaceSuggestion(spaces[1], 2), suggestSpace(card.copy(suggestedSpace = "own"), library, spaces))
        assertEquals(SpaceSuggestion(spaces[1], 2), suggestSpace(card.copy(suggestedSpace = "smart"), library, spaces))
        assertEquals(SpaceSuggestion(spaces[1], 2), suggestSpace(card.copy(suggestedSpace = "gone"), library, spaces))
    }

    @Test
    fun chooseOneShowsTheSuggestedSpaceThenTheCardsOwnThenTheRestUpToThree() {
        assertEquals(listOf("s2", "own", "s1"), inlineSpaces(card, spaces, SpaceSuggestion(spaces[1], 2)).map { it.id })
        assertEquals(listOf("own", "s1", "s2"), inlineSpaces(card, spaces, null).map { it.id })
        assertEquals(listOf("s1"), inlineSpaces(testCard("n"), spaces.take(1), null).map { it.id })
    }
}

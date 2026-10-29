package com.octaviantocan.mymind.ui.share

import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import org.junit.Assert.assertEquals
import org.junit.Test

class ShareSuggestionsTest {
    private val reading = Space("s1", "Reading", "#ff5924", count = 214)
    private val walks = Space("s2", "Walks", "#5e8f4a", count = 38)
    private val smart = Space("s3", "Walking videos", "#4080df", smart = true)
    private val spaces = listOf(reading, walks, smart)
    private val tags = listOf("walking", "essay", "woods", "to read", "Wildness")

    private fun rows(typed: String, chosenSpaces: Set<String> = emptySet(), chosenTags: List<String> = emptyList(), suggestion: SpaceSuggestion? = null) =
        shareSuggestions(typed, spaces, chosenSpaces, tags, listOf("to read", "essay", "recipes", "books"), chosenTags, suggestion)

    @Test
    fun withNothingTypedItOffersTheSuggestedSpaceThenRecentTags() {
        assertEquals(
            listOf(
                ShareSuggestion.AddToSpace(reading, "Suggested · 12 cards like it"),
                ShareSuggestion.AddTag("to read", recent = true),
                ShareSuggestion.AddTag("recipes", recent = true),
                ShareSuggestion.AddTag("books", recent = true),
            ),
            rows("", chosenTags = listOf("Essay"), suggestion = SpaceSuggestion(reading, 12)),
        )
    }

    @Test
    fun aChosenSuggestedSpaceIsNotOfferedAgain() {
        assertEquals(ShareSuggestion.AddTag("to read", recent = true), rows("", setOf("s1"), suggestion = SpaceSuggestion(reading, 1)).first())
    }

    @Test
    fun aWordFindsSpacesAndTagsAndANewTagWhenNoneIsThatWord() {
        assertEquals(
            listOf(
                ShareSuggestion.AddToSpace(walks, "38 cards"),
                ShareSuggestion.AddTag("walking"),
                ShareSuggestion.AddTag("wal", fresh = true),
            ),
            rows("wal"),
        )
    }

    @Test
    fun aTagThatIsTheWordIsNotOfferedAsNew() {
        assertEquals(listOf(ShareSuggestion.AddTag("woods")), rows("Woods"))
        assertEquals(listOf(ShareSuggestion.AddTag("Wildness")), rows("#wild").filterNot { it is ShareSuggestion.AddTag && it.fresh })
    }

    @Test
    fun wordsWithASpaceCanBecomeTheNote() {
        val typed = "Read before Sunday’s walk"
        assertEquals(ShareSuggestion.UseAsNote(typed), rows(" $typed ").last())
        assertEquals(listOf(ShareSuggestion.AddTag("to read"), ShareSuggestion.UseAsNote("to read")), rows("to read"))
    }

    @Test
    fun chosenThingsAreLeftOut() {
        assertEquals(listOf(ShareSuggestion.AddTag("wal", fresh = true)), rows("wal", setOf("s2"), listOf("walking")))
    }

    @Test
    fun aLinkShowsItsAddressUntilThyMindReadsThePage() {
        val state = ShareSheetState(listOf(ShareItem.Text("https://www.theatlantic.com/magazine/walking/")), SavePhase.Saved)
        state.cards = listOf(testCard("c1").copy(type = "Placeholder", title = null, domain = "theatlantic.com"))
        assertEquals(SavedLines("theatlantic.com/magazine/walking", "Getting the page…"), savedLines(state))
        state.cards = listOf(testCard("c1").copy(type = "Article", title = "Walking", domain = "theatlantic.com", siteName = "The Atlantic", author = "Henry David Thoreau"))
        assertEquals(SavedLines("Walking", "The Atlantic · Henry David Thoreau"), savedLines(state))
    }

    @Test
    fun severalFilesCountWhatIsSavedAndWhatIsNot() {
        val state = ShareSheetState(listOf(ShareItem.Stream("a"), ShareItem.Stream("b"), ShareItem.Stream("c")), SavePhase.Saving)
        state.files = listOf(SharedFile("a", "a.jpg", "image/jpeg", 2_200_000), SharedFile("b", "b.jpg", "image/jpeg", 1_000_000), SharedFile("c", "c.png", "image/png", 300_000))
        state.saved = 1
        assertEquals(SavedLines("3 photos", "3.3 MB"), savedLines(state))
        assertEquals("Saving 2 of 3", statusText(state))
        state.saved = 2
        state.failed = 1
        state.phase = SavePhase.Saved
        assertEquals(SavedLines("3 photos", "2 saved, 1 could not be"), savedLines(state))
        assertEquals("Saved to your mind", statusText(state))
    }

    @Test
    fun aShareThatFailedSaysWhy() {
        val state = ShareSheetState(listOf(ShareItem.Stream("a")), SavePhase.Failed)
        state.files = listOf(SharedFile("a", "Hike.mp4", "video/mp4", 31_000_000))
        state.problem = "Cannot reach the server. Check your connection."
        assertEquals(SavedLines("Hike.mp4", "Cannot reach the server. Check your connection."), savedLines(state))
        assertEquals("Nothing was saved", statusText(state))
    }

    @Test
    fun addingARowEmptiesTheField() {
        val state = ShareSheetState(listOf(ShareItem.Text("https://example.com")))
        state.typed = "Read it on Sunday"
        state.add(ShareSuggestion.UseAsNote("Read it on Sunday"))
        state.add(ShareSuggestion.AddToSpace(walks, ""))
        state.add(ShareSuggestion.AddTag("#essay"))
        assertEquals("", state.typed)
        assertEquals(com.octaviantocan.mymind.share.ShareDetails(listOf("essay"), "Read it on Sunday", setOf("s2")), state.details())
    }
}

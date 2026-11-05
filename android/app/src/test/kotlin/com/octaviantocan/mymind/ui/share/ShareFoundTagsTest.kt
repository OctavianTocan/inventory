package com.octaviantocan.mymind.ui.share

import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.share.ShareDetails
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.library.testCard
import org.junit.Assert.assertEquals
import org.junit.Test

class ShareFoundTagsTest {
    private val cardTags = listOf(Tag("t1", "gardens", flags = 2), Tag("t2", "mine", flags = 0), Tag("t3", "Soil", flags = 2), Tag("t4", "soil", flags = 2))

    private fun sheet(vararg items: ShareItem) = ShareSheetState(items.toList(), SavePhase.Saved).apply { cards = listOf(testCard("c1", tags = cardTags)) }

    @Test
    fun theSheetShowsTheTagsThymindPutOnWhatWasSaved() {
        assertEquals(listOf("gardens", "Soil"), sheet(ShareItem.Text("https://a.test")).foundTags)
    }

    @Test
    fun severalThingsSharedShowNoneOfThymindsTags() {
        assertEquals(emptyList<String>(), sheet(ShareItem.Text("https://a.test"), ShareItem.Text("https://b.test")).foundTags)
    }

    @Test
    fun aTagTakenOffLeavesTheSheetAndGoesWithTheDetailsUntilItIsAddedBack() {
        val state = sheet(ShareItem.Text("https://a.test"))
        state.untagged = state.untagged + "gardens"
        assertEquals(listOf("Soil"), state.foundTags)
        assertEquals(ShareDetails(untag = setOf("gardens")), state.details())
        state.allTags = listOf("gardens", "mine")
        state.typed = "gard"
        assertEquals(listOf(ShareSuggestion.AddTag("gardens"), ShareSuggestion.AddTag("gard", fresh = true)), state.suggestions())
        state.add(ShareSuggestion.AddTag("Gardens"))
        assertEquals(listOf("gardens", "Soil"), state.foundTags)
        assertEquals(ShareDetails(), state.details())
    }
}

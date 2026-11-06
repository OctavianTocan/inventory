package com.octaviantocan.mymind.ui.moment

import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareSheetState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** share-tag-step-icon-grid: which tiles the grid shows and what a tap on one does. */
class TagGridTest {
    private fun state() = ShareSheetState(listOf(ShareItem.Text("https://example.com/m6")), SavePhase.Saved).apply {
        cards = listOf(testCard("c1", tags = listOf(Tag("t1", "film camera", 2), Tag("t2", "Rangefinder", 2))))
        recentTags = listOf("rangefinder", "35mm", "vintage")
        allTags = listOf("film camera", "35mm", "vintage", "photography")
    }

    @Test
    fun thymindsTagsComeFirstThenRecentOnesEachOnce() {
        val tiles = tagTiles(state())
        assertEquals(listOf("film camera", "Rangefinder", "35mm", "vintage"), tiles.map { it.tag })
        assertEquals(listOf(true, true, false, false), tiles.map { it.on })
        assertEquals(listOf(true, true, false, false), tiles.map { it.byThyMind })
    }

    @Test
    fun typingFindsAnyTagThatHoldsTheWords() {
        val s = state().apply { typed = "PHOTO" }
        assertEquals(listOf("photography"), tagTiles(s).map { it.tag })
    }

    @Test
    fun aTapTakesThymindsTagOffAndPutsItBack() {
        val s = state()
        toggleTag(s, tagTiles(s).first { it.tag == "Rangefinder" })
        assertEquals(setOf("Rangefinder"), s.untagged)
        val off = tagTiles(s).first { it.tag == "Rangefinder" }
        assertEquals(TagTile("Rangefinder", on = false, byThyMind = true), off)
        toggleTag(s, off)
        assertTrue(s.untagged.isEmpty())
        assertTrue(s.tags.isEmpty())
    }

    @Test
    fun aTapAddsAndRemovesTheUsersOwnTags() {
        val s = state()
        toggleTag(s, tagTiles(s).first { it.tag == "35mm" })
        assertEquals(listOf("35mm"), s.tags)
        toggleTag(s, tagTiles(s).first { it.tag == "35mm" })
        assertTrue(s.tags.isEmpty())
        assertEquals("The details carry what was taken off", setOf("film camera"), s.run {
            toggleTag(this, tagTiles(this).first { it.tag == "film camera" }); details().untag
        })
    }

    @Test
    fun iconsFollowWholeWordsNotLettersInside() {
        assertEquals(tagIcon("camera"), tagIcon("film camera"))
        assertEquals(tagIcon("vintage"), tagIcon("1980s"))
        assertNotEquals("rain is not ai", tagIcon("ai"), tagIcon("rain"))
        assertEquals(tagIcon("ai"), tagIcon("AI tools"))
    }
}

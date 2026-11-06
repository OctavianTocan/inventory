package com.octaviantocan.mymind.ui.moment

import com.octaviantocan.mymind.ui.library.testCard
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** find-rarity-from-library: a find's rarity comes from how many earlier cards share its source. */
class FindRarityTest {
    @Test
    fun theFirstFromASourceIsLegendaryAFewAreRareMoreAreCommon() {
        assertEquals(Rarity.Legendary, rarityOf(0))
        assertEquals(Rarity.Rare, rarityOf(1))
        assertEquals(Rarity.Rare, rarityOf(RARE_BELOW - 1))
        assertEquals(Rarity.Common, rarityOf(RARE_BELOW))
        assertEquals(listOf(1, 3, 5), Rarity.entries.map { it.stars })
        assertEquals(listOf(10, 40, 80), Rarity.entries.map { it.points })
    }

    @Test
    fun oneSiteUnderAllItsNames() {
        assertEquals("site:x.com", sourceOfShare("https://twitter.com/a/status/1", null))
        assertEquals("site:x.com", sourceOfShare("https://mobile.x.com/a", null))
        assertEquals("site:youtube.com", sourceOfShare("https://youtu.be/abc", null))
        assertEquals("site:youtube.com", sourceOfShare("https://music.youtube.com/watch?v=1", null))
        assertEquals("site:youtube.com", sourceOfShare("https://m.youtube.com/watch?v=1", null))
        assertEquals("site:leica-camera.com", sourceOfShare("https://www.Leica-Camera.com/m6", null))
    }

    @Test
    fun filesAndNotesCountByKind() {
        assertEquals("kind:Image", sourceOfShare(null, "image/jpeg"))
        assertEquals("kind:Video", sourceOfShare(null, "video/mp4"))
        assertEquals("kind:Audio", sourceOfShare(null, "audio/mpeg"))
        assertEquals("kind:PDF", sourceOfShare(null, "application/pdf"))
        assertEquals("kind:Document", sourceOfShare(null, "application/zip"))
        assertEquals("kind:Note", sourceOfShare(null, null))
        assertEquals("kind:Image", sourceOf(testCard("p", type = "Image")))
        assertNull("A card not read yet has no source", sourceOf(testCard("p", type = "Placeholder")))
    }

    @Test
    fun countsTheLibraryWithoutWhatThisShareSaved() {
        val cards = listOf(
            testCard("a", url = "https://www.youtube.com/watch?v=1"),
            testCard("b", url = "https://youtu.be/2"),
            testCard("c", url = "https://example.com/x"),
            testCard("new", url = "https://youtu.be/3"),
        )
        assertEquals(Rarity.Rare, rarityIn(cards, "site:youtube.com", leaveOut = setOf("new")))
        assertEquals(Rarity.Legendary, rarityIn(cards, "site:vimeo.com"))
        val many = (1..RARE_BELOW).map { testCard("n$it", type = "Note", url = null) }
        assertEquals(Rarity.Common, rarityIn(many, "kind:Note"))
    }
}

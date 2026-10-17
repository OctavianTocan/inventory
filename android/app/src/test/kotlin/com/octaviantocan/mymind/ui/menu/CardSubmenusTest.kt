package com.octaviantocan.mymind.ui.menu

import org.junit.Assert.assertEquals
import org.junit.Test

class CardSubmenusTest {
    @Test
    fun typedTextFiltersAllTagsTakesFiveAndSortsThem() {
        val all = listOf("zeta", "Alpha", "alps", "salt", "halo", "pal", "beta", "aal")
        assertEquals(listOf("Alpha", "alps", "halo", "pal", "salt"), tagChoices(all, recent = listOf("x"), text = "al"))
    }

    @Test
    fun emptyTextOffersTheRecentTagsSorted() {
        assertEquals(listOf("a", "B", "c"), tagChoices(listOf("q"), recent = listOf("c", "a", "B"), text = ""))
    }

    @Test
    fun addedTagsAreTrimmedWithoutHashAndOnlyOnce() {
        assertEquals(listOf("one"), addTag(emptyList(), "  #one "))
        assertEquals(listOf("one"), addTag(listOf("one"), "ONE"))
        assertEquals(listOf("one"), addTag(listOf("one"), " # "))
        assertEquals(listOf("one", "two"), addTag(listOf("one"), "two"))
    }
}

package com.octaviantocan.mymind.ui.moment

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** share-moment-shooting-star: the moment's beats, in moment time, without drawing it. */
class StarSimTest {
    private val cues = ArrayList<Cue>()

    private fun sim(rarity: Rarity?) = StarSim(360f, 800f, 30f, seed = 1L) { cues += it }.also { it.rarity = rarity }

    private fun StarSim.run(ms: Int, until: (StarSim) -> Boolean = { false }): Int {
        var t = 0
        while (t < ms && !until(this)) { step(16f); t += 16 }
        return t
    }

    @Test
    fun eachRaritySettlesWithItsStarsAndColourTurns() {
        for (r in Rarity.entries) {
            cues.clear()
            val s = sim(r)
            val took = s.run(20_000) { it.phase == StarPhase.Settled }
            assertEquals("$r settles", StarPhase.Settled, s.phase)
            assertTrue("$r settles in time: $took ms", took < 9_000)
            assertEquals(r.stars, s.stars)
            assertEquals(r.points, s.pointsValue)
            assertEquals("$r turns colour once per tease step", r.tease.size, cues.count { it is Cue.Ting })
            assertEquals(r.stars, cues.count { it is Cue.Star })
            assertEquals("One hit as the face comes round", 1, cues.count { it is Cue.Boom })
            assertTrue(cues.contains(Cue.Chime(r)))
        }
    }

    @Test
    fun rarityIsKeptOnceTheStarBreaksOff() {
        val s = sim(null)
        s.run(1_400)
        s.rarity = Rarity.Legendary
        assertEquals("Common when unknown at launch", Rarity.Common, s.shownRarity)
    }

    @Test
    fun aTapFliesItHomeAndSkipsTheTagStep() {
        val s = sim(Rarity.Rare)
        s.run(900)
        s.skip()
        assertTrue(s.skipped)
        s.run(5_000) { it.phase == StarPhase.Done }
        assertEquals(StarPhase.Done, s.phase)
        assertTrue("The find was counted in the backpack", s.bagCounted)
        assertFalse(s.showingBack)
    }

    @Test
    fun theCardTurnsToTheTagStepAndBackThenLeaves() {
        val s = sim(Rarity.Common)
        s.run(20_000) { it.phase == StarPhase.Settled }
        s.turnToBack(400f, 2f)
        s.run(StarSim.TURN_MS.toInt() + 100)
        assertTrue(s.showingBack)
        s.run(10_000)
        assertEquals("It waits on the tag step", StarPhase.Settled, s.phase)
        s.turnToFace()
        s.run(StarSim.TURN_MS.toInt() + 100) { it.phase == StarPhase.Leaving }
        assertFalse(s.showingBack)
        assertEquals(StarPhase.Leaving, s.phase)
        s.run(5_000) { it.phase == StarPhase.Done }
        assertEquals(StarPhase.Done, s.phase)
        assertTrue(s.bagCounted)
    }

    @Test
    fun anAbortFadesWithoutTheBackpack() {
        val s = sim(Rarity.Common)
        s.run(2_000)
        s.abort()
        s.run(1_000) { it.phase == StarPhase.Done }
        assertEquals(StarPhase.Done, s.phase)
        assertFalse(s.bagCounted)
    }
}

package com.octaviantocan.mymind.ui.moment

import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.Random
import kotlin.math.abs

/** share-moment-sounds: every beat's recipe makes a finite sound that the mixer keeps in range. */
class MomentSoundTest {
    @Test
    fun everyCueRendersAndStaysInRange() {
        val rate = 48_000
        val cues = listOf(
            Cue.Shimmer, Cue.Ting(0), Cue.Ting(2), Cue.Meteor(1.6f), Cue.Riser(1.4f), Cue.Suck, Cue.Swoop, Cue.Thud,
            Cue.Boom(1f), Cue.Boom(0.3f), Cue.Chime(Rarity.Legendary), Cue.Chime(Rarity.Common), Cue.Star(4), Cue.Count, Cue.Pop, Cue.Whoosh,
        )
        val voices = cues.flatMap { voicesOf(it, rate, Random(3)) }.toMutableList()
        assertTrue(voices.isNotEmpty())
        val mixer = Mixer(rate)
        var peak = 0f
        repeat(rate * 4 / 256) {
            val block = FloatArray(256)
            voices.forEach { it.render(block) }
            voices.removeAll { it.done }
            mixer.process(block)
            block.forEach { v -> assertTrue("finite", v.isFinite()); peak = maxOf(peak, abs(v)) }
        }
        assertTrue("It is heard: $peak", peak > 0.05f)
        assertTrue("It never clips: $peak", peak <= 1f)
        assertTrue("Every voice ends", voices.isEmpty())
    }
}

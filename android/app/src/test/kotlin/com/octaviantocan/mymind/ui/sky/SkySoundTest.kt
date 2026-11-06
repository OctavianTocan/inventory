package com.octaviantocan.mymind.ui.sky

import com.octaviantocan.mymind.ui.moment.Mixer
import com.octaviantocan.mymind.ui.moment.Voice
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.DataOutputStream
import java.io.File
import java.io.FileOutputStream
import java.util.Random
import kotlin.math.abs

/** sky-sounds: every beat's recipe makes a finite sound that the sky's mixer keeps in range. */
class SkySoundTest {
    private val rate = 48_000

    @Test
    fun everyCueRendersAndStaysInRange() {
        val cues = listOf(
            SkyCue.LookUp(2.6f), SkyCue.LookUp(0.4f), SkyCue.LookDown(1.3f), SkyCue.PullReady, SkyCue.Meteor(1.05f, 1f), SkyCue.Meteor(1.05f, 0.55f),
            SkyCue.Light(0), SkyCue.Light(20), SkyCue.Light(-1), SkyCue.Surge(0), SkyCue.Surge(5), SkyCue.Complete, SkyCue.Settle(1.9f),
            SkyCue.Vortex(3.6f), SkyCue.Appear(3), SkyCue.Named, SkyCue.Focus, SkyCue.Unfocus, SkyCue.Glide(0.95f, 2), SkyCue.Open,
            SkyCue.Mote(0.8f), SkyCue.Land,
        )
        for (cue in cues) {
            val samples = render(listOf(0f to cue), 6f)
            val peak = samples.maxOf { abs(it) }
            assertTrue("$cue is finite", samples.all { it.isFinite() })
            assertTrue("$cue is heard: $peak", peak > 0.01f)
            assertTrue("$cue never clips: $peak", peak <= 1f)
        }
        // Everything at once still stays in range.
        val all = render(cues.map { 0f to it }, 6f)
        assertTrue(all.all { it.isFinite() && abs(it) <= 1f })
    }

    /**
     * Writes a listening copy of the sky's sounds in the order a visit plays them, when `-Dsky.sound.preview=<file.wav>`
     * is set.
     */
    @Test
    fun previewWhenAsked() {
        val path = System.getProperty("sky.sound.preview") ?: System.getenv("SKY_SOUND_PREVIEW") ?: return
        val script = ArrayList<Pair<Float, SkyCue>>()
        var t = 0.3f
        fun at(dt: Float, cue: SkyCue) { t += dt; script += t to cue }
        at(0f, SkyCue.PullReady)
        at(0.5f, SkyCue.LookUp(1.3f))
        // Two finds light their stars.
        for (step in 0..1) { at(if (step == 0) 1.6f else 0.95f, SkyCue.Meteor(1.05f, 1f)); at(1.05f, SkyCue.Light(step)) }
        // The last star lights and the constellation completes.
        at(1.2f, SkyCue.Meteor(1.05f, 1f)); at(1.05f, SkyCue.Light(5))
        at(1.1f, SkyCue.Surge(0)); at(0.3f, SkyCue.Surge(1)); at(0.3f, SkyCue.Surge(2))
        at(0.55f, SkyCue.Complete)
        at(4.2f, SkyCue.Settle(1.9f))
        // The next one forms.
        at(2.2f, SkyCue.Vortex(3.6f))
        for (k in 0 until 6) at(if (k == 0) 2.35f else 0.18f, SkyCue.Appear(k))
        at(1.6f, SkyCue.Named)
        // A star is tapped, a swipe travels on, the find opens.
        at(1.8f, SkyCue.Focus)
        at(2.6f, SkyCue.Glide(0.95f, 3))
        at(2.4f, SkyCue.Open)
        at(1.4f, SkyCue.Unfocus)
        at(1.4f, SkyCue.LookDown(1.3f))
        // A Back today find closes and its star flies to the button.
        at(2.4f, SkyCue.Mote(0.8f)); at(0.8f, SkyCue.Land)
        writeWav(File(path), render(script, t + 3f))
    }

    private fun render(script: List<Pair<Float, SkyCue>>, seconds: Float): FloatArray {
        val random = Random(7)
        val mixer = Mixer(rate, SkySound.ROOM)
        val out = FloatArray((seconds * rate).toInt() / 256 * 256)
        val pending = script.sortedBy { it.first }.toMutableList()
        val active = ArrayList<Voice>()
        var at = 0
        while (at < out.size) {
            while (pending.isNotEmpty() && pending[0].first * rate <= at) active += skyVoices(pending.removeAt(0).second, rate, random)
            val block = FloatArray(256)
            active.forEach { it.render(block) }
            active.removeAll { it.done }
            mixer.process(block)
            block.copyInto(out, at)
            at += 256
        }
        return out
    }

    private fun writeWav(file: File, samples: FloatArray) {
        DataOutputStream(FileOutputStream(file)).use { o ->
            fun le32(v: Int) = o.write(byteArrayOf(v.toByte(), (v shr 8).toByte(), (v shr 16).toByte(), (v shr 24).toByte()))
            fun le16(v: Int) = o.write(byteArrayOf(v.toByte(), (v shr 8).toByte()))
            o.writeBytes("RIFF"); le32(36 + samples.size * 2); o.writeBytes("WAVEfmt ")
            le32(16); le16(1); le16(1); le32(rate); le32(rate * 2); le16(2); le16(16)
            o.writeBytes("data"); le32(samples.size * 2)
            for (v in samples) le16((v.coerceIn(-1f, 1f) * 32767).toInt())
        }
    }
}

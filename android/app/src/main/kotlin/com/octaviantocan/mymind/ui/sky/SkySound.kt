package com.octaviantocan.mymind.ui.sky

import android.content.Context
import com.octaviantocan.mymind.ui.moment.MomentSound
import com.octaviantocan.mymind.ui.moment.Noise
import com.octaviantocan.mymind.ui.moment.Room
import com.octaviantocan.mymind.ui.moment.Tone
import com.octaviantocan.mymind.ui.moment.Voice
import java.util.Random

/** A beat of the sky that makes a sound (sky-sounds). */
internal sealed interface SkyCue {
    /** The camera starts looking up, taking [seconds] to reach the sky. */
    data class LookUp(val seconds: Float) : SkyCue

    /** The camera starts back down to the page, taking [seconds]. */
    data class LookDown(val seconds: Float) : SkyCue

    /** A pull has gone far enough that letting go looks up. */
    data object PullReady : SkyCue

    /** A shooting star leaves for a star, reaching it in [seconds]; [big] is 1 for a constellation's, less for a loose one. */
    data class Meteor(val seconds: Float, val big: Float) : SkyCue

    /** A star lights; [step] counts the stars lit so far in its constellation (from 0), or -1 for a loose star. */
    data class Light(val step: Int) : SkyCue

    /** A completing constellation's surge reaches its [wave]th ring of stars. */
    data class Surge(val wave: Int) : SkyCue

    /** A constellation completes: its figure draws and its name turns gold. */
    data object Complete : SkyCue

    /** A finished constellation glides to its place among the others, taking [seconds]. */
    data class Settle(val seconds: Float) : SkyCue

    /** A new constellation gathers out of a vortex of light over [seconds]. */
    data class Vortex(val seconds: Float) : SkyCue

    /** The [index]th star of a forming constellation appears. */
    data class Appear(val index: Int) : SkyCue

    /** A new constellation's name has typed in. */
    data object Named : SkyCue

    /** The camera moves in on a star and its find condenses out of starlight. */
    data object Focus : SkyCue

    /** The find goes back into starlight and the camera moves out. */
    data object Unfocus : SkyCue

    /** A comet runs down the line to the next star, taking [seconds]; [step] picks the arriving note. */
    data class Glide(val seconds: Float, val step: Int) : SkyCue

    /** A find opens from its star. */
    data object Open : SkyCue

    /** A star leaves a closed Back today find for the star button, taking [seconds]. */
    data class Mote(val seconds: Float) : SkyCue

    /** That star lands in the star button. */
    data object Land : SkyCue
}

/**
 * The sky's sounds, made as they play by the share moment's synth (tones, filtered noise, bells) in a larger room. Like
 * the moment, nothing sounds unless the ringer is on, and it plays at the phone's media volume. The output opens
 * on the first sound and closes itself once the sky has been quiet for two seconds.
 */
internal class SkySound(context: Context) {
    private val out = MomentSound(context, "sky-sound", ROOM)
    private val random = Random()

    /** Plays [cue] now. */
    fun play(cue: SkyCue) {
        out.start()
        out.play(skyVoices(cue, MomentSound.RATE, random))
        out.stop()
    }

    companion object {
        /** A wider room than the moment's, so the sky's sounds hang in the air. */
        val ROOM = Room(0.5f, 0.88f)
    }
}

/** D major pentatonic, the scale the stars light up in as a constellation fills. */
private val STARS = floatArrayOf(587.33f, 659.26f, 739.99f, 880f, 987.77f, 1174.66f, 1318.51f, 1479.98f, 1760f)

private fun starNote(step: Int): Float = STARS[step.coerceIn(0, STARS.lastIndex)]

/**
 * @param cue A beat of the sky.
 * @param rate Samples a second.
 * @param random Source of sparkle scatter and noise.
 * @return The voices that make its sound.
 */
internal fun skyVoices(cue: SkyCue, rate: Int, random: Random): List<Voice> {
    val out = ArrayList<Voice>()
    fun tone(f: Float, dur: Float, wave: Int = Tone.SINE, vol: Float, f2: Float? = null, attack: Float = 0.008f, delay: Float = 0f) {
        out += Tone(f, f2, dur, wave, vol, attack, delay, rate)
    }
    fun noise(dur: Float, f1: Float, f2: Float, vol: Float, q: Float, attack: Float = 0.3f, highpass: Boolean = false, delay: Float = 0f) {
        out += Noise(dur, f1, f2, vol, q, attack, highpass, delay = delay, rate = rate, seed = random.nextLong())
    }
    fun bell(f: Float, at: Float, v: Float, d: Float) {
        tone(f, d, vol = v, delay = at); tone(f * 2.01f, d * 0.5f, vol = v * 0.35f, delay = at); tone(f * 3.98f, d * 0.25f, vol = v * 0.12f, delay = at)
    }
    // Fine glints scattered over [span] seconds from [at], their pitch moving from [lo] to [hi] as they go.
    fun sparkle(n: Int, at: Float, span: Float, lo: Float, hi: Float, vol: Float) = repeat(n) {
        val k = random.nextFloat()
        val f = lo + (hi - lo) * k + random.nextFloat() * 400f
        tone(f, 0.22f + random.nextFloat() * 0.2f, Tone.TRIANGLE, vol, delay = at + k * span)
    }
    when (cue) {
        is SkyCue.LookUp -> {
            val d = cue.seconds
            noise(d, 220f, 2600f, 0.11f, 0.8f, 0.75f)
            tone(146.83f, d + 1.4f, vol = 0.07f, attack = d * 0.8f); tone(147.6f, d + 1.4f, vol = 0.05f, attack = d * 0.8f)
            tone(220f, d + 1.2f, vol = 0.045f, attack = d * 0.85f); tone(293.66f, d + 1f, vol = 0.03f, attack = d * 0.9f)
            bell(880f, d * 0.92f, 0.05f, 1.6f); bell(1318.51f, d * 0.92f + 0.09f, 0.035f, 1.4f)
            sparkle(12, d * 0.75f, 0.9f, 2600f, 5200f, 0.018f)
        }
        is SkyCue.LookDown -> {
            val d = cue.seconds
            noise(d, 2600f, 240f, 0.09f, 0.8f, 0.2f)
            tone(293.66f, d, vol = 0.04f, f2 = 146.83f, attack = d * 0.25f)
        }
        SkyCue.PullReady -> { tone(1760f, 0.28f, vol = 0.06f); tone(2637f, 0.2f, vol = 0.025f, delay = 0.03f) }
        is SkyCue.Meteor -> {
            val d = cue.seconds
            val b = cue.big
            noise(d, 6500f, 900f, 0.12f * b + 0.03f, 1.2f, 0.7f)
            tone(2400f, d, Tone.TRIANGLE, 0.02f * b, 700f, d * 0.6f)
        }
        is SkyCue.Light -> {
            if (cue.step < 0) {
                bell(1760f, 0f, 0.07f, 1.2f)
                noise(0.25f, 7000f, 10000f, 0.04f, 3f, 0.04f)
            } else {
                val f = starNote(cue.step)
                bell(f, 0f, 0.13f, 1.8f); bell(f * 0.5f, 0.01f, 0.05f, 1.2f)
                noise(0.3f, 7000f, 10000f, 0.06f, 3f, 0.03f)
                tone(92f, 0.4f, vol = 0.22f, f2 = 46f, attack = 0.003f)
            }
            sparkle(6, 0.02f, 0.35f, 3200f, 6000f, 0.015f)
        }
        is SkyCue.Surge -> {
            val f = starNote(cue.wave * 2)
            tone(f, 0.55f, Tone.TRIANGLE, 0.05f); tone(f * 2f, 0.4f, vol = 0.03f)
            noise(0.4f, 1500f, 5000f, 0.04f, 2f, 0.6f)
        }
        SkyCue.Complete -> {
            listOf(293.66f, 440f, 587.33f, 739.99f, 880f, 1318.51f).forEachIndexed { i, f -> bell(f, i * 0.065f, 0.1f, 2.8f) }
            tone(73.42f, 3.2f, vol = 0.2f, attack = 0.02f); tone(110f, 3f, vol = 0.08f, attack = 0.05f); tone(146.83f, 2.6f, vol = 0.05f, attack = 0.3f)
            tone(82f, 1.2f, vol = 0.38f, f2 = 32f, attack = 0.004f)
            noise(1.4f, 3200f, 200f, 0.12f, 0.5f, 0.02f)
            noise(2.4f, 5000f, 9000f, 0.03f, 2f, 0.4f)
            sparkle(26, 0.15f, 2.4f, 2400f, 6000f, 0.018f)
        }
        is SkyCue.Settle -> noise(cue.seconds, 1200f, 300f, 0.06f, 0.7f, 0.45f)
        is SkyCue.Vortex -> {
            val d = cue.seconds
            // Overlapping swirls of resonant noise, alternately rising and falling, under a slow beating rise.
            val n = 7
            for (k in 0 until n) {
                val at = k * d / (n + 1)
                val up = k % 2 == 0
                noise(d / 3f, if (up) 500f else 3200f, if (up) 3200f else 600f, 0.06f + 0.012f * k, 4f, 0.5f, delay = at)
            }
            tone(110f, d + 0.6f, vol = 0.035f, f2 = 220f, attack = d * 0.85f); tone(111.2f, d + 0.6f, vol = 0.035f, f2 = 222.4f, attack = d * 0.85f)
            sparkle(18, 0.3f, d, 1800f, 5200f, 0.014f)
        }
        is SkyCue.Appear -> { val f = starNote(cue.index); tone(f, 0.45f, vol = 0.05f); tone(f * 2f, 0.25f, vol = 0.018f) }
        SkyCue.Named -> { bell(880f, 0f, 0.06f, 1.4f); bell(1174.66f, 0.09f, 0.05f, 1.6f) }
        SkyCue.Focus -> {
            noise(0.95f, 380f, 2200f, 0.08f, 0.9f, 0.55f)
            tone(587.33f, 2.4f, vol = 0.03f, attack = 0.9f); tone(880f, 2.2f, vol = 0.02f, attack = 1f)
            sparkle(28, 0.35f, 1.6f, 1600f, 5600f, 0.016f)
        }
        SkyCue.Unfocus -> {
            noise(0.7f, 2200f, 380f, 0.07f, 0.8f, 0.2f)
            sparkle(10, 0f, 0.5f, 5000f, 2200f, 0.014f)
        }
        is SkyCue.Glide -> {
            val d = cue.seconds
            noise(d, 900f, 4800f, 0.09f, 1.5f, 0.5f)
            tone(1200f, d, Tone.TRIANGLE, 0.018f, 1900f, d * 0.5f)
            bell(starNote(cue.step), d * 0.95f, 0.08f, 1.4f)
        }
        SkyCue.Open -> { tone(440f, 0.16f, vol = 0.12f, f2 = 990f); noise(0.08f, 3000f, 5000f, 0.05f, 2f) }
        is SkyCue.Mote -> {
            val d = cue.seconds
            noise(d, 1400f, 5200f, 0.06f, 1.2f, 0.8f)
            tone(1500f, d, Tone.TRIANGLE, 0.015f, 2600f, d * 0.7f)
        }
        SkyCue.Land -> {
            bell(1760f, 0f, 0.09f, 1f); bell(2637f, 0.05f, 0.04f, 0.8f)
            tone(130f, 0.22f, vol = 0.14f, f2 = 65f, attack = 0.003f)
            sparkle(5, 0f, 0.25f, 3500f, 6000f, 0.014f)
        }
    }
    return out
}

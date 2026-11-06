package com.octaviantocan.mymind.ui.moment

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioTrack
import android.os.Process
import java.util.Random
import java.util.concurrent.ConcurrentLinkedQueue
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.log10
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.tanh

/**
 * The moment's sounds, made as it plays from the prototype's Web Audio recipes (tones gliding in pitch, filtered noise,
 * bells), through a small room and a compressor. They play as the phone's own touch sounds do: not at all unless the
 * ringer is on, and at the phone's sound-effect volume (share-moment-sounds).
 */
internal class MomentSound(context: Context) {
    private val audio = context.getSystemService(AudioManager::class.java)
    private val queue = ConcurrentLinkedQueue<Voice>()
    private val random = Random()
    @Volatile private var running = false
    @Volatile private var stopping = false

    /** True when the ringer is on, so the moment may be heard. */
    val audible: Boolean get() = audio?.ringerMode == AudioManager.RINGER_MODE_NORMAL

    /** Opens the sound output; does nothing when the phone is silent or it is already open. */
    fun start() {
        if (running || !audible) return
        running = true
        stopping = false
        Thread(::loop, "share-moment-sound").apply { isDaemon = true }.start()
    }

    /** Plays a beat's sound now. */
    fun play(cue: Cue) {
        if (!running || stopping) return
        voicesOf(cue, RATE, random).forEach(queue::add)
    }

    /** Lets what is sounding ring out, then closes the output. */
    fun stop() { stopping = true }

    private fun loop() {
        Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
        val track = runCatching { openTrack() }.getOrNull()
        if (track == null) { running = false; return }
        val mix = Mixer(RATE)
        val active = ArrayList<Voice>()
        val block = FloatArray(BLOCK)
        var tail = 0
        try {
            track.play()
            while (true) {
                while (true) active += queue.poll() ?: break
                block.fill(0f)
                active.removeAll { it.render(block); it.done }
                mix.process(block)
                track.write(block, 0, BLOCK, AudioTrack.WRITE_BLOCKING)
                if (stopping && active.isEmpty()) { tail += BLOCK; if (tail > RATE * 2) break }
            }
        } finally {
            runCatching { track.stop() }
            track.release()
            running = false
        }
    }

    private fun openTrack(): AudioTrack {
        val format = AudioFormat.Builder().setEncoding(AudioFormat.ENCODING_PCM_FLOAT).setSampleRate(RATE).setChannelMask(AudioFormat.CHANNEL_OUT_MONO).build()
        val min = AudioTrack.getMinBufferSize(RATE, AudioFormat.CHANNEL_OUT_MONO, AudioFormat.ENCODING_PCM_FLOAT)
        return AudioTrack.Builder()
            .setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_ASSISTANCE_SONIFICATION).setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION).build())
            .setAudioFormat(format)
            .setTransferMode(AudioTrack.MODE_STREAM)
            .setPerformanceMode(AudioTrack.PERFORMANCE_MODE_LOW_LATENCY)
            .setBufferSizeInBytes(max(min, BLOCK * 4 * 4))
            .build()
    }

    companion object {
        const val RATE = 48_000
        private const val BLOCK = 256
    }
}

/** One sound being made: it adds itself into each block until it is [done]. */
internal abstract class Voice(delaySeconds: Float, lengthSeconds: Float, protected val rate: Int) {
    private var at = -(delaySeconds * rate).toInt()
    private val length = (lengthSeconds * rate).toInt()
    val done get() = at >= length

    fun render(block: FloatArray) {
        for (i in block.indices) {
            val t = at + i
            if (t in 0 until length) block[i] += sample(t)
        }
        at += block.size
    }

    protected abstract fun sample(t: Int): Float
}

/** An oscillator whose pitch glides from [f] to [f2] and whose level rises over [attack] then dies away by [dur]. */
internal class Tone(
    private val f: Float, private val f2: Float?, private val dur: Float, private val wave: Int, private val vol: Float,
    private val attack: Float = 0.008f, delay: Float = 0f, rate: Int,
) : Voice(delay, dur, rate) {
    private var phase = 0.0

    override fun sample(t: Int): Float {
        val s = t.toFloat() / rate
        val freq = if (f2 == null) f else f * (f2 / f).pow(s / dur)
        phase = (phase + freq / rate) % 1.0
        val p = phase.toFloat()
        val v = when (wave) { SAW -> 2f * p - 1f; TRIANGLE -> 2f * abs(2f * p - 1f) - 1f; else -> sin(2f * PI.toFloat() * p) }
        return v * envelope(s, attack, dur, vol)
    }

    companion object {
        const val SINE = 0
        const val TRIANGLE = 1
        const val SAW = 2
    }
}

/** White noise through a band-pass or high-pass filter that sweeps from [f1] to [f2]. */
internal class Noise(
    private val dur: Float, private val f1: Float, private val f2: Float, private val vol: Float, q: Float,
    private val attack: Float = 0.3f, private val highpass: Boolean = false, private val rise: Boolean = false,
    delay: Float = 0f, rate: Int, seed: Long,
) : Voice(delay, if (rise) dur + 0.02f else dur, rate) {
    private val random = Random(seed)
    // Web Audio reads a high-pass filter's Q in decibels and a band-pass filter's as a plain number.
    private val q = if (highpass) 10f.pow(q / 20f) else q
    private var b0 = 0f; private var b1 = 0f; private var b2 = 0f; private var a1 = 0f; private var a2 = 0f
    private var x1 = 0f; private var x2 = 0f; private var y1 = 0f; private var y2 = 0f

    override fun sample(t: Int): Float {
        val s = t.toFloat() / rate
        if (t % 16 == 0) tune(min(f1 * (f2 / f1).pow(min(1f, s / dur)), rate * 0.45f))
        val x = random.nextFloat() * 2f - 1f
        val y = b0 * x + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2
        x2 = x1; x1 = x; y2 = y1; y1 = y
        val g = if (rise) { if (s <= dur) 0.0008f * (vol / 0.0008f).pow(s / dur) else vol * (1f - (s - dur) / 0.02f).coerceAtLeast(0f) } else envelope(s, dur * attack, dur, vol)
        return y * g
    }

    private fun tune(freq: Float) {
        val w = 2f * PI.toFloat() * freq / rate
        val alpha = sin(w) / (2f * q)
        val c = cos(w)
        val a0 = 1f + alpha
        if (highpass) { b0 = (1f + c) / 2f / a0; b1 = -(1f + c) / a0; b2 = (1f + c) / 2f / a0 } else { b0 = alpha / a0; b1 = 0f; b2 = -alpha / a0 }
        a1 = -2f * c / a0; a2 = (1f - alpha) / a0
    }
}

/** Web Audio's envelope: a straight rise to [vol] over [attack], then an exponential fall to near silence by [dur]. */
private fun envelope(s: Float, attack: Float, dur: Float, vol: Float): Float = when {
    s < attack -> vol * s / attack
    s >= dur -> 0f
    else -> vol * (0.0008f / vol).pow((s - attack) / (dur - attack).coerceAtLeast(0.0001f))
}

/**
 * The prototype's bus: a room (wet 0.3) beside the dry sound, the output at 0.6, then a compressor at -14 dB, 4 to 1,
 * with Web Audio's own make-up gain.
 */
internal class Mixer(rate: Int) {
    private val combs = intArrayOf(1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617).map { Comb((it * rate / 44_100f).toInt()) }
    private val passes = intArrayOf(556, 441, 341, 225).map { AllPass((it * rate / 44_100f).toInt()) }
    private val attack = exp(-1f / (0.003f * rate))
    private val release = exp(-1f / (0.25f * rate))
    private var level = 0f

    fun process(block: FloatArray) {
        for (i in block.indices) {
            val dry = block[i]
            var wet = 0f
            val input = dry * 0.015f
            for (c in combs) wet += c.next(input)
            for (a in passes) wet = a.next(wet)
            var out = (dry + wet * 0.3f) * 0.6f
            val a = abs(out)
            level = if (a > level) attack * level + (1 - attack) * a else release * level + (1 - release) * a
            val db = 20f * log10(max(level, 1e-6f))
            val reduce = if (db > THRESHOLD) (db - THRESHOLD) * (1f - 1f / RATIO) else 0f
            out *= 10f.pow(-reduce / 20f) * MAKEUP
            block[i] = if (abs(out) < 0.9f) out else tanh(out)
        }
    }

    private class Comb(size: Int) {
        private val buf = FloatArray(size)
        private var at = 0
        private var store = 0f
        fun next(x: Float): Float {
            val y = buf[at]
            store = y * 0.8f + store * 0.2f
            buf[at] = x + store * 0.84f
            at = (at + 1) % buf.size
            return y
        }
    }

    private class AllPass(size: Int) {
        private val buf = FloatArray(size)
        private var at = 0
        fun next(x: Float): Float {
            val b = buf[at]
            buf[at] = x + b * 0.5f
            at = (at + 1) % buf.size
            return b - x
        }
    }

    private companion object {
        const val THRESHOLD = -14f
        const val RATIO = 4f
        const val MAKEUP = 2f
    }
}

/**
 * @param cue A beat of the moment.
 * @param rate Samples a second.
 * @param random Source of the shimmer's scatter and the noise.
 * @return The voices that make its sound, as the prototype's recipe for it.
 */
internal fun voicesOf(cue: Cue, rate: Int, random: Random): List<Voice> {
    val out = ArrayList<Voice>()
    fun tone(f: Float, dur: Float, wave: Int = Tone.SINE, vol: Float, f2: Float? = null, attack: Float = 0.008f, delay: Float = 0f) {
        out += Tone(f, f2, dur, wave, vol, attack, delay, rate)
    }
    fun noise(dur: Float, f1: Float, f2: Float, vol: Float, q: Float, attack: Float = 0.3f, highpass: Boolean = false, delay: Float = 0f) {
        out += Noise(dur, f1, f2, vol, q, attack, highpass, delay = delay, rate = rate, seed = random.nextLong())
    }
    fun bell(f: Float, at: Float, v: Float = 0.16f, d: Float = 1.4f) {
        tone(f, d, vol = v, delay = at); tone(f * 2.01f, d * 0.5f, vol = v * 0.35f, delay = at); tone(f * 3.98f, d * 0.25f, vol = v * 0.12f, delay = at)
    }
    when (cue) {
        Cue.Shimmer -> repeat(14) { tone(2500f + random.nextFloat() * 4000f, 0.5f, vol = 0.035f, delay = random.nextFloat() * 1.2f) }
        is Cue.Ting -> { bell(listOf(1318.5f, 1760f, 2349.3f)[cue.step.coerceIn(0, 2)], 0f, 0.14f, 1.2f); noise(0.25f, 6000f, 9000f, 0.08f, 3f, 0.05f) }
        is Cue.Meteor -> { val d = cue.seconds; noise(d, 5000f, 500f, 0.22f, 1.4f, 0.6f); tone(1400f, d, vol = 0.05f, f2 = 260f, attack = d * 0.6f) }
        is Cue.Riser -> {
            val d = cue.seconds
            noise(d, 250f, 6500f, 0.2f, 4f, 0.92f)
            tone(110f, d, Tone.SAW, 0.03f, 440f, d * 0.9f); tone(111.5f, d, Tone.SAW, 0.03f, 446f, d * 0.9f)
        }
        Cue.Suck -> out += Noise(0.25f, 800f, 5000f, 0.32f, 1f, highpass = true, rise = true, rate = rate, seed = random.nextLong())
        Cue.Swoop -> noise(0.38f, 2600f, 500f, 0.17f, 1.1f)
        Cue.Thud -> { tone(110f, 0.35f, vol = 0.55f, f2 = 38f, attack = 0.004f); noise(0.14f, 900f, 250f, 0.35f, 0.7f, 0.02f) }
        is Cue.Boom -> {
            val p = cue.power
            tone(72f, 1f + 0.6f * p, vol = 0.5f + 0.3f * p, f2 = 26f, attack = 0.004f)
            tone(150f, 0.25f, Tone.TRIANGLE, 0.22f * p, 55f, 0.002f)
            noise(0.7f + 0.5f * p, 2200f, 90f, 0.3f + 0.25f * p, 0.5f, 0.01f)
            noise(0.1f, 5000f, 2500f, 0.5f, 0.6f, 0.02f, highpass = true)
            noise(0.2f, 1800f, 900f, 0.25f, 0.8f, 0.02f)
        }
        is Cue.Chime -> {
            val legendary = cue.rarity == Rarity.Legendary
            val notes = when (cue.rarity) {
                Rarity.Legendary -> listOf(523.3f, 659.3f, 784f, 1046.5f, 1318.5f, 1568f)
                Rarity.Rare -> listOf(587.3f, 880f, 1174.7f)
                Rarity.Common -> listOf(659.3f, 987.8f)
            }
            notes.forEachIndexed { i, f -> bell(f, i * 0.07f, if (legendary) 0.14f else 0.16f, if (legendary) 2.4f else 1.3f) }
            if (legendary) { tone(130.8f, 2f, vol = 0.2f); tone(196f, 2f, vol = 0.08f) }
        }
        is Cue.Star -> tone(1300f + cue.index * 200f, 0.07f, Tone.TRIANGLE, 0.14f)
        Cue.Count -> tone(1800f, 0.07f, Tone.TRIANGLE, 0.14f)
        Cue.Pop -> { tone(520f, 0.14f, vol = 0.3f, f2 = 1200f); noise(0.08f, 2500f, 4000f, 0.12f, 2f) }
        Cue.Whoosh -> noise(0.5f, 300f, 2600f, 0.2f, 0.9f)
    }
    return out
}


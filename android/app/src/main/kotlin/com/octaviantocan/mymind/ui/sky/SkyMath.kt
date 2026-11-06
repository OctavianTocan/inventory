package com.octaviantocan.mymind.ui.sky

import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.tan

/**
 * The "Your sky" prototype's own numbers, in its units: a phone 844 tall looked at by a 30° camera that pitches 74° up
 * to a sky 2000 away. One unit is a CSS pixel of the prototype's 390 by 844 phone; [SkyProjector] maps them to pixels.
 */
internal object Proto {
    const val H = 844f
    val TAN = tan(Math.toRadians(15.0)).toFloat()

    /** The camera's focal length, and how far it stands from the page: the page fills the screen exactly. */
    val F = (H / 2f) / TAN
    val PITCH = Math.toRadians(74.0).toFloat()
    const val RC = 2000f
    const val SX = 160f
    const val SY = 196f
    const val CY = 40f
    const val SKY_STAR_Y = 190f
}

/** The prototype's easing curves. */
internal object Ease {
    val io: (Float) -> Float = { p -> if (p < 0.5f) 4f * p * p * p else 1f - (-2f * p + 2f).pow(3) / 2f }
    val out: (Float) -> Float = { p -> 1f - (1f - p).pow(3) }
    val lin: (Float) -> Float = { p -> p }
    val sine: (Float) -> Float = { p -> 0.5f - 0.5f * cos(p * PI.toFloat()) }
    val back: (Float) -> Float = { p -> 1f + 2.4f * (p - 1f).pow(3) + 1.4f * (p - 1f).pow(2) }
}

/** three.js's `smoothstep(x, min, max)`. */
internal fun sstep(x: Float, lo: Float, hi: Float): Float {
    if (x <= lo) return 0f
    if (x >= hi) return 1f
    val t = (x - lo) / (hi - lo)
    return t * t * (3f - 2f * t)
}

internal fun mix(a: Float, b: Float, t: Float): Float = a + (b - a) * t

/** A point or a direction in the prototype's world. */
internal class V3(var x: Float = 0f, var y: Float = 0f, var z: Float = 0f) {
    fun set(x: Float, y: Float, z: Float): V3 = apply { this.x = x; this.y = y; this.z = z }
    fun copy(): V3 = V3(x, y, z)
}

/** Where a point lands on the screen: [x], [y] in pixels, and [s] pixels per unit at its depth. */
internal class Projected(var x: Float = 0f, var y: Float = 0f, var s: Float = 0f)

/**
 * The prototype's camera for one screen: at look 0 it faces the page, which fills the screen; at look 1 it has pitched
 * up to face the sky, which hangs on a rig at the same place and angle. Everything in the sky sits on the rig's "slide",
 * which moves in on a star and pans to it.
 *
 * @param widthPx The screen's width in pixels.
 * @param heightPx The screen's height in pixels.
 */
internal class SkyProjector(val widthPx: Float, val heightPx: Float) {
    /** Pixels per unit on the page. */
    val unit = heightPx / Proto.H

    /** The screen's width in units. */
    val w = widthPx / unit

    private var camX = 0f
    private var camY = 0f
    private var camZ = Proto.F
    private var pitch = 0f
    private var c = 1f
    private var s = 0f

    /** Where the slide sits on the rig: panned to a star and moved in on it. */
    val slide = V3(0f, 0f, -Proto.RC)

    /**
     * Places the camera for this frame.
     *
     * @param look How far it has looked up, from 0 to 1.
     * @param punch A camera shake, from 0 to 1.
     * @param jitterX A random number from -0.5 to 0.5 for the shake.
     * @param jitterY Another.
     */
    fun place(look: Float, punch: Float = 0f, jitterX: Float = 0f, jitterY: Float = 0f) {
        val shake = punch * punch * 4f
        camX = jitterX * shake
        camY = -look * 140f + jitterY * shake
        camZ = Proto.F + look * 240f - sin(punch * PI.toFloat()) * 22f
        pitch = look * Proto.PITCH
        c = cos(pitch)
        s = sin(pitch)
    }

    /** Projects a point of the world onto [out]; false when it is behind the camera. */
    fun project(x: Float, y: Float, z: Float, out: Projected): Boolean {
        val dx = x - camX
        val dy = y - camY
        val dz = z - camZ
        val vy = c * dy + s * dz
        val vz = -s * dy + c * dz
        if (vz > -1f) return false
        val k = Proto.F / -vz
        out.x = (w / 2f + dx * k) * unit
        out.y = (Proto.H / 2f - vy * k) * unit
        out.s = k * unit
        return true
    }

    /** Projects a point on the slide, [x] and [y] across it and [z] out of it, onto [out]. */
    fun projectSlide(x: Float, y: Float, z: Float, out: Projected): Boolean {
        val lx = slide.x + x
        val ly = slide.y + y
        val lz = slide.z + z
        val pc = cos(Proto.PITCH)
        val ps = sin(Proto.PITCH)
        return project(lx, -140f + ly * pc - lz * ps, Proto.F + 240f + ly * ps + lz * pc, out)
    }

    /** Projects a point of the page, in its units with y up from the screen's middle, onto [out]. */
    fun projectPage(x: Float, y: Float, out: Projected): Boolean = project(x, y, 0f, out)
}

/**
 * The prototype's clock: tweens and waits run in sky time, which slows to a crawl for a moment of hit-stop as a star
 * lands. [advance] is called once a frame.
 */
internal class Timeline {
    private class Task(val t0: Float, val dur: Float, val ease: (Float) -> Float, val fn: (Float) -> Unit, val done: CancellableContinuation<Unit>)

    private val tasks = ArrayList<Task>()

    /** Seconds of sky time so far. */
    var now = 0f
        private set

    /** Seconds of slow motion left. */
    var hitStop = 0f

    /** True while a tween or wait is running. */
    val running: Boolean get() = tasks.isNotEmpty()

    /** Counts tweens and waits started, so a sleeping frame loop wakes for one. */
    val started = androidx.compose.runtime.mutableIntStateOf(0)

    /** Runs [fn] with eased progress from 0 to 1 over [dur] seconds of sky time, and returns once it reaches 1. */
    suspend fun tween(dur: Float, ease: (Float) -> Float = Ease.io, fn: (Float) -> Unit) {
        if (dur <= 0f) {
            fn(1f)
            return
        }
        started.intValue++
        suspendCancellableCoroutine { done ->
            val task = Task(now, dur, ease, fn, done)
            tasks += task
            done.invokeOnCancellation { tasks.remove(task) }
        }
    }

    /** Waits [dur] seconds of sky time. */
    suspend fun wait(dur: Float) = tween(dur, Ease.lin) {}

    /**
     * Moves sky time on by [dt] real seconds and steps every tween.
     *
     * @return The sky seconds that passed.
     */
    fun advance(dt: Float): Float {
        var step = dt
        if (hitStop > 0f) {
            hitStop -= dt
            step *= 0.06f
        }
        now += step
        if (tasks.isEmpty()) return step
        for (task in tasks.toTypedArray()) {
            val p = ((now - task.t0) / task.dur).coerceAtMost(1f)
            task.fn(task.ease(p))
            if (p >= 1f) {
                tasks.remove(task)
                if (task.done.isActive) task.done.resume(Unit)
            }
        }
        return step
    }
}

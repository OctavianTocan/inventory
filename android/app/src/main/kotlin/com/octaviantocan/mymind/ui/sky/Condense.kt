package com.octaviantocan.mymind.ui.sky

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.VectorConverter
import androidx.compose.runtime.Stable
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import kotlin.math.PI
import kotlin.math.hypot
import kotlin.math.sin
import kotlin.random.Random

/** How long the camera takes to move in on a star, or back out. */
internal const val FOCUS_MS = 950

/** How long the camera takes to travel along a line to the next star. */
internal const val GLIDE_MS = 950

/** How long the stars take to gather into a find. */
internal const val CONDENSE_MS = 2000

/** How long a find takes to go back into starlight. */
internal const val RELEASE_MS = 300

/**
 * Where the sky is looked at from: at rest it shows the whole sky; moved in, [look] sits at [anchor], [zoom] times
 * nearer. [lift] goes from 0 at rest to 1 moved in, so every step between is one smooth move.
 */
@Stable
internal class SkyCamera {
    val look = Animatable(Offset.Zero, Offset.VectorConverter)
    val zoom = Animatable(1f)
    val lift = Animatable(0f)

    /** Where a star in focus sits on the screen; set as the sky is laid out. */
    var anchor = Offset.Zero

    /** How much nearer the sky is drawn now. */
    fun scale(): Float = 1f + lift.value * (zoom.value - 1f)

    /** How far the sky is moved now, after [scale]. */
    fun shift(): Offset = (anchor - look.value * zoom.value) * lift.value

    /** Where a point of the sky shows on the screen now. */
    fun map(point: Offset): Offset = point * scale() + shift()
}

/**
 * The stars that fly together from all over the screen and become a find, variant 13 of the prototype's Skyrim views:
 * each starts somewhere on the screen and lands somewhere on [rect], thicker in the middle, the same each time for the
 * same [seed].
 */
internal class Condense(seed: String, val rect: Rect, screen: Size) {
    private val count = PARTICLES
    private val start = Array(count) { Offset.Zero }
    private val end = Array(count) { Offset.Zero }
    private val delay = FloatArray(count)
    private val radius = FloatArray(count)
    private val phase = FloatArray(count)
    private val tint = Array(count) { Color.White }

    init {
        val random = Random(seed.hashCode())
        var made = 0
        while (made < count) {
            val x = random.nextFloat() - 0.5f
            val y = random.nextFloat() - 0.5f
            val mask = 1f - smoothStep(0.22f, 0.62f, hypot(x, y * 0.9f) + (random.nextFloat() - 0.5f) * 0.12f)
            if (random.nextFloat() > 0.2f + 0.8f * mask) continue
            start[made] = Offset(random.nextFloat() * screen.width, random.nextFloat() * screen.height)
            end[made] = Offset(rect.center.x + x * rect.width, rect.center.y + y * rect.height)
            delay[made] = random.nextFloat() * 0.55f
            radius[made] = 0.6f + random.nextFloat() * random.nextFloat() * 1.6f
            phase[made] = random.nextFloat() * 6.28f
            tint[made] = random.nextFloat().let { if (it < 0.25f) Color(0xFFC8DAFF) else if (it < 0.8f) Color(0xFFFFF6E4) else Color(0xFFFFD49E) }
            made++
        }
    }

    /**
     * Draws the stars [progress] of the way from where they start to the find, twinkling at [time] seconds, and gone
     * once the find has formed.
     */
    fun DrawScope.drawGathering(progress: Float, time: Float) {
        if (progress <= 0f || progress >= 1f) return
        val fade = 1f - smoothStep(0.8f, 1f, progress)
        for (index in 0 until count) {
            val e = smoothStep(0f, 1f, (progress * 1.55f - delay[index]) / 0.9f)
            val from = start[index]
            val to = end[index]
            val along = to - from
            // Each star swings a little to the side on its way, so they swirl in rather than fly straight.
            val swing = Offset(-along.y, along.x) * (0.18f * sin(e * PI.toFloat()))
            val at = from + along * e + swing
            val twinkle = 0.75f + 0.25f * sin(time * 2.2f + phase[index] * 7f)
            val r = radius[index] * density * (1f + (1f - e) * 0.8f)
            drawCircle(tint[index], r, at, alpha = (fade * (0.35f + 0.65f * e) * twinkle).coerceIn(0f, 1f))
        }
    }
}

private const val PARTICLES = 380

/** A bright spark running along a line from [from] to [to], [progress] of the way, brightest halfway. */
internal fun DrawScope.drawComet(from: Offset, to: Offset, progress: Float) {
    if (progress <= 0f || progress >= 1f) return
    val k = sin(progress * PI.toFloat())
    val at = from + (to - from) * progress
    val glow = 16f * density * (1f + 0.6f * k)
    drawCircle(Brush.radialGradient(listOf(Color.White, Color(0x99FFD58A), Color.Transparent), at, glow), glow, at, alpha = 0.95f * k)
}

/** 0 below [from], 1 above [to], and smooth between. */
internal fun smoothStep(from: Float, to: Float, value: Float): Float {
    val t = ((value - from) / (to - from)).coerceIn(0f, 1f)
    return t * t * (3f - 2f * t)
}

package com.octaviantocan.mymind.ui.sky

import android.graphics.RenderEffect
import android.graphics.RuntimeShader
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asComposeRenderEffect
import androidx.compose.ui.graphics.graphicsLayer
import com.octaviantocan.mymind.data.Card
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.time.Duration
import java.time.Instant
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.roundToInt
import kotlin.random.Random

/**
 * A Back today find as a flickering blue hologram, only a memory of it left (the prototype's look 25, Hologram), and
 * its restore: lifted, cleared from where it was tapped outward with a burst of cyan glints, a shake, then a shine.
 *
 * @param stage How far gone it is, from 0 to 3, as the prototype's wear stages.
 */
@Stable
internal class Hologram(val stage: Float) {
    /** Seconds since it started flickering. */
    var time by mutableFloatStateOf(0f)

    /** How far the restore has cleared it, from 0 to 1. */
    var restore by mutableFloatStateOf(0f)

    /** The shine that crosses it once restored, from 0 to 1, 0 when none. */
    var shine by mutableFloatStateOf(0f)

    /** Where the restore starts, across and up the find from 0 to 1. */
    var origin by mutableStateOf(Offset(0.5f, 0.55f))

    /** How far it is lifted for its restore, 1 at rest. */
    val lift = Animatable(1f)

    /** True from the restore's start until its shine has crossed. */
    var restoring by mutableStateOf(false)
        private set

    /** The glints thrown off as it clears, each 8 numbers: x, y, vx, vy, delay, size, unused, unused (dp, y up). */
    internal var glints = FloatArray(0)

    /** The flicker time the glints started at, or -1 before any. */
    internal var glintsAt by mutableFloatStateOf(-1f)

    /**
     * Restores it from [at], across and down the find from 0 to 1, as the prototype's restore runs: returns once it is
     * clear, when the find should open, while the shine still crosses it in [scope].
     *
     * @param width The find's width in dp.
     * @param height The find's height in dp.
     * @param scope Where the lift and the shine run on after it returns.
     * @param onShake Called as it clears, for the camera's shake.
     */
    suspend fun restoreFrom(at: Offset, width: Float, height: Float, scope: CoroutineScope, onShake: () -> Unit) {
        if (restoring) return
        restoring = true
        origin = Offset(at.x, 1f - at.y)
        val dur = 0.8f + 0.12f * max(1f, stage)
        scope.launch { lift.animateTo(1.035f, spring(dampingRatio = 0.6f, stiffness = Spring.StiffnessMedium)) }
        delay(140)
        burst(width, height, dur)
        run(dur, Ease.sine) { restore = it }
        onShake()
        scope.launch {
            run(0.7f, Ease.io) { shine = it }
            shine = 0f
            restoring = false
        }
        scope.launch { lift.animateTo(1f, spring(dampingRatio = 0.6f, stiffness = Spring.StiffnessMedium)) }
    }

    // Fills the glints the way the prototype's burst does for Hologram: 110 cyan glints, more the further gone it is.
    private fun burst(width: Float, height: Float, dur: Float) {
        val r = Random(System.nanoTime())
        val k = 0.55f + 0.25f * max(1f, stage)
        val n = (GLINTS * k).roundToInt()
        val md = max(width, height)
        val out = FloatArray(n * 8)
        for (i in 0 until n) {
            val u = r.nextFloat()
            val w = r.nextFloat()
            val dx = (u - origin.x) * width
            val dy = (w - origin.y) * height
            val dd = hypot(dx, dy).takeIf { it > 0f } ?: 1f
            val speed = 50f + r.nextFloat() * 150f
            out[i * 8] = u * width
            out[i * 8 + 1] = w * height
            out[i * 8 + 2] = dx / dd * speed
            out[i * 8 + 3] = dy / dd * speed
            out[i * 8 + 4] = dur * max(0f, (dd / md + 0.12f) / 1.7f) + r.nextFloat() * 0.05f
            out[i * 8 + 5] = 1.6f + r.nextFloat() * 2.2f
        }
        glints = out
        glintsAt = time
    }

    private suspend fun run(dur: Float, ease: (Float) -> Float, fn: (Float) -> Unit) {
        val start = withFrameNanos { it }
        while (true) {
            val now = withFrameNanos { it }
            val p = ((now - start) / 1e9f / dur).coerceIn(0f, 1f)
            fn(ease(p))
            if (p >= 1f) break
        }
    }

    companion object {
        private const val GLINTS = 110f
    }
}

/** How far gone a Back today find shows: by months since it was last saved, as the prototype's stages, never under 2. */
internal fun hologramStage(card: Card, now: Instant = Instant.now()): Float {
    val at = runCatching { Instant.parse(card.bumped) }.getOrNull() ?: runCatching { Instant.parse(card.created) }.getOrNull()
    val months = at?.let { Duration.between(it, now).toDays() / 30.4f } ?: 20f
    return max(2f, stageOf(months))
}

// The prototype's wear stages: 3, 9 and 18 months.
private fun stageOf(months: Float): Float {
    if (months >= 20f) return 3f
    for (i in 1 until AGE.size) {
        if (months <= AGE[i].first) {
            val (a, sa) = AGE[i - 1]
            val (b, sb) = AGE[i]
            return sa + (sb - sa) * (months - a) / (b - a)
        }
    }
    return 3f
}

private val AGE = listOf(0f to 0f, 2f to 0f, 4f to 1f, 8f to 1f, 11f to 2f, 16f to 2f, 20f to 3f)

/** Keeps [hologram] flickering while it shows. */
@Composable
internal fun HologramClock(hologram: Hologram) {
    LaunchedEffect(hologram) {
        var last = 0L
        while (isActive) {
            withFrameNanos { now ->
                if (last != 0L) hologram.time += (now - last) / 1e9f
                last = now
            }
        }
    }
}

/**
 * Draws what this modifies as [hologram]; [shader] is the hologram's, or null before Android 13, where it is tinted
 * blue instead.
 */
internal fun Modifier.hologram(hologram: Hologram, shader: RuntimeShader?): Modifier =
    if (shader != null && SkyShaders.supported) {
        graphicsLayer {
            val w = size.width
            val h = size.height
            if (w > 0f && h > 0f) {
                shader.setFloatUniform("uPx", w, h)
                shader.setFloatUniform("uSize", w / density, h / density)
                shader.setFloatUniform("uOrigin", hologram.origin.x, hologram.origin.y)
                shader.setFloatUniform("uTime", hologram.time)
                shader.setFloatUniform("uStage", hologram.stage)
                shader.setFloatUniform("uRestore", hologram.restore)
                shader.setFloatUniform("uShine", hologram.shine)
                renderEffect = RenderEffect.createRuntimeShaderEffect(shader, "content").asComposeRenderEffect()
            }
        }
    } else {
        drawWithContent {
            drawContent()
            drawRect(Color(0xFF1E9CFF), alpha = 0.6f * (1f - hologram.restore), blendMode = BlendMode.Color)
        }
    }

/** The cyan glints [hologram] throws off as it clears, drawn over the find and free to leave it. */
@Composable
internal fun HologramGlints(hologram: Hologram, modifier: Modifier) {
    Canvas(modifier) {
        val start = hologram.glintsAt
        if (start < 0f) return@Canvas
        val t = hologram.time - start
        val g = hologram.glints
        if (t > 3f || g.isEmpty()) return@Canvas
        val d = density
        for (i in 0 until g.size / 8) {
            val a = t - g[i * 8 + 4]
            if (a < 0f) continue
            val alpha = minOf(1f, a * 14f) * (1f - sstep(a, 0.9f * 0.45f, 0.9f)) * 0.9f
            if (alpha <= 0.004f) continue
            val x = g[i * 8] + g[i * 8 + 2] * a * (1f - a * 0.25f)
            val y = g[i * 8 + 1] + g[i * 8 + 3] * a * (1f - a * 0.25f) + 0.5f * 50f * a * a
            drawCircle(GLINT, radius = g[i * 8 + 5] * 0.5f * d, center = Offset(x * d, size.height - y * d), alpha = alpha)
        }
    }
}

private val GLINT = Color(0.4f, 0.85f, 1f)

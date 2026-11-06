package com.octaviantocan.mymind.ui.sky

import android.graphics.Bitmap
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlin.math.exp
import kotlin.math.floor
import kotlin.math.pow
import kotlin.math.sin

// Small, since the nebula is soft: drawn stretched over the screen it shows no pixels, and it paints in a blink.
private const val NEBULA_WIDTH = 180
private const val NEBULA_HEIGHT = 390

/** The sky's nebula, painted once a process off the main thread; stars are drawn over it, sharp, at the screen's size. */
internal object Nebula {
    @Volatile
    private var painted: ImageBitmap? = null

    /** The nebula, painted on first use. */
    suspend fun picture(): ImageBitmap = painted ?: withContext(Dispatchers.Default) { paint() }.also { painted = it }

    private fun paint(): ImageBitmap {
        val w = NEBULA_WIDTH
        val h = NEBULA_HEIGHT
        val aspect = h.toDouble() / w
        val pixels = IntArray(w * h)
        for (y in 0 until h) for (x in 0 until w) {
            val u = (x + 0.5) / w
            val v = (y + 0.5) / h
            pixels[y * w + x] = colorAt(u, v, aspect)
        }
        return Bitmap.createBitmap(pixels, w, h, Bitmap.Config.ARGB_8888).asImageBitmap()
    }

    // A band of cloud from the top right to the bottom left, warped by noise into wisps with dark dust lanes across it.
    private fun colorAt(u: Double, v: Double, aspect: Double): Int {
        val px = u * 3.0
        val py = v * 3.0 * aspect
        val across = (u - 0.5) * -0.62 + (v - 0.52) * aspect * -0.39 + (fbm(px * 0.5, py * 0.5) - 0.5) * 0.35
        val band = exp(-across * across / 0.035)
        val wide = exp(-across * across / 0.18)
        val qx = fbm(px, py)
        val qy = fbm(px + 5.2, py + 1.3)
        val rx = fbm(px * 1.5 + qx * 2.4, py * 1.5 + qy * 2.4)
        val ry = fbm(px * 1.5 + qx * 2.4 + 3.3, py * 1.5 + qy * 2.4 + 2.8)
        val cloud = fbm(px * 2 + rx * 2.6, py * 2 + ry * 2.6)
        val fine = fbm(px * 7 + rx * 3, py * 7 + ry * 3, 4)
        val violet = fbm(px * 2.6 + rx * 1.7 + 11, py * 2.6 + ry * 1.7 + 7)
        val dust = smooth(0.46, 0.72, fbm(px * 3.4 + rx * 2.5 + 2, py * 3.4 + ry * 2.5 + 5)) * smooth(0.0, 0.5, band)
        val blue = cloud.pow(2.2) * wide * 1.1
        val teal = cloud.pow(3.0) * band * (0.5 + fine) * 1.2
        val purple = violet.pow(3.5) * band * 2.2
        val warm = cloud.pow(4.0) * band * (0.4 + 1.1 * fine) * 0.9
        val keep = 0.8 * (1 - dust * 0.85)
        val r = 0.027 + 0.016 * v + keep * (0.10 * blue + 0.14 * teal + 0.42 * purple + 0.70 * warm)
        val g = 0.043 + 0.028 * v + keep * (0.20 * blue + 0.26 * teal + 0.20 * purple + 0.58 * warm)
        val b = 0.094 + 0.055 * v + keep * (0.40 * blue + 0.36 * teal + 0.40 * purple + 0.52 * warm)
        return (0xFF shl 24) or (channel(r) shl 16) or (channel(g) shl 8) or channel(b)
    }

    private fun channel(value: Double): Int = (value.coerceIn(0.0, 1.0) * 255).toInt()

    private fun smooth(from: Double, to: Double, value: Double): Double {
        val t = ((value - from) / (to - from)).coerceIn(0.0, 1.0)
        return t * t * (3 - 2 * t)
    }

    private fun hash(x: Double, y: Double): Double {
        val s = sin(x * 127.1 + y * 311.7) * 43758.5453
        return s - floor(s)
    }

    private fun noise(x: Double, y: Double): Double {
        val ix = floor(x)
        val iy = floor(y)
        var fx = x - ix
        var fy = y - iy
        fx = fx * fx * (3 - 2 * fx)
        fy = fy * fy * (3 - 2 * fy)
        val a = hash(ix, iy)
        val b = hash(ix + 1, iy)
        val c = hash(ix, iy + 1)
        val d = hash(ix + 1, iy + 1)
        return a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy
    }

    private fun fbm(x: Double, y: Double, octaves: Int = 5): Double {
        var sum = 0.0
        var amplitude = 0.5
        var px = x
        var py = y
        repeat(octaves) {
            sum += amplitude * noise(px, py)
            px = px * 2.02 + 3.1
            py = py * 2.02 + 1.7
            amplitude *= 0.5
        }
        return sum
    }
}

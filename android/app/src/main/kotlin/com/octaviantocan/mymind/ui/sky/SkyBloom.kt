package com.octaviantocan.mymind.ui.sky

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import kotlin.math.exp
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow

/**
 * The prototype's bloom pass (three.js UnrealBloomPass at threshold 0.5, radius 0.85), measured from its own frames.
 *
 * The pass keeps what is brighter than half at half the prototype's pixels (one CSS pixel a texel, as it never draws
 * finer than 2 device pixels a CSS pixel), blurs it into five levels about 1.4, 5.1, 14.5, 38.5 and 97 CSS pixels wide
 * and adds them back at weights 0.32, 0.46, 0.6, 0.74 and 0.88 times its strength. The sum it adds also carries those
 * weights in its alpha, and the added blend multiplies by that alpha, so what shows is the blur times 3 times the
 * strength squared: about 3.3 times the bright light, which is why the lit constellation sits in a warm cloud.
 *
 * Here the two narrowest levels and the third go with each light (a line's shader, a star's sprite), and the two widest,
 * where many faint lights overlap, are added up for the whole sky in linear light on a coarse grid ([WideBloom]).
 */
internal object SkyBloom {
    /** The levels' weights at radius 0.85. */
    val WEIGHTS = floatArrayOf(0.32f, 0.46f, 0.6f, 0.74f, 0.88f)

    /** Each level's width (sigma) in CSS pixels. */
    val SIGMAS = floatArrayOf(1.4f, 5.1f, 14.5f, 38.5f, 97f)

    /** The bloom's strength at [look], plus a flare's [kick], as the prototype sets it each frame. */
    fun strength(look: Float, kick: Float): Float = mix(0f, 1.05f, sstep(look, 0.4f, 1f)) + kick

    /** How much of the blurred bright light shows at [strength]: strength times the alpha it carries. */
    fun gain(strength: Float): Float = 3f * strength * strength

    /** [gain] at full strength, the gain the star sprite is drawn for. */
    val FULL_GAIN = gain(1.05f)

    /**
     * The bright light (red, in CSS pixels squared) of a lit star whose glow is [size] CSS pixels across at [opacity]:
     * only its middle brighter than half counts, so a faint or small star gives almost none.
     */
    fun starEnergy(opacity: Float, size: Float): Float {
        if (opacity <= 0.14f || size <= 1f) return 0f
        val bright = ((opacity - 0.14f) / 0.86f).coerceAtMost(1.6f).pow(1.6f)
        return STAR_ENERGY * bright * (size / 72f).pow(2.12f) * (1f - 0.45f * exp(-size / 15f))
    }

    /** The bright light of a lit star 72 CSS pixels across at full opacity. */
    const val STAR_ENERGY = 206f

    /**
     * The bright light (red, CSS pixels squared) of a field star [radius] CSS pixels to its edge at brightness [k], of
     * linear colour [red] in its red and [luminance] in all: only the few big, bright ones still pass half once
     * averaged over a CSS pixel, but there are hundreds of them, and in the prototype they give half the sky's bloom.
     */
    fun fieldEnergy(radius: Float, k: Float, red: Float, luminance: Float): Float {
        val over = k * luminance - (0.45f + 1.8f / (radius * radius))
        return if (over <= 0f) 0f else 0.165f * radius * radius * red * over / luminance
    }

    /**
     * The bright light (red, CSS pixels squared) of one of the dome's finest stars at brightness [b]: a cell about a
     * CSS pixel across, which passes half only where it covers most of a CSS pixel.
     */
    fun microEnergy(b: Float): Float = 0.7f * b * ((b - 0.62f) / 0.65f).coerceIn(0f, 1f)

    /**
     * The opacity at which a line [k] CSS pixels to its unit starts to bloom: a thin line is averaged with the dark
     * beside it before the pass looks at it, so it needs more.
     */
    fun lineThreshold(k: Float): Float = 0.42f / thinPeak(k)

    // How much of a line's peak is left after averaging over one CSS pixel, measured for the prototype's line.
    private fun thinPeak(k: Float): Float {
        if (k <= PEAK_K[0]) return max(0.05f, PEAK[0] * k / PEAK_K[0])
        for (i in 1 until PEAK_K.size) {
            if (k <= PEAK_K[i]) return mix(PEAK[i - 1], PEAK[i], (k - PEAK_K[i - 1]) / (PEAK_K[i] - PEAK_K[i - 1]))
        }
        return min(1f, PEAK.last() + (k - PEAK_K.last()) * 0.004f)
    }

    private val PEAK_K = floatArrayOf(0.25f, 0.3f, 0.4f, 0.5f, 0.6f, 0.788f, 1f, 1.5f, 2f, 3f)
    private val PEAK = floatArrayOf(0.521f, 0.586f, 0.693f, 0.769f, 0.822f, 0.882f, 0.92f, 0.958f, 0.974f, 0.986f)

    /**
     * The bright light (red, CSS pixels squared per CSS pixel of length) of a lit line at [opacity] whose unit is [k]
     * CSS pixels, starting to bloom at [threshold] ([lineThreshold]).
     */
    fun lineEnergy(opacity: Float, k: Float, threshold: Float): Float {
        val o = if (opacity >= 1f) opacity else 0.42f + (opacity - threshold) * 0.58f / max(1f - threshold, 0.1f)
        val g = if (o >= 1f) 2.69f + 3.94f * (o - 1f) else if (o <= 0.42f) 0f else 2.69f * ((o - 0.42f) / 0.58f).pow(0.8f)
        return g * k
    }
}

/**
 * The bloom's widest levels for the whole sky, added up in linear light on a grid of [CELL] CSS pixels and drawn
 * stretched over the sky: far apart, faint lights add as they do in the prototype instead of brightening each other.
 * Each light is shared out to the four grid points round it, and the grid is blurred once a frame by each level.
 */
internal class WideBloom {
    private var cols = 0
    private var rows = 0
    private var cell = 1f
    private var wide = Array(3) { FloatArray(0) }
    private var near = Array(3) { FloatArray(0) }
    private var out = Array(3) { FloatArray(0) }
    private var spare = FloatArray(0)
    private var pixels = IntArray(0)
    private var bitmap: Bitmap? = null
    /** Pixels a CSS pixel, as of the last [begin]. */
    val unit: Float get() = cell / CELL
    private var anyWide = false
    private var anyNear = false
    private val rect = RectF()
    private val kernels = arrayOfNulls<FloatArray>(SkyBloom.SIGMAS.size)

    /** Clears the grid for a sky [width] by [height] pixels at [unit] pixels a CSS pixel. */
    fun begin(width: Float, height: Float, unit: Float) {
        val c = CELL * unit
        val nc = (width / c).toInt() + 3
        val nr = (height / c).toInt() + 3
        if (nc != cols || nr != rows || c != cell) {
            cols = nc
            rows = nr
            cell = c
            wide = Array(3) { FloatArray(nc * nr) }
            near = Array(3) { FloatArray(nc * nr) }
            out = Array(3) { FloatArray(nc * nr) }
            spare = FloatArray(nc * nr)
            pixels = IntArray(nc * nr)
            bitmap?.recycle()
            bitmap = Bitmap.createBitmap(nc, nr, Bitmap.Config.ARGB_8888)
        } else {
            if (anyWide) for (a in wide) a.fill(0f)
            if (anyNear) for (a in near) a.fill(0f)
        }
        anyWide = false
        anyNear = false
    }

    /**
     * Adds a point of bright light [energy] (red, CSS pixels squared) at [x], [y] pixels, coloured [g] and [b] of its
     * red, already times the bloom's gain, to the two widest levels.
     */
    fun point(x: Float, y: Float, energy: Float, g: Float, b: Float) {
        if (energy <= 0.05f) return
        if (share(wide, x, y, energy, g, b)) anyWide = true
    }

    /**
     * Adds a field star's bright light [energy], as [point] does, to the third level as well: field stars are too many
     * to draw it for each, and only add up to a haze in it.
     */
    fun field(x: Float, y: Float, energy: Float, g: Float, b: Float) {
        if (energy <= 0.02f) return
        if (share(wide, x, y, energy, g, b)) anyWide = true
        if (share(near, x, y, energy, g, b)) anyNear = true
    }

    /** Adds a line from [x0], [y0] to [x1], [y1] pixels giving [energy] per CSS pixel of length, as points along it. */
    fun line(x0: Float, y0: Float, x1: Float, y1: Float, energy: Float, g: Float, b: Float) {
        if (energy <= 0.002f) return
        val u = cell / CELL
        val length = kotlin.math.hypot(x1 - x0, y1 - y0) / u
        val n = max(1, (length * 2f / CELL).toInt() + 1)
        val each = energy * length / n
        for (i in 0 until n) {
            val t = (i + 0.5f) / n
            if (share(wide, x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, each, g, b)) anyWide = true
        }
    }

    // Shares a light out to the four grid points round it; grid point i is the middle of cell i - 1.
    private fun share(grid: Array<FloatArray>, x: Float, y: Float, energy: Float, g: Float, b: Float): Boolean {
        val fx = x / cell + 0.5f
        val fy = y / cell + 0.5f
        val i = kotlin.math.floor(fx).toInt()
        val j = kotlin.math.floor(fy).toInt()
        if (i < 0 || j < 0 || i + 1 >= cols || j + 1 >= rows) return false
        val tx = fx - i
        val ty = fy - j
        val k = j * cols + i
        val w00 = (1f - tx) * (1f - ty)
        val w10 = tx * (1f - ty)
        val w01 = (1f - tx) * ty
        val w11 = tx * ty
        for (ch in 0 until 3) {
            val e = if (ch == 0) energy else if (ch == 1) energy * g else energy * b
            val a = grid[ch]
            a[k] += e * w00; a[k + 1] += e * w10; a[k + cols] += e * w01; a[k + cols + 1] += e * w11
        }
        return true
    }

    /** Draws what was added over a sky [width] by [height] pixels with [paint]. */
    fun draw(canvas: Canvas, paint: Paint, width: Float, height: Float) {
        val bitmap = bitmap ?: return
        if (!anyWide && !anyNear) return
        val u = cell / CELL
        for (ch in 0 until 3) {
            val o = out[ch]
            o.fill(0f)
            if (anyWide) {
                blur(wide[ch], o, 3)
                blur(wide[ch], o, 4)
            }
            if (anyNear) blur(near[ch], o, 2)
        }
        for (k in pixels.indices) {
            val r = Srgb.enc(out[0][k])
            val g = Srgb.enc(out[1][k])
            val b = Srgb.enc(out[2][k])
            val top = max(r, max(g, b))
            pixels[k] = if (top <= 0.002f) 0 else argb(top, (r / top * 255f).toInt(), (g / top * 255f).toInt(), (b / top * 255f).toInt())
        }
        bitmap.setPixels(pixels, 0, cols, 0, 0, cols, rows)
        // Each grid point is the middle of its cell, one cell outside the sky on every side.
        rect.set(-cell, -cell, -cell + cols * cell, -cell + rows * cell)
        paint.alpha = 255
        canvas.drawBitmap(bitmap, null, rect, paint)
    }

    // Adds level [level] of the energy in [from] to [into] as light: blurred by its width and spread over a cell.
    private fun blur(from: FloatArray, into: FloatArray, level: Int) {
        val kernel = kernels[level] ?: gaussian(SkyBloom.SIGMAS[level] / CELL).also { kernels[level] = it }
        val reach = kernel.size / 2
        val scale = SkyBloom.WEIGHTS[level] / (CELL * CELL)
        val tmp = spare
        for (j in 0 until rows) {
            val row = j * cols
            for (i in 0 until cols) {
                var v = 0f
                val lo = max(0, i - reach)
                val hi = min(cols - 1, i + reach)
                for (q in lo..hi) v += from[row + q] * kernel[q - i + reach]
                tmp[row + i] = v
            }
        }
        for (i in 0 until cols) {
            for (j in 0 until rows) {
                var v = 0f
                val lo = max(0, j - reach)
                val hi = min(rows - 1, j + reach)
                for (q in lo..hi) v += tmp[q * cols + i] * kernel[q - j + reach]
                into[j * cols + i] += v * scale
            }
        }
    }

    private companion object {
        /** The grid's cell in CSS pixels: a third of the second widest level's width, smooth once stretched. */
        const val CELL = 13f

        // A normalised Gaussian [sigma] cells wide, out to three widths.
        fun gaussian(sigma: Float): FloatArray {
            val reach = kotlin.math.ceil(3f * sigma).toInt()
            val k = FloatArray(2 * reach + 1) { exp(-((it - reach) * (it - reach)) / (2f * sigma * sigma)) }
            val sum = k.sum()
            for (i in k.indices) k[i] /= sum
            return k
        }
    }
}

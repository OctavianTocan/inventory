package com.octaviantocan.mymind.ui.sky

import android.graphics.Bitmap
import android.graphics.BlendMode
import android.graphics.BlurMaskFilter
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sin

/**
 * The prototype adds its light in linear colour and turns the sum into the screen's sRGB at the end, which is what
 * makes a faint halo glow rather than vanish. Each light here is turned into sRGB on its own before it is gathered
 * with Screen ([LightBlend]): exact over black, a little brighter where lights overlap.
 */
internal object Srgb {
    private const val STEPS = 4096
    private val ENCODE = FloatArray(STEPS + 1) { exactEncode(it / STEPS.toFloat()) }

    /** sRGB of a linear value from 0 to 1, clamped. */
    fun enc(linear: Float): Float {
        if (linear <= 0f) return 0f
        if (linear >= 1f) return 1f
        return ENCODE[(linear * STEPS + 0.5f).toInt()]
    }

    /** Linear value of an sRGB value from 0 to 1. */
    fun dec(srgb: Float): Float = if (srgb <= 0.04045f) srgb / 12.92f else ((srgb + 0.055f) / 1.055f).pow(2.4f)

    /** The paint alpha that fades an encoded light as fading it by [linear] in linear light would. */
    fun fade(linear: Float): Float = if (linear <= 0f) 0f else if (linear >= 1f) 1f else linear.pow(0.4545f)

    private fun exactEncode(x: Float): Float = if (x <= 0.0031308f) x * 12.92f else 1.055f * x.pow(1f / 2.4f) - 0.055f
}

/**
 * Gathers the sky's light with Screen, a blend every GPU runs in its fixed blending, so the phone draws what the tests
 * draw. Linear light is added only inside [WideBloom], on the CPU. Android 16's own blends (RuntimeXfermode) are not
 * used: on a Galaxy Z Fold's GPU they drew field stars as dark octagons on lighter panels (Octavian's screenshot, 6 November
 * 2026, 16:53).
 */
internal object LightBlend {
    /** Makes [paint] gather its light with what is under it, with Screen. */
    fun use(paint: Paint) {
        paint.blendMode = BlendMode.SCREEN
    }

    /** Makes [paint] gather its light with Screen. */
    fun screen(paint: Paint) = use(paint)
}

/**
 * The prototype's sprite pictures, drawn once a process and made ready by [additive]: a star's glow, its spike, a dark
 * star's ring, a landing's streak and a lit star's bloom.
 */
internal object SkyTextures {
    /** A star's glow: white in the middle, warm gold, fading out. */
    val glow: Bitmap by lazy {
        additive(radial(128, floatArrayOf(0f, 0.08f, 0.22f, 0.5f, 1f), intArrayOf(argb(1f, 255, 252, 244), argb(0.95f, 255, 236, 196), argb(0.32f, 255, 200, 120), argb(0.07f, 255, 170, 80), argb(0f, 255, 160, 60))))
    }

    /** Four long and four short rays. */
    val spike = SpriteLevels(256) { res ->
        val bitmap = Bitmap.createBitmap(res, res, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        canvas.scale(res / 256f, res / 256f)
        canvas.translate(128f, 128f)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { blendMode = BlendMode.PLUS }
        for ((rotation, length, width, alpha) in listOf(Ray(0f, 128f, 1.6f, 1f), Ray(90f, 128f, 1.6f, 1f), Ray(45f, 54f, 0.9f, 0.5f), Ray(-45f, 54f, 0.9f, 0.5f))) {
            canvas.save()
            canvas.rotate(rotation)
            paint.shader = LinearGradient(-length, 0f, length, 0f, intArrayOf(argb(0f, 255, 240, 215), argb(alpha, 255, 248, 235), argb(0f, 255, 240, 215)), null, Shader.TileMode.CLAMP)
            canvas.drawPath(Path().apply { moveTo(-length, 0f); lineTo(0f, -width); lineTo(length, 0f); lineTo(0f, width); close() }, paint)
            canvas.restore()
        }
        additive(bitmap)
    }

    /** A dark star: a thin ring round a small dot. */
    val ring = SpriteLevels(128) { res ->
        val bitmap = Bitmap.createBitmap(res, res, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        canvas.scale(res / 128f, res / 128f)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeWidth = 3f; color = argb(0.9f, 190, 205, 225) }
        canvas.drawCircle(64f, 64f, 40f, paint)
        paint.style = Paint.Style.FILL
        paint.color = argb(0.9f, 200, 215, 235)
        canvas.drawCircle(64f, 64f, 6f, paint)
        additive(bitmap)
    }

    /** The bright bar a shooting star leaves as it lands. */
    val streak: Bitmap by lazy {
        val bitmap = Bitmap.createBitmap(512, 64, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG)
        paint.shader = LinearGradient(0f, 0f, 512f, 0f, intArrayOf(argb(0f, 160, 200, 255), argb(1f, 255, 250, 240), argb(0f, 160, 200, 255)), null, Shader.TileMode.CLAMP)
        canvas.drawRect(0f, 30f, 512f, 34f, paint)
        paint.maskFilter = BlurMaskFilter(6f, BlurMaskFilter.Blur.NORMAL)
        paint.alpha = 128
        canvas.drawRect(0f, 26f, 512f, 38f, paint)
        additive(bitmap)
    }

    /**
     * The bloom's second and third levels round a lit star of [SkyBloom.STAR_ENERGY] at full strength, [BLOOM_SIZE]
     * CSS pixels across, clipped where it would pass white (inside the star's own white middle).
     */
    val bloom: Bitmap by lazy {
        val size = BLOOM_RES
        val pixels = IntArray(size * size)
        for (y in 0 until size) for (x in 0 until size) {
            val v = bloomShare(x, y)
            val r = Srgb.enc(min(1f, v))
            val g = Srgb.enc(min(1f, v * 0.86f))
            val b = Srgb.enc(min(1f, v * 0.67f))
            pixels[y * size + x] = if (r <= 0.002f) 0 else argb(r, 255, (g / r * 255f).toInt(), (b / r * 255f).toInt())
        }
        Bitmap.createBitmap(pixels, size, size, Bitmap.Config.ARGB_8888)
    }

    /** How wide the bloom sprite is, in CSS pixels: three widths of its wider level either side. */
    const val BLOOM_SIZE = 92f

    private const val BLOOM_RES = 256

    private fun bloomShare(x: Int, y: Int): Float {
        val scale = BLOOM_SIZE / BLOOM_RES
        val half = BLOOM_RES / 2f
        val dx = (x + 0.5f - half) * scale
        val dy = (y + 0.5f - half) * scale
        val r2 = dx * dx + dy * dy
        val edge = 1f - sstep(kotlin.math.sqrt(r2), BLOOM_SIZE * 0.38f, BLOOM_SIZE * 0.5f)
        var sum = 0f
        for (i in 1..2) {
            val sigma = SkyBloom.SIGMAS[i]
            sum += SkyBloom.WEIGHTS[i] / (2f * PI.toFloat() * sigma * sigma) * kotlin.math.exp(-r2 / (2f * sigma * sigma))
        }
        return SkyBloom.FULL_GAIN * SkyBloom.STAR_ENERGY * sum * edge
    }

    private data class Ray(val rotation: Float, val length: Float, val width: Float, val alpha: Float)

    private fun radial(size: Int, stops: FloatArray, colors: IntArray): Bitmap {
        val bitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val half = size / 2f
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { shader = RadialGradient(half, half, half, colors, stops, Shader.TileMode.CLAMP) }
        Canvas(bitmap).drawRect(0f, 0f, size.toFloat(), size.toFloat(), paint)
        return bitmap
    }
}

/**
 * [picture] as light to add: each pixel's colour times its alpha, taken to linear and back to sRGB, so drawing it with
 * an added blend gives what the prototype's added sprite gives over black.
 */
internal fun additive(picture: Bitmap): Bitmap {
    val width = picture.width
    val height = picture.height
    val pixels = IntArray(width * height)
    picture.getPixels(pixels, 0, width, 0, 0, width, height)
    for (i in pixels.indices) {
        val p = pixels[i]
        val a = (p ushr 24) / 255f
        if (a <= 0f) {
            pixels[i] = 0
            continue
        }
        val r = Srgb.enc(Srgb.dec((p shr 16 and 255) / 255f) * a)
        val g = Srgb.enc(Srgb.dec((p shr 8 and 255) / 255f) * a)
        val b = Srgb.enc(Srgb.dec((p and 255) / 255f) * a)
        val top = max(r, max(g, b))
        if (top <= 0.002f) {
            pixels[i] = 0
            continue
        }
        pixels[i] = argb(top, (r / top * 255f).toInt(), (g / top * 255f).toInt(), (b / top * 255f).toInt())
    }
    val out = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    out.setPixels(pixels, 0, width, 0, 0, width, height)
    if (picture !== out) picture.recycle()
    return out
}

/** A colour from an alpha from 0 to 1 and red, green, blue from 0 to 255. */
internal fun argb(alpha: Float, red: Int, green: Int, blue: Int): Int =
    ((alpha.coerceIn(0f, 1f) * 255f + 0.5f).toInt() shl 24) or (red.coerceIn(0, 255) shl 16) or (green.coerceIn(0, 255) shl 8) or blue.coerceIn(0, 255)

/** An opaque colour from linear red, green and blue times [k], turned into sRGB. */
internal fun lightColor(red: Float, green: Float, blue: Float, k: Float = 1f): Int =
    (255 shl 24) or ((Srgb.enc(red * k) * 255f + 0.5f).toInt() shl 16) or ((Srgb.enc(green * k) * 255f + 0.5f).toInt() shl 8) or (Srgb.enc(blue * k) * 255f + 0.5f).toInt()

/**
 * Soft round lights drawn in one go, each with its own place, size and colour, added to what is under them. Every
 * light has the same fall-off: [levels] of its colour at its middle and then at each of [rings], given as shares of its
 * radius. Draw onto something opaque: each light also adds alpha.
 *
 * @param capacity The most lights it holds.
 * @param rings Where the fall-off is given, from the middle out, as shares of a light's radius; the last is its edge.
 * @param levels How much of the light's colour shows at the middle and at each ring, in linear light.
 */
internal class GlowBatch(
    private val capacity: Int,
    private val rings: FloatArray = SOFT_RINGS,
    private val levels: FloatArray = SOFT_LEVELS,
) {
    private val sides = SIDES
    private val verts = 1 + sides * rings.size
    private val tris = sides + (rings.size - 1) * sides * 2
    private val chunk = 65535 / verts

    // One pair of arrays per draw: Android adds a draw's vertex offset to the missing texture pointer and reads from
    // there (libs/hwui/jni/android_graphics_Canvas.cpp, drawVertices), so a draw must always start at offset 0.
    private val positions = Array((capacity + chunk - 1) / chunk) { FloatArray(chunk * verts * 2) }
    private val colors = Array(positions.size) { IntArray(chunk * verts) }
    private val indices: ShortArray = ShortArray(chunk * tris * 3).also { index ->
        var k = 0
        for (glow in 0 until chunk) {
            val base = glow * verts
            for (i in 0 until sides) {
                val a = base + 1 + i
                val b = base + 1 + (i + 1) % sides
                index[k++] = base.toShort(); index[k++] = a.toShort(); index[k++] = b.toShort()
            }
            for (ring in 0 until rings.size - 1) {
                val inner = base + 1 + ring * sides
                val outer = inner + sides
                for (i in 0 until sides) {
                    val a = inner + i
                    val b = inner + (i + 1) % sides
                    val oa = outer + i
                    val ob = outer + (i + 1) % sides
                    index[k++] = a.toShort(); index[k++] = oa.toShort(); index[k++] = ob.toShort()
                    index[k++] = a.toShort(); index[k++] = ob.toShort(); index[k++] = b.toShort()
                }
            }
        }
    }
    private val paint = Paint().also { LightBlend.screen(it) }
    private var count = 0

    init {
        require(levels.size == rings.size + 1)
    }

    /** How many lights are waiting to be drawn. */
    val size: Int get() = count

    /**
     * Adds a light at [x], [y] with [radius] pixels to its edge, of linear colour [red], [green], [blue] times [k].
     */
    fun add(x: Float, y: Float, radius: Float, red: Float, green: Float, blue: Float, k: Float) {
        if (count >= capacity || k <= 0.0005f || radius <= 0.3f) return
        val at = positions[count / chunk]
        val tint = colors[count / chunk]
        var v = count % chunk * verts * 2
        var c = count % chunk * verts
        at[v++] = x; at[v++] = y; tint[c++] = lightColor(red, green, blue, k * levels[0])
        for (ring in rings.indices) {
            val r = radius * rings[ring]
            val color = lightColor(red, green, blue, k * levels[ring + 1])
            for (i in 0 until sides) {
                at[v++] = x + COS[i] * r; at[v++] = y + SIN[i] * r; tint[c++] = color
            }
        }
        count++
    }

    /** Draws the lights onto [canvas], then forgets them. */
    fun draw(canvas: Canvas) {
        var k = 0
        while (k * chunk < count) {
            val n = minOf(chunk, count - k * chunk)
            canvas.drawVertices(Canvas.VertexMode.TRIANGLES, n * verts * 2, positions[k], 0, null, 0, colors[k], 0, indices, 0, n * tris * 3, paint)
            k++
        }
        count = 0
    }

    /** Forgets the lights added so far. */
    fun clear() {
        count = 0
    }

    companion object {
        private const val SIDES = 10
        private val COS = FloatArray(SIDES) { cos(it * 2 * PI / SIDES).toFloat() }
        private val SIN = FloatArray(SIDES) { sin(it * 2 * PI / SIDES).toFloat() }

        /** A point of light, exp(-d² × 3) to the edge: the prototype's condensing stars. */
        val SOFT_RINGS = floatArrayOf(0.3f, 0.6f, 1f)
        val SOFT_LEVELS = floatArrayOf(1f, 0.76f, 0.34f, 0f)

        /** A field star: a sharp core and a faint halo, as the prototype's star layers draw them. */
        val STAR_RINGS = floatArrayOf(0.0625f, 0.125f, 0.25f, 0.4f, 0.6f, 1f)
        val STAR_LEVELS = floatArrayOf(1.22f, 1.08f, 0.83f, 0.31f, 0.066f, 0.018f, 0f)
    }
}

/**
 * Draws [bitmap], made by [additive], as a sprite centred on [x], [y], [size] pixels across and [height] high, at
 * [opacity] in linear light, turned [rotation] radians, added to what is under it.
 */
internal fun Canvas.sprite(bitmap: Bitmap, x: Float, y: Float, size: Float, opacity: Float, paint: Paint, rotation: Float = 0f, height: Float = size) {
    if (opacity <= 0.002f || size <= 0.5f) return
    paint.alpha = (Srgb.fade(opacity) * 255f).toInt()
    val rect = RECT
    rect.set(x - size / 2f, y - height / 2f, x + size / 2f, y + height / 2f)
    if (rotation != 0f) {
        save()
        rotate(Math.toDegrees(rotation.toDouble()).toFloat(), x, y)
        drawBitmap(bitmap, null, rect, paint)
        restore()
    } else {
        drawBitmap(bitmap, null, rect, paint)
    }
}

private val RECT = RectF()

/**
 * A sprite picture drawn at its full size and at each half of it down to 16 pixels, as the prototype's textures keep
 * mipmaps: Android draws a bitmap without them, so a sprite drawn small takes the level nearest its size, and its thin
 * rays and rings fade as they shrink instead of staying sharp and long.
 */
internal class SpriteLevels(full: Int, draw: (Int) -> Bitmap) {
    private val levels: List<Bitmap> by lazy { generateSequence(full) { it / 2 }.takeWhile { it >= 16 }.map(draw).toList() }

    /** The level to draw [size] pixels across. */
    fun at(size: Float): Bitmap {
        val all = levels
        val level = kotlin.math.log2(all[0].width / max(size, 1f)).roundToInt()
        return all[level.coerceIn(0, all.lastIndex)]
    }
}

/** [Canvas.sprite] with the level of [levels] nearest its size. */
internal fun Canvas.sprite(levels: SpriteLevels, x: Float, y: Float, size: Float, opacity: Float, paint: Paint, rotation: Float = 0f) {
    if (opacity <= 0.002f || size <= 0.5f) return
    sprite(levels.at(size), x, y, size, opacity, paint, rotation)
}

/** A paint for sprites: smooth, gathering its light with the others. */
internal fun spritePaint(): Paint = Paint(Paint.FILTER_BITMAP_FLAG or Paint.ANTI_ALIAS_FLAG).also { LightBlend.use(it) }

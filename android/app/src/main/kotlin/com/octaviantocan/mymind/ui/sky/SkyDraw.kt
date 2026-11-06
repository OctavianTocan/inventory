package com.octaviantocan.mymind.ui.sky

import android.graphics.Bitmap
import android.graphics.BlendMode
import android.graphics.BitmapShader
import android.graphics.Canvas
import android.graphics.DashPathEffect
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.RadialGradient
import android.graphics.RuntimeShader
import android.graphics.Shader
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin
import kotlin.random.Random

/**
 * Draws a [SkyScene] the way the prototype's renderer does: the star layers and the dome's finest stars, the figures,
 * the lines, the stars as sprites with their bloom, a shooting star and its sparks, a vortex forming, the find
 * condensing, and the vignette. The dome itself is drawn under it by [drawDome].
 */
internal class SkyPainter {
    private var projector: SkyProjector? = null
    private val at = Projected()
    private val at2 = Projected()
    private val at3 = Projected()
    private val sprite = spritePaint()
    private val fieldStars = GlowBatch(7000, GlowBatch.STAR_RINGS, GlowBatch.STAR_LEVELS)
    private val soft = GlowBatch(FindView.MAX_CONDENSE + 600)
    private val fieldGlow = GlowBatch(4000)
    private val microPaints = Array(MICRO_LEVELS.size) {
        Paint(Paint.ANTI_ALIAS_FLAG).apply { LightBlend.screen(this); strokeCap = Paint.Cap.ROUND; color = lightColor(0.7f, 0.8f, 1f, MICRO_LEVELS[it]) }
    }
    private val microPoints = Array(MICRO_LEVELS.size) { FloatArray(8000 * 2) }
    private val microCounts = IntArray(MICRO_LEVELS.size)
    private val matrix = Matrix()
    private val src = FloatArray(8)
    private val dst = FloatArray(8)
    private val figurePaint = Paint(Paint.FILTER_BITMAP_FLAG or Paint.ANTI_ALIAS_FLAG).also { LightBlend.use(it) }
    private val strokePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeCap = Paint.Cap.ROUND; strokeJoin = Paint.Join.ROUND; blendMode = BlendMode.PLUS }
    private val linePaint = Paint().also { LightBlend.use(it) }
    private val lineShader: RuntimeShader? = if (SkyShaders.supported) SkyShaders.line() else null
    private val shockShader: RuntimeShader? = if (SkyShaders.supported) SkyShaders.shock() else null
    private val smokeShader: RuntimeShader? = if (SkyShaders.supported) SkyShaders.smoke() else null
    private val effectPaint = Paint().also { LightBlend.use(it) }
    private val fallbackLine = Paint(Paint.ANTI_ALIAS_FLAG).apply { LightBlend.use(this); style = Paint.Style.STROKE; strokeCap = Paint.Cap.ROUND }
    private val vignettePaint = Paint()
    private var vignetteFor = -1f
    private var vignetteSize = 0L
    private var smokeFor: Bitmap? = null
    private val ribbonPositions = FloatArray((SkyScene.SEG + 1) * 3 * 2)
    private val ribbonColors = IntArray((SkyScene.SEG + 1) * 3)
    private val ribbonIndices = ShortArray(SkyScene.SEG * 12).also { index ->
        var k = 0
        for (i in 0 until SkyScene.SEG) {
            val a = i * 3
            val b = a + 3
            index[k++] = a.toShort(); index[k++] = (a + 1).toShort(); index[k++] = b.toShort()
            index[k++] = (a + 1).toShort(); index[k++] = (b + 1).toShort(); index[k++] = b.toShort()
            index[k++] = (a + 1).toShort(); index[k++] = (a + 2).toShort(); index[k++] = (b + 1).toShort()
            index[k++] = (a + 2).toShort(); index[k++] = (b + 2).toShort(); index[k++] = (b + 1).toShort()
        }
    }
    private val ribbonPaint = Paint().also { LightBlend.use(it) }
    private val layerPaint = Paint()
    private var blendVersion = -1
    private val wide = WideBloom()
    private var gain = 0f

    /** The camera for a sky [width] by [height] pixels, placed for this frame. */
    fun camera(scene: SkyScene, width: Float, height: Float): SkyProjector {
        val p = projector?.takeIf { it.widthPx == width && it.heightPx == height } ?: SkyProjector(width, height).also { projector = it }
        p.place(scene.look, scene.punch, scene.jitterX, scene.jitterY)
        p.slide.set(scene.focusTx * scene.fz + scene.dragX, scene.focusTy * scene.fz + scene.dragY, -Proto.RC + 700f * scene.fz * (1f - scene.zoomDip) + scene.pushZ)
        return p
    }

    /** Draws [scene] onto [canvas], a sky [width] by [height] pixels. */
    fun draw(canvas: Canvas, scene: SkyScene, width: Float, height: Float) {
        val p = camera(scene, width, height)
        val look = scene.look
        if (look <= 0.001f) return
        val now = scene.time.now
        val so = sstep(look, 0.16f, 0.5f)
        if (blendVersion != LightBlend.version) {
            blendVersion = LightBlend.version
            for (paint in listOf(sprite, figurePaint, linePaint, effectPaint, fallbackLine, ribbonPaint, layerPaint)) LightBlend.use(paint)
        }
        // Every light is gathered in one layer, then added to the dome in linear light as the prototype adds it.
        canvas.saveLayer(null, layerPaint)
        gain = SkyBloom.gain(SkyBloom.strength(look, scene.bloomKick))
        wide.begin(width, height, p.unit)
        drawMicro(canvas, p, look)
        drawLayers(canvas, p, now, sstep(look, 0.03f, 0.4f))
        if (so > 0.01f) {
            for (con in scene.cons) drawFigure(canvas, p, scene, con, so)
            for (con in scene.cons) drawLines(canvas, p, scene, con, so, now)
            for (con in scene.cons) drawStars(canvas, p, scene, con, so, now)
            drawLoose(canvas, p, scene, so, now)
            for (con in scene.cons) drawVortex(canvas, p, con)
        }
        drawShootingStar(canvas, p, scene)
        drawSparks(canvas, p, scene)
        drawComet(canvas, p, scene)
        if (gain > 0.01f) wide.draw(canvas, sprite, width, height)
        scene.find?.let { drawFind(canvas, p, scene, it, now) }
        canvas.restore()
        drawVignette(canvas, width, height, sstep(look, 0.2f, 1f))
    }

    /** Which star of the middle constellation is under [x], [y] pixels of a sky [width] by [height], or -1. */
    fun starAt(scene: SkyScene, x: Float, y: Float, width: Float, height: Float): Int {
        val p = camera(scene, width, height)
        val con = scene.active ?: return -1
        var best = -1
        var bestDistance = Float.MAX_VALUE
        for (i in con.stars.indices) {
            val star = con.stars[i]
            if (star.vis < 0.5f) continue
            if (!p.projectSlide(con.x + con.px[i] * con.scale, con.y + con.py[i] * con.scale, con.z, at)) continue
            val half = (if (star.lit) 64f + star.big * 16f else 30f) * 0.5f * at.s * con.scale
            val reach = max(half, HIT_UNITS * p.unit)
            val d = kotlin.math.hypot(x - at.x, y - at.y)
            if (d < reach && d < bestDistance) {
                bestDistance = d
                best = i
            }
        }
        return best
    }

    /** Where star [i] of the middle constellation shows on a sky [width] by [height], in pixels; null out of sight. */
    fun starPoint(scene: SkyScene, i: Int, width: Float, height: Float): Pair<Float, Float>? {
        val p = camera(scene, width, height)
        val con = scene.active ?: return null
        if (i !in con.stars.indices || !p.projectSlide(con.x + con.px[i] * con.scale, con.y + con.py[i] * con.scale, con.z, at)) return null
        return at.x to at.y
    }

    /** Where the middle constellation's name sits: centred on x, its top at y, in pixels; null while out of sight. */
    fun labelAt(scene: SkyScene): Pair<Float, Float>? {
        val p = projector ?: return null
        val con = scene.active ?: return null
        if (!p.projectSlide(con.x, con.y + (-Proto.SY * 1.05f - 34f) * con.scale, con.z, at)) return null
        return at.x to at.y
    }

    private fun drawMicro(canvas: Canvas, p: SkyProjector, look: Float) {
        val stars = SkySceneStatics.micro
        val pitch = look * Proto.PITCH
        val c = cos(pitch)
        val s = sin(pitch)
        val half = p.w / 2f
        val f = Proto.F
        val unit = p.unit
        val dusk = 1f - 0.6f * (1f - sstep(look, 0.35f, 0.95f))
        microCounts.fill(0)
        val maxX = p.widthPx
        val maxY = p.heightPx
        for (i in stars.dx.indices) {
            val vy = stars.dy[i]
            val vz = stars.dz[i]
            val cz = -s * vy + c * vz
            if (cz >= -0.01f) continue
            val cy = c * vy + s * vz
            val k = f / -cz
            val x = (half + stars.dx[i] * k) * unit
            if (x < 0f || x > maxX) continue
            val y = (Proto.H / 2f - cy * k) * unit
            if (y < 0f || y > maxY) continue
            val b = stars.bright[i] * dusk
            if (gain > 0.01f && b > 0.62f) smallBloom(x, y, SkyBloom.microEnergy(b) * gain, 0.8f / 0.7f, 1f / 0.7f)
            var level = MICRO_LEVELS.size - 1
            while (level > 0 && b < MICRO_LEVELS[level] * 0.75f) level--
            if (b < MICRO_LEVELS[0] * 0.6f) continue
            val n = microCounts[level]
            val points = microPoints[level]
            if (n * 2 + 1 >= points.size) continue
            points[n * 2] = x
            points[n * 2 + 1] = y
            microCounts[level] = n + 1
        }
        for (level in MICRO_LEVELS.indices) {
            if (microCounts[level] == 0) continue
            microPaints[level].strokeWidth = unit * 1.05f
            canvas.drawPoints(microPoints[level], 0, microCounts[level] * 2, microPaints[level])
        }
    }

    private fun drawLayers(canvas: Canvas, p: SkyProjector, now: Float, op: Float) {
        if (op <= 0.002f) return
        val unit = p.unit
        val maxX = p.widthPx
        val maxY = p.heightPx
        for (layer in SkySceneStatics.layers) {
            for (i in 0 until layer.count) {
                if (!p.projectSlide(layer.x[i], layer.y[i], layer.z, at)) continue
                val radius = layer.size[i] * 3.2f * 0.5f * unit
                if (at.x < -radius || at.x > maxX + radius || at.y < -radius || at.y > maxY + radius) continue
                val ph = layer.phase[i]
                val tw = 0.7f + 0.3f * sin(now * (1.1f + ph * 0.6f) + ph * 9f)
                val tint = FIELD_TINTS[layer.tint[i].let { if (it < 0.33f) 0 else if (it < 0.8f) 1 else 2 }]
                fieldStars.add(at.x, at.y, radius, tint[0], tint[1], tint[2], tw * op)
                if (gain > 0.01f) {
                    val energy = SkyBloom.fieldEnergy(radius / unit, tw * op, tint[0], tint[3])
                    if (energy > 0f) smallBloom(at.x, at.y, energy * gain, tint[1] / tint[0], tint[2] / tint[0])
                }
            }
        }
        fieldStars.draw(canvas)
        fieldGlow.draw(canvas)
    }

    // The bloom of a small light: its two finest levels as a soft glow round it, the rest in the wide grid.
    private fun smallBloom(x: Float, y: Float, energy: Float, g: Float, b: Float) {
        if (energy <= 0.02f) return
        wide.field(x, y, energy, g, b)
        val unit = wide.unit
        for (level in 0..1) {
            val sigma = SkyBloom.SIGMAS[level]
            fieldGlow.add(x, y, sigma * SOFT_REACH * unit, 1f, g, b, energy * SkyBloom.WEIGHTS[level] / (PI2 * sigma * sigma))
        }
    }

    private fun conOp(scene: SkyScene, con: ConState): Float = if (con === scene.active) 1f else 1f - scene.fz * 0.8f

    private fun drawFigure(canvas: Canvas, p: SkyProjector, scene: SkyScene, con: ConState, so: Float) {
        val op = so * con.figOp * (if (con === scene.active) 0.6f else 0.2f) * con.dim * conOp(scene, con)
        if (op <= 0.003f || con.figDraw <= 0f) return
        val res = if (con.scale < 0.5f && con.figDraw >= 1f) 384 else 1024
        if (con.figBitmap == null || con.figRes != res || con.figShown != con.figDraw) renderFigure(con, res)
        val bitmap = con.figBitmap ?: return
        val hx = Proto.SX * 1.28f
        val hy = Proto.SY * 1.28f
        val z = con.z - 5f * con.scale
        if (!corner(p, con.x - hx * con.scale, con.y + hy * con.scale, z, 0)) return
        if (!corner(p, con.x + hx * con.scale, con.y + hy * con.scale, z, 1)) return
        if (!corner(p, con.x + hx * con.scale, con.y - hy * con.scale, z, 2)) return
        if (!corner(p, con.x - hx * con.scale, con.y - hy * con.scale, z, 3)) return
        val r = res.toFloat()
        src[0] = 0f; src[1] = 0f; src[2] = r; src[3] = 0f; src[4] = r; src[5] = r; src[6] = 0f; src[7] = r
        matrix.setPolyToPoly(src, 0, dst, 0, 4)
        figurePaint.alpha = (Srgb.fade(op) * 255f).toInt()
        canvas.drawBitmap(bitmap, matrix, figurePaint)
    }

    private fun corner(p: SkyProjector, x: Float, y: Float, z: Float, index: Int): Boolean {
        if (!p.projectSlide(x, y, z, at)) return false
        dst[index * 2] = at.x
        dst[index * 2 + 1] = at.y
        return true
    }

    private fun renderFigure(con: ConState, res: Int) {
        val bitmap = con.figBitmap?.takeIf { it.width == res } ?: Bitmap.createBitmap(res, res, Bitmap.Config.ARGB_8888).also {
            con.figBitmap?.recycle()
            con.figBitmap = it
        }
        bitmap.eraseColor(0)
        val canvas = Canvas(bitmap)
        val k = res * 400f / 1024f
        canvas.translate(res / 2f, res / 2f)
        canvas.scale(k, -k)
        for ((i, path) in con.figPaths.withIndex()) {
            val pi = ((con.figDraw - i * 0.07f) / 0.55f).coerceIn(0f, 1f)
            if (pi <= 0f) continue
            strokePaint.pathEffect = if (pi < 1f) DashPathEffect(floatArrayOf(pi * 1900f / 400f, 6000f / 400f), 0f) else null
            for (stroke in FIGURE_STROKES) {
                strokePaint.strokeWidth = stroke.width / 400f
                strokePaint.color = stroke.color
                canvas.drawPath(path, strokePaint)
            }
        }
        strokePaint.pathEffect = null
        con.figRes = res
        con.figShown = con.figDraw
    }

    private fun drawLines(canvas: Canvas, p: SkyProjector, scene: SkyScene, con: ConState, so: Float, now: Float) {
        val isActive = con === scene.active
        val fzD = if (isActive) scene.fz else 0f
        val fi = if (isActive && scene.focusedCon === con) scene.focusedIndex else -1
        val base = so * con.lineVis * con.dim * conOp(scene, con)
        if (base <= 0.002f) return
        val shader = lineShader
        val bloom = gain
        for (line in con.lines) {
            val touch = line.a == fi || line.b == fi
            val op = base * (if (touch) 1f else 1f - fzD * 0.7f)
            if (op <= 0.002f) continue
            val f = line.from
            val t = line.to
            val x0 = con.px[f]
            val y0 = con.py[f]
            val x1 = con.px[t]
            val y1 = con.py[t]
            val len = kotlin.math.hypot(x1 - x0, y1 - y0)
            if (len < 1f) continue
            if (shader == null) {
                drawFallbackLine(canvas, p, con, x0, y0, x1, y1, line, op)
                continue
            }
            val ux = (x1 - x0) / len
            val uy = (y1 - y0) / len
            val mx = (x0 + x1) / 2f
            val my = (y0 + y1) / 2f
            val s = con.scale
            if (!p.projectSlide(con.x + mx * s, con.y + my * s, con.z, at)) continue
            val k = at.s * s / p.unit
            val threshold = SkyBloom.lineThreshold(k)
            if (bloom > 0.01f && line.lit > 0.01f) {
                val end = line.draw.coerceIn(0f, 1f)
                if (p.projectSlide(con.x + x0 * s, con.y + y0 * s, con.z, at2) && p.projectSlide(con.x + (x0 + (x1 - x0) * end) * s, con.y + (y0 + (y1 - y0) * end) * s, con.z, at3)) {
                    wide.line(at2.x, at2.y, at3.x, at3.y, SkyBloom.lineEnergy(op * (1f + PULSE_MEAN * line.lit), k, threshold) * bloom * line.lit, 0.86f, 0.56f)
                }
            }
            val half = if (line.lit <= 0.01f) DARK_HALF else if (bloom > 0.01f) max(LINE_HALF, BLOOM_REACH / max(k, 0.1f)) else LINE_HALF
            fun put(index: Int, lx: Float, ly: Float): Boolean {
                val gx = mx + ux * lx - uy * ly
                val gy = my + uy * lx + ux * ly
                return corner(p, con.x + gx * s, con.y + gy * s, con.z, index)
            }
            if (!put(0, -len / 2f, -half) || !put(1, len / 2f, -half) || !put(2, len / 2f, half) || !put(3, -len / 2f, half)) continue
            src[0] = -len / 2f; src[1] = -half; src[2] = len / 2f; src[3] = -half; src[4] = len / 2f; src[5] = half; src[6] = -len / 2f; src[7] = half
            matrix.setPolyToPoly(src, 0, dst, 0, 4)
            shader.setFloatUniform("uLen", len)
            shader.setFloatUniform("uLit", line.lit)
            shader.setFloatUniform("uDraw", line.draw)
            shader.setFloatUniform("uTime", now)
            shader.setFloatUniform("uOp", op)
            shader.setFloatUniform("uSurge", line.surge)
            shader.setFloatUniform("uBloom", bloom * line.lit)
            shader.setFloatUniform("uHalf", half)
            shader.setFloatUniform("uK", k)
            shader.setFloatUniform("uT", threshold)
            linePaint.shader = shader
            canvas.save()
            canvas.concat(matrix)
            canvas.drawRect(-len / 2f, -half, len / 2f, half, linePaint)
            canvas.restore()
        }
    }

    // Phones before Android 13: a plain glowing line, dotted while dark.
    private fun drawFallbackLine(canvas: Canvas, p: SkyProjector, con: ConState, x0: Float, y0: Float, x1: Float, y1: Float, line: LineState, op: Float) {
        val s = con.scale
        if (!p.projectSlide(con.x + x0 * s, con.y + y0 * s, con.z, at)) return
        if (!p.projectSlide(con.x + (x0 + (x1 - x0) * line.draw) * s, con.y + (y0 + (y1 - y0) * line.draw) * s, con.z, at2)) return
        val width = at.s * s
        if (line.lit > 0.5f) {
            fallbackLine.pathEffect = null
            fallbackLine.strokeWidth = width * 6f
            fallbackLine.color = lightColor(1f, 0.86f, 0.56f, 0.12f * op)
            canvas.drawLine(at.x, at.y, at2.x, at2.y, fallbackLine)
            fallbackLine.strokeWidth = width * 1.6f
            fallbackLine.color = lightColor(1f, 0.86f, 0.56f, 0.95f * op)
            canvas.drawLine(at.x, at.y, at2.x, at2.y, fallbackLine)
        } else {
            fallbackLine.strokeWidth = width * 1.4f
            fallbackLine.pathEffect = DashPathEffect(floatArrayOf(width * 2f, width * 9f), 0f)
            fallbackLine.color = lightColor(0.55f, 0.64f, 0.76f, 0.45f * op)
            canvas.drawLine(at.x, at.y, at2.x, at2.y, fallbackLine)
        }
    }

    private fun drawStars(canvas: Canvas, p: SkyProjector, scene: SkyScene, con: ConState, so: Float, now: Float) {
        val isActive = con === scene.active
        val fz = scene.fz
        val fzD = if (isActive) fz else 0f
        val fi = if (isActive && scene.focusedCon === con) scene.focusedIndex else -1
        val away = conOp(scene, con)
        for ((i, star) in con.stars.withIndex()) {
            if (!p.projectSlide(con.x + con.px[i] * con.scale, con.y + con.py[i] * con.scale, con.z, at)) continue
            val px = at.s * con.scale
            val tw = 0.86f + 0.14f * sin(now * (1.7f + star.seed * 0.2f) + star.seed * 5f)
            val pa = now - star.popT
            val popS = if (pa < 1.2f) 1f + sin(min(1f, pa / 0.5f) * PI.toFloat()) * 0.9f * star.pop * (1f - pa / 1.2f) else 1f
            val sel = i == fi
            val f = so * star.vis * con.dim * away * (if (sel) 1f else 1f - fzD * 0.6f)
            if (f <= 0.002f) continue
            val grow = if (sel) 1f + 0.35f * fz else 1f
            if (star.lit) {
                val b = (64f + star.big * 16f) * tw * popS * grow
                drawBloom(canvas, p, b * px, f)
                canvas.sprite(SkyTextures.glow, at.x, at.y, b * px, f, sprite)
                canvas.sprite(SkyTextures.spike, at.x, at.y, b * 1.9f * (1f - 0.45f * fzD) * px, f * (0.55f + 0.45f * tw), sprite, 0.1f * sin(now * 0.3f + star.seed))
            } else {
                val glow = f * (0.32f + (if (sel) 0.3f * fz else 0f)) * tw
                canvas.sprite(SkyTextures.glow, at.x, at.y, 30f * popS * grow * px, glow, sprite)
                val ring = 30f * popS * grow * (if (sel) 1f + 0.12f * sin(now * 4f) else 1f)
                canvas.sprite(SkyTextures.ring, at.x, at.y, ring * px, f * 0.7f * (0.8f + 0.2f * tw) * (if (sel) 1.4f else 1f), sprite)
            }
        }
    }

    private fun drawLoose(canvas: Canvas, p: SkyProjector, scene: SkyScene, so: Float, now: Float) {
        for (star in scene.loose) {
            if (!p.projectSlide(star.x, star.y, 0f, at)) continue
            val tw = 0.8f + 0.2f * sin(now * 1.9f + star.seed * 7f)
            val a = min(1f, (now - star.t0) / 0.4f).coerceAtLeast(0f)
            val fade = 1f - scene.fz * 0.5f
            drawBloom(canvas, p, 40f * tw * at.s, so * a * fade)
            canvas.sprite(SkyTextures.glow, at.x, at.y, 40f * tw * at.s, so * a * fade, sprite)
            canvas.sprite(SkyTextures.spike, at.x, at.y, 70f * tw * at.s, so * a * 0.6f * tw * fade, sprite)
        }
    }

    /**
     * The prototype's bloom round a glowing star at [at] whose glow is [size] pixels across at [opacity]: the bright
     * middle it gives ([SkyBloom.starEnergy]) as a sprite of the narrower levels here and as a point of the wider ones.
     */
    private fun drawBloom(canvas: Canvas, p: SkyProjector, size: Float, opacity: Float) {
        if (gain <= 0.01f) return
        val css = size / p.unit
        val energy = SkyBloom.starEnergy(opacity, css)
        if (energy <= 0.01f) return
        wide.point(at.x, at.y, energy * gain, 0.86f, 0.67f)
        // A star drawn large has a wide bright middle, which spreads the narrow levels.
        val spread = kotlin.math.sqrt(1f + (0.05f * css / 10f).let { it * it })
        val amount = gain / SkyBloom.FULL_GAIN * energy / SkyBloom.STAR_ENERGY / (spread * spread)
        canvas.sprite(SkyTextures.bloom, at.x, at.y, SkyTextures.BLOOM_SIZE * p.unit * spread, min(1f, amount), sprite)
    }

    private fun drawVortex(canvas: Canvas, p: SkyProjector, con: ConState) {
        val points = con.vortexPoints ?: return
        val t = con.vortex
        if (t < 0f) return
        val n = points.size / 6
        val shrink = 1f - 0.45f * sstep(t, 0f, 1.8f)
        for (k in 0 until n) {
            val x0 = points[k * 6]
            val y0 = points[k * 6 + 1]
            val dl = points[k * 6 + 4]
            val sd = points[k * 6 + 5]
            val ang0 = kotlin.math.atan2(y0, x0)
            val r0 = kotlin.math.hypot(x0, y0)
            val sw = t * (1.3f + sd * 1.7f)
            val r = r0 * shrink
            val sx = cos(ang0 + sw) * r
            val sy = sin(ang0 + sw) * r * 0.82f
            val amb = if (dl >= 8f) 1f else 0f
            var k2 = ((t - 1.4f - dl) / 0.85f).coerceIn(0f, 1f) * (1f - amb)
            k2 = k2 * k2 * (3f - 2f * k2)
            val gx = mix(sx, points[k * 6 + 2], k2)
            val gy = mix(sy, points[k * 6 + 3], k2)
            val va = sstep(t, 0f, 0.5f) * (1f - sstep(t - 2.25f - dl, 0f, 0.3f) * (1f - amb)) * (1f - amb * sstep(t, 2.2f, 3.2f))
            if (va <= 0.002f) continue
            if (!p.projectSlide(con.x + gx * con.scale, con.y + gy * con.scale, con.z, at)) continue
            val size = (1.4f + sd * 2.6f) * (1f + (1f - k2) * 0.6f) * p.unit
            soft.add(at.x, at.y, size * 0.5f * 1.15f, 1f, 0.9f, 0.7f, va)
        }
        soft.draw(canvas)
    }

    private fun drawShootingStar(canvas: Canvas, p: SkyProjector, scene: SkyScene) {
        if (scene.ribbonOn) {
            var ok = true
            for (k in 0..SkyScene.SEG) {
                val t = k / SkyScene.SEG.toFloat()
                val lx = scene.ribbon[k * 4]
                val ly = scene.ribbon[k * 4 + 1]
                val rx = scene.ribbon[k * 4 + 2]
                val ry = scene.ribbon[k * 4 + 3]
                if (!p.projectSlide(lx, ly, 0f, at) || !p.projectSlide(rx, ry, 0f, at2)) {
                    ok = false
                    break
                }
                ribbonPositions[k * 6] = at.x; ribbonPositions[k * 6 + 1] = at.y
                ribbonPositions[k * 6 + 2] = (at.x + at2.x) / 2f; ribbonPositions[k * 6 + 3] = (at.y + at2.y) / 2f
                ribbonPositions[k * 6 + 4] = at2.x; ribbonPositions[k * 6 + 5] = at2.y
                val a = t.pow(2.2f) * scene.ribbonOp * 1.4f
                val m = t * t * t
                val r = mix(0.45f, 1f, m)
                val g = mix(0.6f, 0.93f, m)
                val b = mix(1f, 0.78f, m)
                val edge = lightColor(r, g, b, a * 0.074f)
                ribbonColors[k * 3] = edge
                ribbonColors[k * 3 + 1] = lightColor(r, g, b, a)
                ribbonColors[k * 3 + 2] = edge
            }
            if (ok) canvas.drawVertices(Canvas.VertexMode.TRIANGLES, ribbonPositions.size, ribbonPositions, 0, null, 0, ribbonColors, 0, ribbonIndices, 0, ribbonIndices.size, ribbonPaint)
            if (p.projectSlide(scene.headX, scene.headY, 0f, at)) {
                drawBloom(canvas, p, scene.headSize * at.s, scene.headOp)
                canvas.sprite(SkyTextures.glow, at.x, at.y, scene.headSize * at.s, scene.headOp, sprite)
                canvas.sprite(SkyTextures.spike, at.x, at.y, scene.headSpike * at.s, scene.headOp * 0.9f, sprite, scene.headRot)
            }
        }
        if (scene.flashOp > 0.002f || scene.shockOp > 0.002f || scene.streakOp > 0.002f) {
            if (!p.projectSlide(scene.fxX, scene.fxY, 0f, at)) return
            drawBloom(canvas, p, scene.flashSize * at.s, scene.flashOp)
            canvas.sprite(SkyTextures.glow, at.x, at.y, scene.flashSize * at.s, scene.flashOp, sprite)
            canvas.sprite(SkyTextures.streak, at.x, at.y, scene.streakW * at.s, scene.streakOp, sprite, height = scene.streakH * at.s)
            val shock = shockShader
            if (shock != null && scene.shockOp > 0.002f) {
                val radius = scene.shockSize * 0.5f * at.s
                shock.setFloatUniform("uCenter", at.x, at.y)
                shock.setFloatUniform("uRadius", radius)
                shock.setFloatUniform("uP", scene.shockP)
                shock.setFloatUniform("uOp", scene.shockOp)
                effectPaint.shader = shock
                canvas.drawRect(at.x - radius, at.y - radius, at.x + radius, at.y + radius, effectPaint)
            }
        }
    }

    private fun drawSparks(canvas: Canvas, p: SkyProjector, scene: SkyScene) {
        val now = scene.time.now
        for (i in 0 until SkyScene.SPARKS) {
            if (!scene.sparkLive[i]) continue
            val a = ((now - scene.sparkT0[i]) / scene.sparkD[i]).coerceIn(0f, 1f)
            if (!p.projectSlide(scene.sparkX[i], scene.sparkY[i], 0f, at)) continue
            canvas.sprite(SkyTextures.glow, at.x, at.y, (14f * (1f - a) + 3f) * at.s, (1f - a) * (1f - a) * 0.9f, sprite)
        }
    }

    private fun drawComet(canvas: Canvas, p: SkyProjector, scene: SkyScene) {
        if (scene.cometOp <= 0.002f) return
        val con = scene.focusedCon ?: return
        if (!p.projectSlide(scene.cometX, scene.cometY, con.z, at)) return
        drawBloom(canvas, p, scene.cometSize * at.s, scene.cometOp * 0.95f)
        canvas.sprite(SkyTextures.glow, at.x, at.y, scene.cometSize * at.s, scene.cometOp * 0.95f, sprite)
        canvas.sprite(SkyTextures.spike, at.x, at.y, scene.cometSpike * at.s, scene.cometOp * 0.7f, sprite, scene.cometRot)
    }

    private fun drawFind(canvas: Canvas, p: SkyProjector, scene: SkyScene, view: FindView, now: Float) {
        val unit = p.unit
        val cx = (p.w / 2f + scene.viewDx) * unit
        val cy = (Proto.SKY_STAR_Y + 120f + scene.viewDy) * unit
        val side = SkyScene.FIND_SIDE
        val smoke = smokeShader
        if (smoke != null && view.op > 0.002f) {
            if (smokeFor !== view.picture) {
                smoke.setInputShader("uMap", BitmapShader(view.picture, Shader.TileMode.CLAMP, Shader.TileMode.CLAMP))
                smoke.setFloatUniform("uMapSize", view.picture.width.toFloat(), view.picture.height.toFloat())
                smokeFor = view.picture
            }
            smoke.setFloatUniform("uSide", side)
            smoke.setFloatUniform("uAsp", view.aspect)
            smoke.setFloatUniform("uOp", view.op)
            smoke.setFloatUniform("uIn", view.into)
            smoke.setFloatUniform("uTime", now)
            smoke.setFloatUniform("uWorn", view.wornK)
            effectPaint.shader = smoke
            canvas.save()
            canvas.translate(cx, cy)
            canvas.scale(unit, unit)
            canvas.drawRect(-side / 2f, -side / 2f, side / 2f, side / 2f, effectPaint)
            canvas.restore()
        }
        if (view.condOp <= 0.002f) return
        val w = p.w
        val h = Proto.H
        val shiftX = scene.viewDx
        val shiftY = scene.viewDy
        val into = view.into
        for (i in 0 until view.count) {
            val q = ((into * 1.55f - view.delay[i]) / 0.9f).coerceIn(0f, 1f)
            val e = q * q * (3f - 2f * q)
            val ax = view.sx[i] * w
            val ay = view.sy[i] * h
            val tx = w / 2f + view.tx[i] + shiftX
            val ty = view.ty[i] + shiftY
            val bend = 0.18f * sin(e * PI.toFloat())
            val x = mix(ax, tx, e) + (ty - ay) * bend
            val y = mix(ay, ty, e) - (tx - ax) * bend
            val tw = 0.75f + 0.25f * sin(now * 2.2f + view.phase[i] * 7f)
            val size = view.size[i] * (1f + (1f - e) * 0.8f)
            val a = view.condOp * (0.35f + 0.65f * e) * tw
            soft.add(x * unit, y * unit, size * 0.5f * unit, view.cr[i], view.cg[i], view.cb[i], a)
        }
        soft.draw(canvas)
    }

    private fun drawVignette(canvas: Canvas, width: Float, height: Float, amount: Float) {
        if (amount <= 0.002f) return
        val sizeKey = (width.toLong() shl 20) or height.toLong()
        if (vignetteFor != amount || vignetteSize != sizeKey) {
            vignetteFor = amount
            vignetteSize = sizeKey
            // The prototype's grade, in sRGB: darken by (0.35 + 0.65 v) in linear light, v = smoothstep(0.62, 0.12, r).
            val steps = 9
            val stops = FloatArray(steps) { it / (steps - 1f) }
            val colors = IntArray(steps) { k ->
                val r = stops[k] * 0.62f / 0.55f
                val v = 1f - sstep(r * 0.55f, 0.12f, 0.62f)
                val factor = mix(1f, 0.35f + 0.65f * v, amount)
                argb(1f - factor.pow(0.4545f), 0, 0, 0)
            }
            val gradient = RadialGradient(0f, 0f, 1f, colors, stops, Shader.TileMode.CLAMP)
            val local = Matrix()
            local.setScale(width * 0.62f / 0.55f, height * 0.62f)
            local.postTranslate(width / 2f, height / 2f)
            gradient.setLocalMatrix(local)
            vignettePaint.shader = gradient
        }
        canvas.drawRect(0f, 0f, width, height, vignettePaint)
    }

    private class Stroke(val width: Float, val color: Int)

    companion object {
        /** The least a star's touch target reaches, in units. */
        const val HIT_UNITS = 26f

        /** The least a line's quad reaches either side, in units: the prototype's 14 and room for its glow. */
        const val LINE_HALF = 40f

        /** How far a lit line's bloom reaches either side, in CSS pixels: three widths of its widest level drawn here. */
        const val BLOOM_REACH = 46f

        /** How much brighter a lit line is on average from the pulses travelling along it, as its shader draws them. */
        const val PULSE_MEAN = 0.12f

        /** A soft glow's radius in widths (sigma) of the Gaussian it draws: [GlowBatch.SOFT_LEVELS] is exp(-3d²). */
        const val SOFT_REACH = 2.449f

        const val PI2 = 6.2831855f

        /** The field stars' three colours in linear light, then their luminance. */
        val FIELD_TINTS = arrayOf(floatArrayOf(0.72f, 0.82f, 1f, 0.811f), floatArrayOf(1f, 0.97f, 0.92f, 0.973f), floatArrayOf(1f, 0.83f, 0.62f, 0.851f))

        /** How far a dark line's dots reach either side, in units. */
        const val DARK_HALF = 12f

        private val MICRO_LEVELS = floatArrayOf(0.03f, 0.07f, 0.16f, 0.36f, 0.8f, 1.25f, 2f)

        // The figure's three strokes as the prototype layers them (16 px at 0.10, 7 px at 0.22, 2.2 px at 0.9 in its
        // 1024 canvas), as the amounts each adds once nested strokes are added rather than laid over each other.
        private val FIGURE_STROKES = listOf(
            Stroke(16f, argb(1f, 89, 70, 45)),
            Stroke(7f, argb(1f, 59, 55, 47)),
            Stroke(2.2f, argb(1f, 99, 110, 115)),
        )
    }
}

/** The field and the dome's finest stars, laid out once a process: the same sky every time. */
internal object SkySceneStatics {
    val layers: List<StarLayer> by lazy {
        val random = Random(29)
        listOf(StarLayer(random, 2600, -3000f, 2600f, 1900f, 1.0f, 0.6f), StarLayer(random, 900, -1000f, 1700f, 1000f, 1.25f, 0.5f), StarLayer(random, 160, 900f, 1500f, 520f, 1.6f, 0.2f))
    }
    val micro: MicroStars by lazy { MicroStars(Random(41)) }
}

/**
 * Draws the dome for [look] on [canvas], [width] by [height] pixels, with the dome turned [spin] radians. The shader is
 * expensive, so it is drawn small and scaled up, and only when the camera moves.
 */
internal fun drawDome(canvas: Canvas, shader: RuntimeShader, paint: Paint, width: Float, height: Float, look: Float, spin: Float) {
    shader.setFloatUniform("uSize", width, height)
    shader.setFloatUniform("uTan", Proto.TAN)
    shader.setFloatUniform("uPitch", look * Proto.PITCH)
    shader.setFloatUniform("uSpin", spin)
    shader.setFloatUniform("uTw", 1f - sstep(look, 0.35f, 0.95f))
    paint.shader = shader
    canvas.drawRect(0f, 0f, width, height, paint)
}

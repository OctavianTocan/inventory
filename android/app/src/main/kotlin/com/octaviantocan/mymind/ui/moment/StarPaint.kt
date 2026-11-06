package com.octaviantocan.mymind.ui.moment

import android.graphics.Bitmap
import android.graphics.BlendMode
import android.graphics.Canvas
import android.graphics.ComposeShader
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Path
import android.graphics.PorterDuff
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.SweepGradient
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.core.graphics.PathParser
import kotlin.math.PI
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin

/**
 * Paints the moment's two layers with Android's canvas, in the prototype's units: behind the card (the sky, hills,
 * constellation and the light of the hit) and in front of it (the star, sparks, embers and flashes). Light adds, as
 * the prototype's `lighter` canvas did.
 */
internal class StarPaint {
    private val sprites = HashMap<Int, Bitmap>()
    private val sprite = Paint(Paint.FILTER_BITMAP_FLAG or Paint.ANTI_ALIAS_FLAG).apply { blendMode = BlendMode.PLUS }
    private val add = Paint(Paint.ANTI_ALIAS_FLAG).apply { blendMode = BlendMode.PLUS }
    private val line = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeCap = Paint.Cap.ROUND; blendMode = BlendMode.PLUS }
    private val plain = Paint(Paint.ANTI_ALIAS_FLAG)
    private val plainLine = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeCap = Paint.Cap.ROUND; strokeJoin = Paint.Join.ROUND }
    private val rect = RectF()
    private val star = Path()
    private var skyFor = -1f
    private var sky: Shader? = null
    private val blooms = HashMap<Int, Shader>()
    private val rays = HashMap<Int, Pair<Shader, Shader>>()
    private val rayGlow = HashMap<Int, Shader>()
    private val nebulae = NEBULAE.map { n -> RadialGradient(0f, 0f, 1f, intArrayOf(n.color.toArgb(), n.color.copy(alpha = 0f).toArgb()), floatArrayOf(0f, 0.7f), Shader.TileMode.CLAMP) }
    private val hills = HILLS.map { PathParser.createPathFromPathData(it) }
    private val rims = RIMS.map { PathParser.createPathFromPathData(it) }

    /** Paints everything behind the card. */
    fun back(canvas: Canvas, sim: StarSim) {
        val w = sim.width; val h = sim.height; val stage = sim.stage
        if (stage <= 0f) return
        if (skyFor != h) { skyFor = h; sky = LinearGradient(0f, 0f, 0f, h, SKY, floatArrayOf(0f, 0.4f, 0.75f, 1f), Shader.TileMode.CLAMP) }
        plain.shader = sky; plain.alpha = alpha(sim.skyA * stage)
        // Past the edges, so the camera's shake never shows the page at a corner.
        canvas.drawRect(-40f, -40f, w + 40f, h + 40f, plain)
        plain.shader = null
        nebula(canvas, sim, stage)
        val glowA = sim.hzA * stage
        if (glowA > 0f) spriteAt(canvas, HORIZON, w / 2f, h + 40f, 600f, 300f, glowA)
        backHills(canvas, sim, stage)
        constellation(canvas, sim, stage)
        sim.particles.forEach { if (it.back) particle(canvas, it, sim.clock) }
        sim.rings.forEach { if (!it.front) ring(canvas, it) }
        sim.lights.forEach { light(canvas, it) }
        if (sim.raysA > 0f) rays(canvas, sim, stage)
        if (sim.cardOn && sim.glowA > 0f) {
            val s = sim.cardScale
            spriteAt(canvas, sim.shownRarity.color.toArgb(), sim.cardX, sim.cardY, 200f * s * 1.9f, 284f * s * 1.6f, min(1f, sim.glowA * 0.7f) * stage)
        }
    }

    /** Paints everything in front of the card. */
    fun front(canvas: Canvas, sim: StarSim) {
        if (sim.cometOn) comet(canvas, sim.cometTrail(), sim.cometColor.toArgb(), sim.cometW, 1f, sim.clock)
        val trail = sim.flightTrail()
        if (trail.isNotEmpty()) comet(canvas, trail, sim.shownRarity.color.toArgb(), 3f, 1f, sim.clock)
        sim.particles.forEach { if (!it.back) particle(canvas, it, sim.clock) }
        sim.rings.forEach { if (it.front) ring(canvas, it) }
        sim.flashes.forEach { f ->
            val p = f.t / f.seconds
            plain.color = android.graphics.Color.WHITE; plain.alpha = alpha(f.alpha * (1 - p) * (1 - p) * sim.stage)
            canvas.drawRect(-40f, -40f, sim.width + 40f, sim.height + 40f, plain)
        }
    }

    private fun nebula(canvas: Canvas, sim: StarSim, stage: Float) {
        val a = sim.nebA * stage
        if (a <= 0f) return
        // The prototype's nebula layer is 12% larger than the screen on each side and drifts over nine seconds.
        val bw = sim.width * 1.24f; val bh = sim.height * 1.24f
        val d = sim.nebDrift
        canvas.save()
        canvas.translate(sim.width / 2f - 14f * d, sim.height / 2f + 8f * d)
        canvas.scale(1f + 0.06f * d, 1f + 0.06f * d)
        canvas.translate(-bw / 2f, -bh / 2f)
        NEBULAE.forEachIndexed { i, n ->
            canvas.save()
            canvas.translate(n.x * bw, n.y * bh)
            canvas.scale(n.rx * bw, n.ry * bh)
            plain.shader = nebulae[i]; plain.alpha = alpha(a)
            canvas.drawCircle(0f, 0f, 1f, plain)
            canvas.restore()
        }
        plain.shader = null
        canvas.restore()
    }

    private fun backHills(canvas: Canvas, sim: StarSim, stage: Float) {
        val a = sim.hillsA * stage
        if (a <= 0f) return
        canvas.save()
        canvas.translate(0f, sim.height - 230f + sim.hillsDy)
        canvas.scale(sim.width / 360f, 1f)
        val rim = sim.shownRarity.color.toArgb()
        hills.forEachIndexed { i, path ->
            plain.color = HILL_FILLS[i]; plain.alpha = alpha(a)
            canvas.drawPath(path, plain)
            val r = sim.rimA[i]
            if (r > 0f) {
                plainLine.color = rim; plainLine.alpha = alpha(r * a); plainLine.strokeWidth = if (i == 2) 2f else 1.5f
                canvas.drawPath(rims[i], plainLine)
            }
        }
        canvas.restore()
    }

    private fun constellation(canvas: Canvas, sim: StarSim, stage: Float) {
        val a = sim.conA * stage
        if (a <= 0f) return
        val p = sim.conPts
        val segments = listOf(p[0] to p[1], p[1] to p[2], p[2] to p[3], p[3] to p[0], p[3] to p[1], p[4] to floatArrayOf(p[0][0], p[3][1]), floatArrayOf(p[0][0], p[3][1]) to p[5])
        val total = segments.sumOf { (s, e) -> hypot(e[0] - s[0], e[1] - s[1]).toDouble() }.toFloat()
        var left = total * sim.conDraw
        plainLine.color = StarSim.MINT.toArgb(); plainLine.alpha = alpha(0.55f * a); plainLine.strokeWidth = 1f
        for ((s, e) in segments) {
            if (left <= 0f) break
            val len = hypot(e[0] - s[0], e[1] - s[1])
            val k = min(1f, left / len)
            canvas.drawLine(s[0], s[1], s[0] + (e[0] - s[0]) * k, s[1] + (e[1] - s[1]) * k, plainLine)
            left -= len
        }
        plain.color = 0xFFF4F2EF.toInt(); plain.alpha = alpha(a)
        p.forEach { (x, y) -> canvas.drawCircle(x, y, 2f, plain) }
    }

    private fun comet(canvas: Canvas, trail: List<FloatArray>, color: Int, w: Float, a: Float, clock: Float) {
        val n = trail.size
        if (n < 2) return
        line.color = color
        for (i in 1 until n) {
            val q = i.toFloat() / n
            line.alpha = alpha(q.pow(1.6f) * a); line.strokeWidth = w * q * 2.4f
            canvas.drawLine(trail[i - 1][0], trail[i - 1][1], trail[i][0], trail[i][1], line)
        }
        line.color = android.graphics.Color.WHITE
        for (i in 1 until n) {
            val q = i.toFloat() / n
            line.alpha = alpha(q * q * a); line.strokeWidth = w * q * 0.7f
            canvas.drawLine(trail[i - 1][0], trail[i - 1][1], trail[i][0], trail[i][1], line)
        }
        val (x, y) = trail.last()
        spriteAt(canvas, color, x, y, w * 14f, w * 14f, a)
        add.color = android.graphics.Color.WHITE; add.alpha = alpha(a)
        star4(canvas, x, y, w * 3.2f * (0.85f + 0.15f * sin(clock * 40f)), add)
    }

    private fun particle(canvas: Canvas, p: StarSim.Particle, clock: Float) {
        if (p.t < 0f) return
        val k = 1f - p.t / p.life
        when (p.kind) {
            StarSim.Particle.SPARK -> {
                val sp = hypot(p.vx, p.vy).takeIf { it > 0f } ?: 1f
                val len = min(36f, sp * 0.045f + 1.5f)
                val a = min(1f, k * 1.6f)
                line.color = p.color; line.alpha = alpha(a); line.strokeWidth = p.size * (0.4f + 0.6f * k)
                canvas.drawLine(p.x - p.vx / sp * len, p.y - p.vy / sp * len, p.x, p.y, line)
                spriteAt(canvas, p.color, p.x, p.y, p.size * 5f, p.size * 5f, a * 0.55f)
            }
            StarSim.Particle.GLINT -> {
                val twinkle = max(0f, sin(PI.toFloat() * p.t / p.life))
                val a = twinkle * (0.75f + 0.25f * sin(clock * 22f + p.ph))
                add.color = android.graphics.Color.WHITE; add.alpha = alpha(a)
                star4(canvas, p.x, p.y, p.size * 3.4f * a + 0.4f, add)
                spriteAt(canvas, p.color, p.x, p.y, p.size * 9f * a, p.size * 9f * a, a)
            }
            else -> {
                val a = min(1f, k * 1.5f) * (0.55f + 0.45f * sin(clock * 11f + p.ph))
                spriteAt(canvas, p.color, p.x, p.y, p.size * 6f, p.size * 6f, a)
            }
        }
    }

    private fun ring(canvas: Canvas, r: StarSim.Ring) {
        if (r.t < 0f) return
        val p = r.t / r.seconds
        val e = 1f - (1f - p).pow(3)
        val radius = 6f + e * r.max
        line.color = r.color; line.alpha = alpha((1 - p) * 0.35f); line.strokeWidth = r.w * 3f * (1 - p) + 1f
        canvas.drawCircle(r.x, r.y, radius, line)
        line.alpha = alpha((1 - p) * 0.95f); line.strokeWidth = r.w * (1 - p) + 0.6f
        canvas.drawCircle(r.x, r.y, radius, line)
        line.color = android.graphics.Color.WHITE; line.alpha = alpha((1 - p) * 0.7f); line.strokeWidth = max(0.6f, r.w * 0.3f * (1 - p))
        canvas.drawCircle(r.x, r.y, radius, line)
    }

    private fun light(canvas: Canvas, l: StarSim.Light) {
        val k = 1f - l.t / l.life
        when (l) {
            is StarSim.Light.Flare -> {
                val a = l.a * k; val c = l.color.toArgb()
                spriteRect(canvas, c, l.x - l.len, l.y - l.th, l.len * 2f, l.th * 2f, a)
                spriteRect(canvas, android.graphics.Color.WHITE, l.x - l.len * 0.55f, l.y - l.th * 0.3f, l.len * 1.1f, l.th * 0.6f, a)
                spriteAt(canvas, c, l.x, l.y, l.core, l.core, a)
            }
            is StarSim.Light.Bloom -> {
                val p = 1f - k
                val a = l.a * k * k
                val radius = l.radius * (0.6f + 0.4f * p)
                val c = l.color.toArgb()
                add.shader = blooms.getOrPut(c) {
                    RadialGradient(0f, 0f, 1f, intArrayOf(android.graphics.Color.WHITE, l.color.copy(alpha = 0.75f).toArgb(), l.color.copy(alpha = 0f).toArgb()),
                        floatArrayOf(0f, 0.25f, 1f), Shader.TileMode.CLAMP)
                }
                add.alpha = alpha(a)
                canvas.save(); canvas.translate(l.x, l.y); canvas.scale(radius, radius)
                canvas.drawCircle(0f, 0f, 1f, add)
                canvas.restore()
                add.shader = null
            }
        }
    }

    /** Light rays behind a Rare or Legendary card, two sets turning against each other. */
    private fun rays(canvas: Canvas, sim: StarSim, stage: Float) {
        val c = sim.shownRarity.color
        val (a, b) = rays.getOrPut(c.toArgb()) { raysOf(c) }
        val glow = rayGlow.getOrPut(c.toArgb()) {
            RadialGradient(0f, 0f, 0.44f, intArrayOf(c.copy(alpha = 0.6f).toArgb(), c.copy(alpha = 0f).toArgb()), floatArrayOf(0f, 0.7f), Shader.TileMode.CLAMP)
        }
        val radius = 350f * sim.raysScale
        val alpha = alpha(sim.raysA * stage)
        canvas.save()
        canvas.translate(sim.cardX, sim.cardY)
        canvas.scale(radius, radius)
        add.alpha = alpha
        canvas.save(); canvas.rotate(sim.clock * 360f / 26f); add.shader = a; canvas.drawCircle(0f, 0f, 1f, add); canvas.restore()
        canvas.save(); canvas.rotate(-sim.clock * 360f / 34f); add.shader = b; canvas.drawCircle(0f, 0f, 1f, add); canvas.restore()
        add.shader = glow; canvas.drawCircle(0f, 0f, 0.44f, add)
        add.shader = null
        canvas.restore()
    }

    private fun raysOf(c: Color): Pair<Shader, Shader> {
        fun spokes(period: Int, from: Float, to: Float, color: Int): Shader {
            val colors = ArrayList<Int>(); val stops = ArrayList<Float>()
            for (i in 0 until period) {
                val base = i.toFloat() / period
                val span = 1f / period
                colors += 0; stops += base
                colors += 0; stops += base + span * from * 0.8f
                colors += color; stops += base + span * from
                colors += color; stops += base + span * to
                colors += 0; stops += base + span * (to + (1f - to) * 0.2f)
            }
            colors += 0; stops += 1f
            return SweepGradient(0f, 0f, colors.toIntArray(), stops.toFloatArray())
        }
        fun mask(inner: Float, peak: Float, outer: Float) = RadialGradient(0f, 0f, 1f, intArrayOf(0, 0, android.graphics.Color.BLACK, 0),
            floatArrayOf(0f, inner, peak, outer), Shader.TileMode.CLAMP)
        val first = ComposeShader(spokes(15, 10f / 24f, 13f / 24f, c.copy(alpha = 0.3f).toArgb()), mask(0.14f, 0.26f, 1f), PorterDuff.Mode.DST_IN)
        val second = ComposeShader(spokes(12, 15f / 30f, 17f / 30f, Color.White.copy(alpha = 0.14f).toArgb()), mask(0.12f, 0.22f, 0.9f), PorterDuff.Mode.DST_IN)
        return first to second
    }

    private fun star4(canvas: Canvas, x: Float, y: Float, r: Float, paint: Paint) {
        star.rewind()
        star.moveTo(x, y - r)
        star.quadTo(x, y, x + r, y); star.quadTo(x, y, x, y + r); star.quadTo(x, y, x - r, y); star.quadTo(x, y, x, y - r)
        canvas.drawPath(star, paint)
    }

    private fun spriteAt(canvas: Canvas, color: Int, x: Float, y: Float, w: Float, h: Float, a: Float) = spriteRect(canvas, color, x - w / 2f, y - h / 2f, w, h, a)

    private fun spriteRect(canvas: Canvas, color: Int, x: Float, y: Float, w: Float, h: Float, a: Float) {
        if (a <= 0.004f || w <= 0f || h <= 0f) return
        rect.set(x, y, x + w, y + h)
        sprite.alpha = alpha(a)
        canvas.drawBitmap(spriteOf(color), null, rect, sprite)
    }

    /** A soft round light: a white core into the colour into nothing, as the prototype's sprite. */
    private fun spriteOf(color: Int): Bitmap = sprites.getOrPut(color) {
        val c = Color(color)
        Bitmap.createBitmap(64, 64, Bitmap.Config.ARGB_8888).also { bitmap ->
            val paint = Paint().apply {
                shader = RadialGradient(32f, 32f, 32f, intArrayOf(android.graphics.Color.WHITE, c.copy(alpha = 0.95f).toArgb(), c.copy(alpha = 0.32f).toArgb(), c.copy(alpha = 0f).toArgb()),
                    floatArrayOf(0f, 0.16f, 0.42f, 1f), Shader.TileMode.CLAMP)
            }
            Canvas(bitmap).drawRect(0f, 0f, 64f, 64f, paint)
        }
    }

    private fun alpha(a: Float) = (a.coerceIn(0f, 1f) * 255f).toInt()

    private class Nebula(val x: Float, val y: Float, val rx: Float, val ry: Float, val color: Color)

    private companion object {
        val SKY = intArrayOf(0xFF020A0F.toInt(), 0xFF06181D.toInt(), 0xFF0D2E34.toInt(), 0xFF1E4E55.toInt())
        val HORIZON = 0xFF2D6A72.toInt()
        val NEBULAE = listOf(
            Nebula(0.28f, 0.28f, 0.38f, 0.26f, Color(63, 122, 130).copy(alpha = 0.45f)),
            Nebula(0.74f, 0.20f, 0.30f, 0.22f, Color(110, 80, 130).copy(alpha = 0.32f)),
            Nebula(0.60f, 0.52f, 0.46f, 0.26f, Color(30, 78, 85).copy(alpha = 0.40f)),
            Nebula(0.40f, 0.46f, 0.20f, 0.12f, Color(167, 211, 204).copy(alpha = 0.12f)),
        )
        val HILLS = listOf(
            "M0 110 C40 92 70 104 104 86 C140 66 172 96 212 80 C252 64 292 90 360 70 V230 H0Z",
            "M0 150 C50 128 90 146 140 124 C190 104 230 136 280 120 C320 108 340 118 360 112 V230 H0Z",
            "M0 192 C60 170 110 186 170 168 C230 150 280 178 360 160 V230 H0Z",
        )
        val RIMS = listOf(
            "M0 110 C40 92 70 104 104 86 C140 66 172 96 212 80 C252 64 292 90 360 70",
            "M0 150 C50 128 90 146 140 124 C190 104 230 136 280 120 C320 108 340 118 360 112",
            "M0 192 C60 170 110 186 170 168 C230 150 280 178 360 160",
        )
        val HILL_FILLS = intArrayOf(0xFF0A2226.toInt(), 0xFF071A1D.toInt(), 0xFF040E10.toInt())
    }
}

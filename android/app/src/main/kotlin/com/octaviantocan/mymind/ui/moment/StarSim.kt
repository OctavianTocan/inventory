package com.octaviantocan.mymind.ui.moment

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import java.util.PriorityQueue
import java.util.Random
import kotlin.math.PI
import kotlin.math.ceil
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.sqrt

/** A beat of the moment that is heard and felt. */
sealed interface Cue {
    /** Fourteen high glints of sound over a second. */
    data object Shimmer : Cue

    /** A bell for the shooting star breaking off ([step] 0) and for each colour it turns. */
    data class Ting(val step: Int) : Cue

    /** The star's long fall, as long as its flight. */
    data class Meteor(val seconds: Float) : Cue

    /** Rising noise and two climbing tones under the flight. */
    data class Riser(val seconds: Float) : Cue

    /** Air drawn in just before the landing. */
    data object Suck : Cue

    /** The card thrown up out of view. */
    data object Swoop : Cue

    /** The card landing. */
    data object Thud : Cue

    /** The hit as the card's face comes round, from 0.3 for Common to 1 for Legendary. */
    data class Boom(val power: Float) : Cue

    /** The chord for the find, longer and fuller for rarer ones. */
    data class Chime(val rarity: Rarity) : Cue

    /** One star stamped on the card, climbing with [index]. */
    data class Star(val index: Int) : Cue

    /** A step of the points counting up. */
    data object Count : Cue

    /** The find landing in the backpack. */
    data object Pop : Cue

    /** The page coming back at once after a tap. */
    data object Whoosh : Cue
}

/** Where the moment is. */
enum class StarPhase {
    /** The sky, the star and the reveal are playing; a tap skips to the backpack. */
    Playing,

    /** The card is face up with its stars and points; the tag step may open. */
    Settled,

    /** The find is flying into the backpack over the page. */
    Leaving,

    /** Nothing is left on screen. */
    Done,
}

/**
 * The shooting star moment (Tavi's pick of 03, 5 November 2026) as a simulation in moment time, so it plays the same at
 * any frame rate, under a test's fixed steps, and through hit-stops. Positions are in units of a 360-wide phone, as in
 * the prototype; [width] and [height] are the screen in those units, the story centred on it.
 *
 * @param width The screen's width in units, at least 360 on a phone.
 * @param height The screen's height in units.
 * @param top The status bar's height in units, for the backpack.
 * @param seed Seed for the particles, fixed in tests.
 * @param onCue Called for each beat that is heard and felt, on the thread that steps the moment.
 */
internal class StarSim(val width: Float, val height: Float, private val top: Float, seed: Long, private val onCue: (Cue) -> Unit) {
    private val random = Random(seed)
    val ox = (width - 360f) / 2f
    val oy = height * CARD_AT - 362f

    /** The card's resting centre. */
    val cx = 180f + ox
    val cy = 362f + oy

    /** Bumped on every step, for drawing to follow. */
    var frame by mutableLongStateOf(0L)
        private set
    var phase by mutableStateOf(StarPhase.Playing)
        private set

    private var known by mutableStateOf<Rarity?>(null)
    private var rarityKept = false

    /** The find's rarity, set once the library is counted; Common if it is not known when the star breaks off, and kept from then on. */
    var rarity: Rarity?
        get() = known
        set(value) { if (!rarityKept) known = value }
    val shownRarity: Rarity get() = rarity ?: Rarity.Common

    /** Moment time in milliseconds, which stops during a hit-stop. */
    var time = 0f
        private set
    private var hold = 0f
    var clock = 0f
        private set
    private var trauma = 0f

    private class Timed(val at: Float, val order: Int, val run: () -> Unit)
    private var orders = 0
    private val timeline = PriorityQueue<Timed>(compareBy<Timed>({ it.at }, { it.order }))
    private fun at(ms: Float, run: () -> Unit) { timeline += Timed(ms, orders++, run) }

    val particles = ArrayList<Particle>()
    val lights = ArrayList<Light>()
    val rings = ArrayList<Ring>()
    val flashes = ArrayList<Flash>()
    private val emitters = ArrayList<Emitter>()

    // The sky and its parts, as the prototype's backdrop, nebulae, hills and horizon glow.
    var skyA = 0f; private set
    var nebA = 0f; private set
    var nebDrift = 0f; private set
    var hillsA = 0f; private set
    var hillsDy = 40f; private set
    var hzA = 0f; private set
    val rimA = FloatArray(3)
    private var rimAt = -1f

    /** What is left of the sky and its light while the page comes back, from 1 to 0. */
    var stage = 1f; private set
    private var stageGoneAt = -1f
    private var stageGoneMs = 160f

    // The camera's push and shake.
    var camScale = 1f; private set
    var camY = 0f; private set
    var shakeX = 0f; private set
    var shakeY = 0f; private set
    var shakeRot = 0f; private set
    private var camFrom = floatArrayOf(1f, 0f)
    private var camTo = floatArrayOf(1.02f, -10f)
    private var camStart = 0f
    private var camMs = 600f
    private var camEase = CAM_IN

    // The constellation in the shape of the Inventory gem.
    var conDraw = 0f; private set
    var conA = 0f; private set
    val conPts = CONSTELLATION.map { (x, y) -> floatArrayOf(x + ox, y + oy) }

    // The shooting star.
    var cometOn = false; private set
    var cometX = 0f; private set
    var cometY = 0f; private set
    var cometColor = StarWhite; private set
    var cometW = 3.4f; private set
    private var launchAt = -1f
    private var flightMs = 0f
    private var marks = FloatArray(0)
    private var marksHit = 0
    private var suckDone = false
    private var landAt = -1f

    // The card.
    var cardOn by mutableStateOf(false)
        private set
    var cardX = cx; private set
    var cardY = cy; private set
    var cardScale = 0.55f; private set
    var cardRx = 0f; private set
    var cardRy = 0f; private set
    var cardFlip = PI.toFloat(); private set
    var cardRz = 0f; private set
    var cardA = 1f; private set
    var faceAt = -1f; private set
    private var backSeen = false
    var glowA = 0f; private set
    var raysA = 0f; private set
    var raysScale = 0.4f; private set
    var sheen = -1f; private set
    var sheenA = 0f; private set
    private val kicks = ArrayList<FloatArray>()
    private var settleAt = -1f
    /** The card's turn for the tag step in degrees: 0 face up, 180 showing the tag grid, 360 face up again. */
    var turnDeg = 0f; private set
    private var turnAt = -1f
    private var turnBack = true
    private var turnPose = FloatArray(4)

    /** Stars stamped on the card so far. */
    var stars by mutableIntStateOf(0)
        private set

    // The points under the card.
    var pointsOn by mutableStateOf(false)
        private set
    var pointsValue by mutableIntStateOf(0)
        private set
    var pointsA = 0f; private set
    var pointsScale = 1f; private set
    var pointsDy = 0f; private set
    private var pointsAt = -1f
    private var pointsPopAt = -1f
    private var pointsOutAt = -1f

    // The flight into the backpack, and the backpack.
    private var flightAt = -1f
    private var flightFrom = FloatArray(4)
    private var skipAt = -1f
    private var abortAt = -1f
    var bagOn by mutableStateOf(false)
        private set
    var bagA = 0f; private set
    var bagDy = 0f; private set
    var bagScale = 1f; private set
    var bagCounted by mutableStateOf(false)
        private set
    var bagLabel = 0f; private set
    val bagX get() = width - 14f - 24f
    val bagY get() = top + 10f + 24f
    private var bagAt = -1f
    private var popAt = -1f

    /** True once the moment was skipped by a tap. */
    val skipped get() = skipAt >= 0f

    init {
        at(0f) { emitters += Emitter(18f, 60_000f) { twinkle(1.2f, 2.6f, 0f) } }
        at(0f) { repeat(26) { twinkle(1.1f, 2.4f, 0.6f) } }
        at(420f) {
            onCue(Cue.Shimmer)
            conPts.forEachIndexed { i, (x, y) -> glints(x, y, MINT, 1, 1f, life = 1.4f, size = 1.8f, delay = i * 0.08f) }
        }
        at(1180f) { launch() }
    }

    /**
     * Moves the moment on.
     *
     * @param realMs Milliseconds since the last step, kept under 50 so a stall does not jump the story.
     */
    fun step(realMs: Float) {
        if (phase == StarPhase.Done) return
        var ms = min(realMs, 50f)
        if (hold > 0f) {
            val held = min(hold, ms)
            hold -= held
            ms -= held
        }
        val dt = ms / 1000f
        time += ms
        clock += dt
        while (timeline.isNotEmpty() && timeline.peek()!!.at <= time) timeline.poll()!!.run()
        runEmitters(ms)
        stepParticles(dt)
        stepStage()
        stepComet()
        stepCard(dt)
        stepPoints()
        stepFlight()
        trauma = max(0f, trauma - dt * 1.5f)
        val s = trauma * trauma
        shakeX = 16f * s * nz(clock * 34f)
        shakeY = 16f * s * nz(clock * 31f + 7f)
        shakeRot = 2.4f * s * nz(clock * 23f + 3f)
        val back = turnDeg % 360f in 90f..270f
        if (back != showingBack) showingBack = back
        frame++
    }

    /** A tap while the moment plays (Tavi, 5 November 2026): the page comes back at once and the find flies home. */
    fun skip() {
        if (phase != StarPhase.Playing || skipAt >= 0f) return
        keepRarity()
        skipAt = time
        timeline.clear()
        clearLight()
        goneStage(150f)
        onCue(Cue.Whoosh)
        cometOn = false
        cardOn = true
        stars = shownRarity.stars
        pointsOn = false
        cardFlip = 2f * PI.toFloat()
        cardRx = 0f; cardRy = 0f
        cardX = cx; cardY = cy + 17.5f
        val c = shownRarity.color
        at(time + 300f) {
            ring(cardX, cardY, c, 110f, 4f, 0.45f)
            sparks(cardX, cardY, c, 26, 240f, gravity = 120f, back = false)
            onCue(Cue.Chime(shownRarity))
        }
        at(time + 560f) { leave() }
    }

    /**
     * Turns the settled card over while it grows, for the tag grid on its back (Tavi's pick M9, 5 November 2026).
     *
     * @param y Where its centre goes, in units.
     * @param scale How big it gets, against its 284-unit height.
     */
    fun turnToBack(y: Float, scale: Float) {
        if (phase != StarPhase.Settled || turnAt >= 0f) return
        turnPose = floatArrayOf(cardY, cardScale, y, scale)
        turnAt = time; turnBack = true
        onCue(Cue.Whoosh)
    }

    /** Turns the card face up again at its own size, then flies it into the backpack. */
    fun turnToFace() {
        if (phase != StarPhase.Settled || turnAt < 0f || !turnBack) return
        turnPose = floatArrayOf(turnPose[2], turnPose[3], cy, 1f)
        turnAt = time; turnBack = false
        onCue(Cue.Whoosh)
        at(time + TURN_MS + 60f) { leave() }
    }

    /** True while the tag grid is the side of the card that shows. */
    var showingBack by mutableStateOf(false)
        private set

    /** Flies the card into the backpack while the page comes back. */
    fun leave() {
        if (phase == StarPhase.Leaving || phase == StarPhase.Done) return
        phase = StarPhase.Leaving
        flightAt = time
        flightFrom = floatArrayOf(cardX, cardY, cardScale, cardRz)
        turnDeg = 0f
        clearLight()
        goneStage(160f)
        bagOn = true
        bagAt = time
    }

    /** Ends without the backpack, when nothing was saved after all. */
    fun abort() {
        if (phase == StarPhase.Done || abortAt >= 0f) return
        abortAt = time
        timeline.clear()
        goneStage(300f)
        at(time + 300f) { phase = StarPhase.Done }
    }

    private fun keepRarity() { known = shownRarity; rarityKept = true }

    private fun launch() {
        keepRarity()
        val r = shownRarity
        val steps = r.tease
        launchAt = time
        flightMs = 1400f + 250f * (steps.size - 1)
        marks = when (steps.size) { 3 -> floatArrayOf(0.38f, 0.68f); 2 -> floatArrayOf(0.55f); else -> FloatArray(0) }
        cometOn = true
        cometColor = steps[0]
        cometX = conPts[0][0]; cometY = conPts[0][1]
        sparks(cometX, cometY, StarWhite, 16, 160f, gravity = 0f, back = false)
        ring(cometX, cometY, StarWhite, 40f, 2f, 0.4f)
        onCue(Cue.Ting(0))
        onCue(Cue.Meteor(flightMs / 1000f + 0.2f))
        onCue(Cue.Riser(flightMs / 1000f))
        emitters += Emitter(120f, flightMs + 100f) {
            sparks(cometX, cometY, cometColor, 2, 90f, gravity = 60f, life = 0.7f, drag = 1.5f, white = 0.5f, back = false)
            if (random.nextFloat() < 0.3f) glints(cometX, cometY, cometColor, 1, 6f, life = 0.6f)
        }
        landAt = launchAt + flightMs
        at(landAt) { land() }
    }

    /** Where the star is along its fall, from 0 to 1, at moment time [t]. */
    private fun cometU(t: Float): Float {
        val p = ((t - launchAt) / flightMs).coerceIn(0f, 1f)
        return 0.55f * p + 0.45f * p * p
    }

    /** The point of the star's curve at [u]. */
    fun cometAt(u: Float): FloatArray {
        val a = (1 - u) * (1 - u); val b = 2 * (1 - u) * u; val c = u * u
        return floatArrayOf(a * (180f + ox) + b * (470f + ox) + c * (180f + ox), a * (70f + oy) + b * (120f + oy) + c * (362f + oy))
    }

    /** The star's trail, newest last: where it was every 16 ms of moment time, for [n] points. */
    fun cometTrail(n: Int = 30): List<FloatArray> = (n - 1 downTo 0).map { k -> cometAt(cometU(time - k * 16f)) }

    private fun stepComet() {
        if (!cometOn) return
        val u = cometU(time)
        val (x, y) = cometAt(u)
        cometX = x; cometY = y
        while (marksHit < marks.size && u >= marks[marksHit]) {
            val i = marksHit++
            val c = shownRarity.tease[i + 1]
            cometColor = c
            cometW += 1f
            onCue(Cue.Ting(i + 1))
            kick(0.12f + 0.08f * i)
            lights += Light.Flare(x, y, c, 170f + 60f * i, 6f, 70f, 1f, 0.55f)
            ring(x, y, c, 70f + 30f * i, 4f, 0.5f)
            sparks(x, y, c, 24, 260f, gravity = 100f, back = false)
            lights += Light.Bloom(x, y, c, 0.4f, 260f, 0.5f)
        }
        if (!suckDone && u >= 0.9f) { suckDone = true; onCue(Cue.Suck) }
    }

    private fun land() {
        cometOn = false
        emitters.removeAll { it.rate == 120f }
        cardOn = true
        cardX = cx; cardY = cy; cardScale = 0.55f
        flash(0.6f, 0.45f)
        ring(cx, cy, StarWhite, 260f, 5f, 0.5f)
        camFrom = floatArrayOf(camScale, camY); camTo = floatArrayOf(1f, 0f); camStart = time; camMs = 900f; camEase = CAM_OUT
        onCue(Cue.Swoop)
    }

    private fun stepCard(dt: Float) {
        if (!cardOn) return
        val r = shownRarity
        if (skipAt >= 0f && phase == StarPhase.Playing) {
            val p = ((time - skipAt) / 300f).coerceIn(0f, 1f)
            val e = POP.transform(p)
            if (e < 0.7f) { val q = e / 0.7f; cardScale = 0.75f * lerp(0.4f, 1.08f, q); cardRz = lerp(-8f, 2f, q); cardA = q }
            else { val q = (e - 0.7f) / 0.3f; cardScale = 0.75f * lerp(1.08f, 1f, q); cardRz = lerp(2f, 0f, q); cardA = 1f }
            return
        }
        if (phase == StarPhase.Leaving) return
        val t = time - landAt
        val pi = PI.toFloat()
        when {
            t < 320f -> { val p = t / 320f; cardY = lerp(cy, -180f, p * p * p); cardRx = lerp(0f, 0.5f, p * p * p) }
            t < 740f -> {
                val p = (t - 320f) / 420f
                cardScale = 1f; cardX = cx
                cardY = lerp(-180f, cy, p * p); cardRx = lerp(0.5f, -0.15f, p)
            }
            t < 1120f -> {
                if (!thudDone) { thudDone = true; onCue(Cue.Thud); kick(0.3f); hitstop(50f) }
                val p = (t - 740f) / 380f
                cardY = cy - 46f * sin(pi * p)
                cardFlip = lerp(pi, 2f * pi, easeOut(p))
                cardRx = lerp(-0.15f, 0.1f, p)
            }
            t < 1380f -> { cardY = cy; cardFlip = 2f * pi; cardRx = lerp(0.1f, 0f, BACK.transform((t - 1120f) / 260f)) }
            else -> {
                cardFlip = 2f * pi
                if (settleAt < 0f) settle()
                val live = (time - settleAt) / 1000f
                val sway = min(1f, live / 0.8f)
                cardRx = sway * 0.05f * sin(live * 1.3f)
                cardRy = sway * 0.07f * sin(live * 0.9f + 0.6f)
            }
        }
        // Little pushes when stars land on the card, as the prototype's springs.
        var push = 0f
        kicks.removeAll { k -> val a = (time - k[0]) / 1000f; if (a > 1f) true else { push += k[1] * exp(-a * 7f) * cos(a * 26f); false } }
        cardRx += push
        val f = cos(cardFlip) * cos(cardRx)
        if (f < -0.2f) backSeen = true
        if (backSeen && faceAt < 0f && f > 0.3f) face(r)
        if (glowFrom >= 0f) {
            val p = ((time - glowFrom) / 1300f).coerceIn(0f, 1f)
            glowA = lerp(1.2f, 0.42f, easeOut(p))
        }
        if (raysFrom >= 0f) {
            val p = ((time - raysFrom) / 700f).coerceIn(0f, 1f)
            raysA = CARD_EASE.transform(p); raysScale = lerp(0.4f, 1f, CARD_EASE.transform(p))
        }
        if (sheenFrom >= 0f) {
            val p = (time - sheenFrom) / 1000f
            if (p in 0f..1f) { sheen = lerp(-0.4f, 1.9f, easeInOut(p)); sheenA = 0.55f } else if (p > 1f) sheenA = 0f
        }
        if (turnAt >= 0f) {
            val e = TURN.transform(((time - turnAt) / TURN_MS).coerceIn(0f, 1f))
            cardY = lerp(turnPose[0], turnPose[2], e); cardScale = lerp(turnPose[1], turnPose[3], e)
            turnDeg = if (turnBack) 180f * e else 180f + 180f * e
        }
        if (rimAt >= 0f) for (i in 0..2) {
            val p = ((time - rimAt - i * 60f) / 1600f).coerceIn(0f, 1f)
            val e = CARD_EASE.transform(p)
            rimA[i] = if (p <= 0f) 0f else if (e < 0.15f) e / 0.15f else lerp(1f, 0.35f, (e - 0.15f) / 0.85f)
        }
    }

    private var thudDone = false
    private var glowFrom = -1f
    private var raysFrom = -1f
    private var sheenFrom = -1f

    /** The hit as the card's face comes round: light, rings, sparks, a freeze and a shake, sized by rarity. */
    private fun face(r: Rarity) {
        faceAt = time
        val c = r.color
        val p = when (r) { Rarity.Legendary -> 1f; Rarity.Rare -> 0.62f; Rarity.Common -> 0.3f }
        onCue(Cue.Boom(p))
        lights += Light.Bloom(cardX, cardY, c, 0.75f, 260f + 180f * p, 0.5f + 0.4f * p)
        ring(cardX, cardY, StarWhite, 110f + 120f * p, 4f, 0.42f)
        ring(cardX, cardY, c, 160f + 200f * p, 9f, 0.7f, delay = 0.06f)
        sparks(cardX, cardY, c, (30 + 120 * p).toInt(), 360f + 460f * p, gravity = 280f, back = true)
        sparks(cardX, cardY, StarWhite, (12 + 30 * p).toInt(), 700f, gravity = 0f, life = 0.35f, drag = 3f, back = true)
        lights += Light.Flare(cardX, cardY, c, 150f + 160f * p, 7f + 6f * p, 90f + 60f * p, 1f, 0.5f + 0.5f * p)
        glints(cardX, cardY, c, (4 + 10 * p).toInt(), 130f, delay = 0.3f, back = true)
        kick(0.3f + 0.55f * p)
        hitstop(55f + 75f * p)
        glowFrom = time
        if (r != Rarity.Common) raysFrom = time
        at(time + 260f) { sheenFrom = time }
        flash(0.3f, 0.35f)
        rimAt = time
    }

    /** After the reveal: a chime, embers rising, the stars stamped one by one, then the points counting up. */
    private fun settle() {
        settleAt = time + 400f
        val r = shownRarity
        at(settleAt) {
            onCue(Cue.Chime(r))
            onCue(Cue.Shimmer)
            sheenFrom = time
            emitters += Emitter(if (r == Rarity.Legendary) 20f else 10f, 60_000f) { ember(r.color) }
        }
        for (i in 0 until r.stars) {
            at(settleAt + i * 140f) { stars = i + 1 }
            at(settleAt + i * 140f + 70f) {
                onCue(Cue.Star(i))
                val sx = cardX + (184f - (r.stars - 1 - i) * 13f - 100f) * cardScale
                val sy = cardY + (21f - 142f) * cardScale
                glints(sx, sy, r.color, 1, 1f)
                sparks(sx, sy, r.color, 7, 100f, gravity = 0f, life = 0.5f, back = false)
                ring(sx, sy, r.color, 18f, 2f, 0.3f, front = true)
                kicks += floatArrayOf(time, -0.06f)
            }
        }
        pointsAt = settleAt + r.stars * 140f
        val step = ceil(r.points / 8f).toInt()
        val values = (0..r.points step step).toList()
        values.forEachIndexed { i, v -> at(pointsAt + i * 32f) { pointsValue = v; onCue(Cue.Count) } }
        pointsPopAt = pointsAt + values.size * 32f
        at(pointsPopAt) { pointsValue = r.points }
        pointsOutAt = pointsPopAt + if (r == Rarity.Legendary) 700f else 450f
        at(pointsOutAt + 200f) { if (phase == StarPhase.Playing) phase = StarPhase.Settled }
    }

    private fun stepPoints() {
        if (pointsAt < 0f || time < pointsAt || phase == StarPhase.Leaving) { if (phase == StarPhase.Leaving) pointsOn = false; return }
        pointsOn = true
        val inP = ((time - pointsAt) / 220f).coerceIn(0f, 1f)
        val e = POINTS_IN.transform(inP)
        pointsA = inP.coerceAtMost(1f)
        pointsDy = 10f * (1 - e)
        pointsScale = lerp(0.9f, 1f, e)
        if (time >= pointsPopAt) {
            val p = ((time - pointsPopAt) / 200f).coerceIn(0f, 1f)
            pointsScale = lerp(1.18f, 1f, CARD_EASE.transform(p))
        }
        if (time >= pointsOutAt) {
            val p = ((time - pointsOutAt) / 200f).coerceIn(0f, 1f)
            pointsA = 1f - CARD_EASE.transform(p)
            if (p >= 1f) pointsOn = false
        }
    }

    private fun stepStage() {
        skyA = (time / 420f).coerceIn(0f, 1f).let(CARD_EASE::transform)
        nebA = (time / 900f).coerceIn(0f, 1f).let(CARD_EASE::transform)
        nebDrift = easeInOut((time / 9000f).coerceIn(0f, 1f))
        val h = (time / 800f).coerceIn(0f, 1f).let(CARD_EASE::transform)
        hillsA = h; hillsDy = 40f * (1 - h)
        hzA = 0.6f * easeOut((time / 600f).coerceIn(0f, 1f))
        if (stageGoneAt >= 0f) stage = 1f - ((time - stageGoneAt) / stageGoneMs).coerceIn(0f, 1f)
        val c = camEase.transform(((time - camStart) / camMs).coerceIn(0f, 1f))
        if (stageGoneAt >= 0f) { camScale = 1f; camY = 0f } else { camScale = lerp(camFrom[0], camTo[0], c); camY = lerp(camFrom[1], camTo[1], c) }
        conDraw = easeInOut(((time - 420f) / 700f).coerceIn(0f, 1f))
        conA = if (launchAt < 0f) (if (time >= 420f) 1f else 0f) else 1f - CARD_EASE.transform(((time - launchAt) / 600f).coerceIn(0f, 1f))
    }

    private fun stepFlight() {
        if (flightAt < 0f) return
        val r = shownRarity
        val t = time - flightAt
        val bx = bagX; val by = bagY
        if (bagOn) {
            val p = ((time - bagAt) / 240f).coerceIn(0f, 1f)
            val e = BAG_IN.transform(p)
            bagA = p; bagDy = -10f * (1 - e); bagScale = lerp(0.8f, 1f, e)
        }
        val (x0, y0, s0, rz0) = flightFrom
        if (t < 150f) {
            val e = LIFT.transform(t / 150f)
            cardX = x0; cardY = y0 - 14f * e; cardScale = s0 * (1f + 0.05f * e); cardRz = rz0 * (1 - e)
            return
        }
        if (t < 670f) {
            if (!swoopDone) { swoopDone = true; onCue(Cue.Swoop) }
            val pose = flightPose(t - 150f, x0, y0, s0, bx, by)
            cardX = pose[0]; cardY = pose[1]; cardScale = pose[2]; cardRz = pose[3]; cardA = pose[4]
            return
        }
        if (popAt < 0f) {
            popAt = time
            cardOn = false
            onCue(Cue.Pop)
            onCue(Cue.Shimmer)
            sparks(bx, by, r.color, 22, 180f, gravity = 0f, life = 0.5f, back = false)
            ring(bx, by, r.color, 46f, 3f, 0.4f, front = true)
            glints(bx, by, r.color, 4, 22f)
            bagCounted = true
        }
        val q = time - popAt
        bagScale = if (q < 280f) { val p = q / 280f; if (p < 0.5f) lerp(1f, 1.22f, easeOut(p * 2)) else lerp(1.22f, 1f, easeOut((p - 0.5f) * 2)) } else 1f
        bagLabel = CARD_EASE.transform(((q - 80f) / 260f).coerceIn(0f, 1f))
        if (q >= 1500f) {
            val p = ((q - 1500f) / 240f).coerceIn(0f, 1f)
            bagA = 1f - p; bagDy = -8f * p
            if (p >= 1f) phase = StarPhase.Done
        }
    }
    private var swoopDone = false

    /** The card's way into the backpack at [t] ms into its 520 ms flight: x, y, scale, turn and opacity. */
    private fun flightPose(t: Float, x0: Float, y0: Float, s0: Float, bx: Float, by: Float): FloatArray {
        val e = FLY.transform((t / 520f).coerceIn(0f, 1f))
        val dx = bx - x0; val dy = by - y0
        return if (e < 0.45f) {
            val q = e / 0.45f
            floatArrayOf(lerp(x0, x0 + dx * 0.35f, q), lerp(y0 - 14f, y0 + dy * 0.2f - 50f, q), s0 * lerp(1.05f, 0.55f, q), lerp(0f, 8f, q), 1f)
        } else {
            val q = (e - 0.45f) / 0.55f
            floatArrayOf(lerp(x0 + dx * 0.35f, bx, q), lerp(y0 + dy * 0.2f - 50f, by, q), s0 * lerp(0.55f, 0.1f, q), lerp(8f, 20f, q), lerp(1f, 0.5f, q))
        }
    }

    /** The trail behind the card on its way home, newest last, or none outside the flight. */
    fun flightTrail(n: Int = 16): List<FloatArray> {
        val t = time - flightAt - 150f
        if (flightAt < 0f || t <= 0f || t > 520f) return emptyList()
        val (x0, y0, s0) = flightFrom
        return (n - 1 downTo 0).map { k -> flightPose(max(0f, t - k * 16f), x0, y0, s0, bagX, bagY) }
    }

    private fun goneStage(ms: Float) { if (stageGoneAt < 0f) { stageGoneAt = time; stageGoneMs = ms } }

    private fun clearLight() {
        particles.clear(); lights.clear(); rings.clear(); flashes.clear(); emitters.clear()
        raysFrom = -1f; raysA = 0f; glowA = 0f; sheenA = 0f; trauma = 0f; hold = 0f
    }

    private fun hitstop(ms: Float) { if (skipAt < 0f) hold = max(hold, ms) }
    private fun kick(amount: Float) { trauma = min(1f, trauma + amount) }
    private fun flash(alpha: Float, seconds: Float) { flashes += Flash(alpha, seconds) }

    private fun ring(x: Float, y: Float, color: Color, max: Float, w: Float, seconds: Float, delay: Float = 0f, front: Boolean = false) {
        rings += Ring(x, y, color.toArgb(), max, w, seconds, -delay, front)
    }

    private fun rnd(a: Float, b: Float) = a + random.nextFloat() * (b - a)

    private fun sparks(
        x: Float, y: Float, color: Color, n: Int, speed: Float,
        gravity: Float = 300f, life: Float = 1f, drag: Float = 2.4f, white: Float = 0.35f, back: Boolean,
    ) {
        repeat(n) {
            val a = rnd(0f, 2f * PI.toFloat()); val v = rnd(speed * 0.3f, speed)
            val c = if (random.nextFloat() < white) StarWhite else color
            particles += Particle(Particle.SPARK, x, y, cos(a) * v, sin(a) * v, rnd(0.35f, 0.9f) * life, 0f, rnd(0.9f, 2.2f), c.toArgb(), gravity, drag, 0f, 0f, back)
        }
    }

    private fun glints(x: Float, y: Float, color: Color, n: Int, spread: Float, life: Float = 1f, size: Float = 2.2f, delay: Float = 0f, back: Boolean = false) {
        repeat(n) {
            particles += Particle(Particle.GLINT, x + rnd(-spread, spread), y + rnd(-spread, spread), 0f, 0f, rnd(0.5f, 1.1f) * life,
                -rnd(0f, delay), rnd(1f, size), color.toArgb(), 0f, 0f, rnd(0f, 6f), 0f, back)
        }
    }

    private fun twinkle(size: Float, life: Float, delay: Float) {
        particles += Particle(Particle.GLINT, rnd(10f, width - 10f), rnd(30f, cy + 118f), 0f, 0f, rnd(0.5f, 1.1f) * life,
            -rnd(0f, delay), rnd(1f, size), MINT.toArgb(), 0f, 0f, rnd(0f, 6f), 0f, true)
    }

    private fun ember(color: Color) {
        val c = if (random.nextBoolean()) color else StarWhite
        particles += Particle(Particle.EMBER, rnd(40f + ox, 320f + ox), rnd(cardY + 40f, min(height - 20f, cardY + 340f)), rnd(-10f, 10f), rnd(-75f, -25f),
            rnd(1.4f, 2.8f), 0f, rnd(0.7f, 1.7f), c.toArgb(), -4f, 0.3f, rnd(0f, 6f), rnd(1f, 3f), false)
    }

    private fun runEmitters(ms: Float) {
        val it = emitters.iterator()
        val due = ArrayList<() -> Unit>()
        while (it.hasNext()) {
            val e = it.next()
            e.t += ms; e.acc += ms / 1000f * e.rate
            while (e.acc > 1f) { due += e.spawn; e.acc -= 1f }
            if (e.t >= e.duration) it.remove()
        }
        due.forEach { it() }
    }

    private fun stepParticles(dt: Float) {
        if (dt <= 0f) return
        particles.removeAll { p ->
            p.t += dt
            if (p.t < 0f) return@removeAll false
            if (p.t >= p.life) return@removeAll true
            val d = exp(-p.drag * dt)
            p.vx *= d; p.vy *= d; p.vy += p.g * dt
            if (p.wob > 0f) p.vx += sin(clock * p.wob + p.ph) * 24f * dt
            p.x += p.vx * dt; p.y += p.vy * dt
            false
        }
        lights.removeAll { l -> l.t += dt; l.t >= l.life }
        rings.removeAll { r -> r.t += dt; r.t >= r.seconds }
        flashes.removeAll { f -> f.t += dt; f.t >= f.seconds }
    }

    /** One particle: a spark with a streak, a twinkling glint, or a rising ember. */
    class Particle(
        val kind: Int, var x: Float, var y: Float, var vx: Float, var vy: Float, val life: Float, var t: Float,
        val size: Float, val color: Int, val g: Float, val drag: Float, val ph: Float, val wob: Float, val back: Boolean,
    ) {
        companion object {
            const val SPARK = 0
            const val GLINT = 1
            const val EMBER = 2
        }
    }

    /** A light that fades over its [life] in seconds. */
    sealed class Light(val x: Float, val y: Float, val color: Color, val a: Float, val life: Float) {
        var t = 0f
        class Flare(x: Float, y: Float, color: Color, val len: Float, val th: Float, val core: Float, a: Float, life: Float) : Light(x, y, color, a, life)
        class Bloom(x: Float, y: Float, color: Color, a: Float, val radius: Float, life: Float) : Light(x, y, color, a, life)
    }

    /** A ring of light growing out from a point over [seconds]. */
    class Ring(val x: Float, val y: Float, val color: Int, val max: Float, val w: Float, val seconds: Float, var t: Float, val front: Boolean)

    /** The screen washed in white, fading over [seconds]. */
    class Flash(val alpha: Float, val seconds: Float) { var t = 0f }

    private class Emitter(val rate: Float, val duration: Float, val spawn: () -> Unit) { var t = 0f; var acc = 0f }

    companion object {
        /** Where the card rests, as a part of the screen's height from the top. */
        const val CARD_AT = 0.47f
        val MINT = Color(0xFFA7D3CC)
        val CONSTELLATION = listOf(180f to 70f, 222f to 104f, 180f to 176f, 138f to 104f, 160f to 88f, 200f to 88f)
        private val CAM_IN = CubicBezierEasing(0.3f, 0.7f, 0.3f, 1f)
        private val CAM_OUT = CubicBezierEasing(0.2f, 0.8f, 0.2f, 1f)
        private val CARD_EASE = CubicBezierEasing(0.2f, 0.8f, 0.2f, 1f)
        private val POP = CubicBezierEasing(0.2f, 0.9f, 0.3f, 1f)
        private val POINTS_IN = CubicBezierEasing(0.3f, 1.4f, 0.5f, 1f)
        private val BAG_IN = CubicBezierEasing(0.3f, 1.5f, 0.5f, 1f)
        private val LIFT = CubicBezierEasing(0.3f, 0f, 0.3f, 1f)
        private val FLY = CubicBezierEasing(0.45f, 0.05f, 0.55f, 0.95f)
        private val BACK = CubicBezierEasing(0.34f, 1.56f, 0.64f, 1f)
        private val TURN = CubicBezierEasing(0.3f, 1.25f, 0.45f, 1f)

        /** How long the card takes to turn over for the tag step, as M9's flip. */
        const val TURN_MS = 700f
    }
}

internal fun lerp(a: Float, b: Float, p: Float) = a + (b - a) * p
private fun easeOut(p: Float) = 1f - (1f - p).pow(3)
private fun easeInOut(p: Float) = if (p < 0.5f) 4f * p * p * p else 1f - (-2f * p + 2f).pow(3) / 2f
private fun nz(t: Float) = sin(t) * 0.5f + sin(t * 2.13f + 1.3f) * 0.3f + sin(t * 4.7f + 0.4f) * 0.2f
private operator fun FloatArray.component1() = this[0]
private operator fun FloatArray.component2() = this[1]
private operator fun FloatArray.component3() = this[2]
private operator fun FloatArray.component4() = this[3]

/** The time the star needs from breaking off to landing, for [rarity]. */
internal fun flightMsOf(rarity: Rarity) = 1400f + 250f * (rarity.tease.size - 1)

/** Solves the star's ease for the part of its flight at which it reaches [u] along its curve. */
internal fun flightPartAt(u: Float) = (-0.55f + sqrt(0.3025f + 1.8f * u)) / 0.9f

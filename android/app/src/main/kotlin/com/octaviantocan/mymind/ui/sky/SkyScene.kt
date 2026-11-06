package com.octaviantocan.mymind.ui.sky

import android.graphics.Bitmap
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.asAndroidPath
import androidx.compose.ui.graphics.vector.PathParser
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.sqrt
import kotlin.random.Random

/** A star of a constellation as the sky shows it: lit or dark, shown or still forming, and its pop when it changes. */
internal class StarState(val id: String, val seed: Float, val big: Int) {
    var lit = false
    var vis = 1f
    var pop = 0f
    var popT = -9f
}

/** A line between two stars; [from] is the end it draws from, [draw] how far it has drawn, [surge] where a pulse runs. */
internal class LineState(val a: Int, val b: Int) {
    var from = a
    var lit = 0f
    var draw = 1f
    var surge = -1f
    val to: Int get() = if (from == a) b else a
}

/**
 * One constellation in the sky, in the prototype's units: its stars at [px], [py] around its middle, which sits at
 * [x], [y], [z] on the slide at [scale].
 */
internal class ConState(val id: Long, val figure: SkyFigure, val name: String, val cards: List<String>, val done: Boolean) {
    val px = FloatArray(figure.stars.size) { (figure.stars[it].x - 100f) / 80f * Proto.SX }
    val py = FloatArray(figure.stars.size) { (100f - figure.stars[it].y) / 80f * Proto.SY }
    val stars: List<StarState> = cards.take(figure.stars.size).mapIndexed { i, card -> StarState(card, seedOf(card), i * 7 % 3) }
    val lines: List<LineState> = figure.edges.filter { (a, b) -> a < stars.size && b < stars.size }.map { (a, b) -> LineState(a, b) }
    var x = 0f
    var y = Proto.CY
    var z = 0f
    var scale = 1f
    var dim = 1f
    var lineVis = 1f
    var figOp = 0f

    /** How far the figure has drawn in, 0 to 1. */
    var figDraw = 0f

    /** Seconds the vortex it forms out of has run, or a negative number when there is none. */
    var vortex = -1f
    var vortexPoints: FloatArray? = null

    /** The figure's picture, drawn at [figRes] and [figShown] of the way in. */
    var figBitmap: Bitmap? = null
    var figRes = 0
    var figShown = -1f

    /** The figure's strokes in its own units, x and y from -1 to 1, y up. */
    val figPaths: List<android.graphics.Path> by lazy { figureStrokes(figure.art) }

    fun index(card: String): Int = stars.indexOfFirst { it.id == card }
}

/** A find outside every constellation still adds a star: a small one of its own beside it. */
internal class LooseStar(val id: String, val x: Float, val y: Float, val seed: Float, val t0: Float)

/** A constellation as the data gives it, for [SkyScene.show]. */
internal data class ConPlan(val id: Long, val figure: String, val name: String, val cards: List<String>, val lit: Set<String>, val done: Boolean)

/**
 * What the sky holds: the constellation in the middle, the finished ones in their places, the finds that light loose
 * stars, and the finds waiting to light in order.
 */
internal data class SkyPlan(
    val center: ConPlan? = null,
    val next: List<ConPlan> = emptyList(),
    val slots: List<ConPlan> = emptyList(),
    val loose: List<String> = emptyList(),
    val pending: List<String> = emptyList(),
)

/** What a star shows of its find when the camera is on it: its picture as smoke and the stars that condense into it. */
internal class FindView(val card: String, val title: String, val picture: Bitmap, val worn: Boolean) {
    var into = 0f
    var op = 1f
    var condOp = 1f
    var wornK = if (worn) 1f else 0f
    var count = 0
    val tx = FloatArray(MAX_CONDENSE)
    val ty = FloatArray(MAX_CONDENSE)
    val sx = FloatArray(MAX_CONDENSE)
    val sy = FloatArray(MAX_CONDENSE)
    val cr = FloatArray(MAX_CONDENSE)
    val cg = FloatArray(MAX_CONDENSE)
    val cb = FloatArray(MAX_CONDENSE)
    val delay = FloatArray(MAX_CONDENSE)
    val size = FloatArray(MAX_CONDENSE)
    val phase = FloatArray(MAX_CONDENSE)
    val aspect: Float = picture.width.toFloat() / picture.height.coerceAtLeast(1)

    companion object {
        const val MAX_CONDENSE = 4200
    }
}

/**
 * The prototype's sky ("Your sky", look 25 and Condense), run in its own units on [Timeline] time: the camera that
 * looks up from the page, the constellations, the shooting stars that light them, and a find condensing out of the
 * stars. [SkyLayer] draws it and passes it touches; the callbacks reach the app.
 */
internal class SkyScene {
    val time = Timeline()
    var scope: CoroutineScope? = null

    /** How far the camera has looked up from the page, 0 to 1. */
    var look by mutableFloatStateOf(0f)

    /** A camera shake from 0 to 1, which also moves the page. */
    var punch by mutableFloatStateOf(0f)

    /** True from looking up until back down; the sky then takes every touch. */
    var lookingUp by mutableStateOf(false)
        private set

    /** True once the camera has finished looking up: "Your sky" shows. */
    var skyOn by mutableStateOf(false)
        private set

    /** The pull at the top of the page, in units, and whether it has gone far enough to look up. */
    var pull by mutableFloatStateOf(0f)
        private set

    /** The find's title under the find in view, and whether it shows. */
    var memTitle by mutableStateOf("")
        private set
    var memOn by mutableStateOf(false)
        private set

    /** The focused star's find, or null. */
    var focusedCard by mutableStateOf<String?>(null)
        private set

    /** Counts frames, so what draws the sky draws again. */
    val frames = mutableIntStateOf(0)

    var busy = false
        private set
    var fz = 0f
    var zoomDip = 0f
    var pushZ = 0f
    var bloomKick = 0f
    var focusTx = 0f
    var focusTy = 0f
    var dragX = 0f
    var dragY = 0f

    /** The find view's offset while dragging, in screen units. */
    var viewDx = 0f
    var viewDy = 0f
    var upPull = 0f
    var focusedCon: ConState? = null
    var focusedIndex = -1
    var find: FindView? = null

    val cons = ArrayList<ConState>()
    var active: ConState? = null
    val loose = ArrayList<LooseStar>()

    /** The constellation's name under it: [labelName] typed to [labelTyped] characters, gold once [labelDone]. */
    var labelName = ""
    var labelTyped = 0
    var labelDone = 0f
    var labelOp = 1f

    // Shooting star.
    var ribbonOn = false
    val ribbon = FloatArray((SEG + 1) * 2 * 2)
    var ribbonOp = 0f
    var headX = 0f
    var headY = 0f
    var headSize = 0f
    var headSpike = 0f
    var headRot = 0f
    var headOp = 0f
    var fxX = 0f
    var fxY = 0f
    var flashOp = 0f
    var flashSize = 0f
    var streakOp = 0f
    var streakW = 0f
    var streakH = 0f
    var shockOp = 0f
    var shockP = 0f
    var shockSize = 0f
    var cometOp = 0f
    var cometX = 0f
    var cometY = 0f
    var cometSize = 0f
    var cometSpike = 0f
    var cometRot = 0f

    // Sparks: a pool of glows on the slide.
    val sparkX = FloatArray(SPARKS)
    val sparkY = FloatArray(SPARKS)
    val sparkVx = FloatArray(SPARKS)
    val sparkVy = FloatArray(SPARKS)
    val sparkT0 = FloatArray(SPARKS)
    val sparkD = FloatArray(SPARKS) { 1f }
    val sparkLive = BooleanArray(SPARKS)
    private var sparkNext = 0

    /** The dome's turn as the camera pans to a star, radians; changes only when it moves enough to see. */
    var domeSpin by mutableFloatStateOf(0f)
        private set

    /** This frame's camera shake, from -0.5 to 0.5 each way, shared by the sky and the page under it. */
    var jitterX = 0f
        private set
    var jitterY = 0f
        private set

    /** Counts sequences started, so a sleeping frame loop wakes for one. */
    val kicks = mutableIntStateOf(0)

    // Callbacks to the app.
    var onOpenFind: (String) -> Unit = {}
    var onSeen: (String) -> Unit = {}
    var onCompleted: () -> Unit = {}
    var onLookUp: () -> Unit = {}
    var picture: suspend (String) -> Bitmap? = { null }
    var title: (String) -> String = { "" }

    private var plan = SkyPlan()
    private var pendingPlan: SkyPlan? = null
    private var queue = ArrayList<String>()
    private var lightOnReturn: Pair<ConState, Int>? = null
    private val random = Random(17)

    /** Moves time on by [dt] real seconds and steps everything that moves. */
    fun frame(dt: Float) {
        val step = time.advance(dt.coerceAtMost(0.05f))
        punch = max(0f, punch - step * 3.2f)
        jitterX = if (punch > 0f) random.nextFloat() - 0.5f else 0f
        jitterY = if (punch > 0f) random.nextFloat() - 0.5f else 0f
        bloomKick *= 0.2f.pow(step)
        val now = time.now
        for (i in 0 until SPARKS) {
            if (!sparkLive[i]) continue
            val a = (now - sparkT0[i]) / sparkD[i]
            if (a >= 1f) {
                sparkLive[i] = false
                continue
            }
            sparkX[i] += sparkVx[i] * step
            sparkY[i] += sparkVy[i] * step
            sparkVy[i] -= 140f * step
        }
        if (!busy) pendingPlan?.let { pendingPlan = null; show(it) }
        val spin = (focusTx * fz + dragX) * 0.0001f
        if (abs(spin - domeSpin) > 0.00002f) domeSpin = spin
        frames.intValue++
    }

    /** True while something moves on its own, so frames must keep coming. */
    val moving: Boolean get() = time.running || look > 0f || punch > 0f || pull > 0f || sparkLive.any { it }

    /** Throws a spark from [x], [y] on the slide. */
    fun emitSpark(x: Float, y: Float, vx: Float, vy: Float) {
        val i = sparkNext++ % SPARKS
        sparkX[i] = x
        sparkY[i] = y
        sparkVx[i] = vx
        sparkVy[i] = vy
        sparkT0[i] = time.now
        sparkD[i] = 0.5f + random.nextFloat() * 0.5f
        sparkLive[i] = true
    }

    /**
     * Shows [next]: the constellation in the middle, the finished ones in their places and the loose stars. Waits while
     * a sequence runs. A new constellation in the middle forms out of a vortex while the sky shows.
     */
    fun show(next: SkyPlan) {
        if (busy) {
            pendingPlan = next
            return
        }
        val before = plan
        plan = next
        queue = ArrayList(next.pending)
        val keep = HashSet<Long>()
        next.slots.takeLast(SLOTS.size).forEachIndexed { index, spec ->
            keep += spec.id
            val con = conFor(spec)
            val (sx, sy, sz) = SLOTS[index]
            con.x = sx
            con.y = sy
            con.z = sz
            con.scale = SLOT_SCALE
            con.dim = 0.5f
            con.figOp = 1f
            con.figDraw = 1f
            con.stars.forEach { it.lit = true; it.vis = 1f }
            con.lines.forEach { it.lit = 1f; it.draw = 1f }
        }
        val center = next.center
        val previous = active
        if (center != null) {
            keep += center.id
            val fresh = cons.none { it.id == center.id }
            val con = conFor(center)
            con.x = 0f
            con.y = Proto.CY
            con.z = 0f
            con.scale = 1f
            con.dim = 1f
            con.figOp = 0f
            con.figDraw = 0f
            val shownLit = center.lit - next.pending.toSet()
            con.stars.forEach { it.lit = it.id in shownLit }
            con.lines.forEach { it.lit = if (con.stars[it.a].lit && con.stars[it.b].lit) 1f else 0f; it.draw = 1f }
            active = con
            if (fresh && skyOn && previous?.id != center.id) {
                con.stars.forEach { it.vis = 0f }
                con.lineVis = 0f
                labelName = con.name
                labelTyped = 0
                launch { busy = true; generate(con); busy = false }
            } else if (labelName != con.name || before.center?.id != center.id) {
                labelName = con.name
                labelTyped = con.name.length
            }
        } else {
            active = null
            labelName = ""
            labelTyped = 0
        }
        cons.removeAll { it.id !in keep }
        val seenLoose = next.loose.toHashSet()
        loose.removeAll { it.id !in seenLoose }
        for (id in next.loose) if (loose.none { it.id == id }) loose += looseFor(id, time.now - 1f)
    }

    private fun conFor(spec: ConPlan): ConState = cons.firstOrNull { it.id == spec.id && it.cards == spec.cards }
        ?: ConState(spec.id, figureOf(spec.figure), spec.name, spec.cards, spec.done).also { made -> cons.removeAll { it.id == spec.id }; cons += made }

    private fun looseFor(id: String, t0: Float): LooseStar {
        val r = Random(id.hashCode())
        val side = if (r.nextBoolean()) -1f else 1f
        return LooseStar(id, side * (178f + r.nextFloat() * 40f), Proto.CY - 120f + r.nextFloat() * 400f, r.nextFloat() * 6.28f, t0)
    }

    private fun launch(block: suspend () -> Unit): Job? {
        kicks.intValue++
        return scope?.launch { block() }
    }

    private fun tweenLater(dur: Float, ease: (Float) -> Float = Ease.io, fn: (Float) -> Unit): Job? = launch { time.tween(dur, ease, fn) }

    // Looking up and down.

    /** Follows a pull at the top of the page by [dy] screen units, down positive. */
    fun pullBy(dy: Float) {
        if (lookingUp || busy) return
        pull = max(0f, pull + dy)
        look = min(0.42f, pull / 520f)
    }

    /** Lets go of the pull: past the threshold the camera looks up, else the page settles back. */
    fun releasePull() {
        if (pull <= 0f) return
        val amount = pull
        pull = 0f
        if (amount > PULL_TO_LOOK) lookUp()
        else {
            val l0 = look
            tweenLater(0.35f, Ease.out) { look = l0 * (1f - it) }
        }
    }

    /** The camera pitches up from the page to the sky, then any finds waiting light their stars. */
    fun lookUp() {
        if (lookingUp || busy) return
        lookingUp = true
        busy = true
        pull = 0f
        onLookUp()
        launch {
            val l0 = look
            time.tween(2.6f * (1f - l0 * 0.7f), Ease.io) { look = l0 + (1f - l0) * it }
            skyOn = true
            busy = false
            if (queue.isNotEmpty()) igniteAll()
        }
    }

    /** Back down to the page, the find in view first going back into starlight. */
    fun lookDown() {
        if (!lookingUp || busy) return
        busy = true
        launch {
            if (focusedCon != null) unfocusNow(true)
            skyOn = false
            val l0 = look
            time.tween(2.6f * l0, Ease.io) { look = l0 * (1f - it) }
            lookingUp = false
            busy = false
        }
    }

    /** Back: from a star to the sky, from the sky to the page. */
    fun back() {
        if (busy) return
        if (focusedCon != null) unfocus() else lookDown()
    }

    // Lighting stars.

    private suspend fun igniteAll() {
        busy = true
        // Loose finds first, then each constellation in turn: one finished while the sky was away first shows in the
        // middle, lights and takes its place; then the next forms.
        val loosePending = queue.filter { id -> cons.none { con -> con.index(id) >= 0 } && plan.next.none { id in it.cards } }
        for (id in loosePending) {
            igniteLoose(id)
            onSeen(id)
            queue.remove(id)
            time.wait(0.25f)
        }
        val order = ArrayList<ConPlan>()
        plan.center?.let(order::add)
        order += plan.next
        for ((index, spec) in order.withIndex()) {
            val con = if (index == 0) active else null
            val current = con ?: run {
                val made = conFor(spec)
                made.x = 0f; made.y = Proto.CY; made.z = 0f; made.scale = 1f; made.dim = 1f; made.figOp = 0f; made.figDraw = 0f
                val shownLit = spec.lit - queue.toSet()
                made.stars.forEach { it.lit = it.id in shownLit; it.vis = 0f }
                made.lines.forEach { it.lit = if (made.stars[it.a].lit && made.stars[it.b].lit) 1f else 0f; it.draw = 1f }
                made.lineVis = 0f
                active = made
                labelName = made.name
                labelTyped = 0
                labelOp = 1f
                generate(made)
                made
            } ?: continue
            var lit = 0
            for (id in queue.toList()) {
                val i = current.index(id)
                if (i < 0 || current.stars[i].lit) continue
                ignite(current, i)
                onSeen(id)
                queue.remove(id)
                lit++
                time.wait(0.35f)
            }
            if (lit > 0 && current.stars.all { it.lit }) complete(current)
        }
        queue.clear()
        busy = false
    }

    private suspend fun shootTo(tx: Float, ty: Float, big: Float = 1f) {
        val fromX = tx + 520f + random.nextFloat() * 120f
        val fromY = ty + 700f
        val ctrlX = tx + 380f
        val ctrlY = ty + 160f
        fun bx(q: Float) = (1 - q) * (1 - q) * fromX + 2 * (1 - q) * q * ctrlX + q * q * tx
        fun by(q: Float) = (1 - q) * (1 - q) * fromY + 2 * (1 - q) * q * ctrlY + q * q * ty
        ribbonOn = true
        ribbonOp = 1f
        headOp = 1f
        var lastEmit = -1f
        time.tween(1.05f, Ease.lin) { p ->
            val q0 = Ease.io(p)
            headX = bx(q0)
            headY = by(q0)
            headSize = 70f + 40f * p
            headSpike = 90f + 60f * p
            headRot = p * 1.4f
            val tl = 0.42f * min(1f, p * 3f)
            for (k in 0..SEG) {
                val t = k / SEG.toFloat()
                val q = max(0f, q0 - (1f - t) * tl)
                val x = bx(q)
                val y = by(q)
                var nx = bx(min(1f, q + 0.002f)) - x
                var ny = by(min(1f, q + 0.002f)) - y
                val len = hypot(nx, ny).coerceAtLeast(1e-4f)
                nx /= len
                ny /= len
                val wd = 1.5f + 9f * t.pow(1.6f)
                ribbon[k * 4] = x - ny * wd
                ribbon[k * 4 + 1] = y + nx * wd
                ribbon[k * 4 + 2] = x + ny * wd
                ribbon[k * 4 + 3] = y - nx * wd
            }
            if (time.now - lastEmit > 0.025f) {
                lastEmit = time.now
                emitSpark(headX, headY, (random.nextFloat() - 0.3f) * 90f, (random.nextFloat() - 0.1f) * 90f)
            }
        }
        ribbonOn = false
        headOp = 0f
        time.hitStop = 0.09f
        fxX = tx
        fxY = ty
        repeat((26 * big).toInt()) {
            val a = random.nextFloat() * 6.28f
            val sp = 120f + random.nextFloat() * 260f
            emitSpark(tx, ty, cos(a) * sp, sin(a) * sp)
        }
        tweenLater(1.8f, Ease.lin) { p ->
            val k = if (p < 0.06f) p / 0.06f else (1f - (p - 0.06f) / 0.94f).pow(2)
            flashOp = k * 0.85f * big
            flashSize = 70f + 260f * k * big
            streakOp = k * 0.8f * big
            streakW = 720f * k * big + 80f
            streakH = 36f * k + 8f
            shockOp = (1f - p) * 0.6f * big
            shockP = Ease.out(p)
            shockSize = 480f * big
            bloomKick = max(bloomKick, k * 0.45f * big)
        }
    }

    private suspend fun ignite(con: ConState, i: Int) {
        shootTo(con.x + con.px[i] * con.scale, con.y + con.py[i] * con.scale)
        val star = con.stars[i]
        star.lit = true
        star.pop = 1f
        star.popT = time.now
        time.wait(0.3f)
        val edges = con.lines.filter { (it.a == i && con.stars[it.b].lit) || (it.b == i && con.stars[it.a].lit) }
        for (line in edges) {
            line.from = if (line.a == i) line.b else line.a
            line.lit = 1f
            line.draw = 0f
        }
        time.tween(0.75f, Ease.out) { p -> edges.forEach { it.draw = p } }
    }

    private suspend fun igniteLoose(id: String) {
        val star = looseFor(id, 0f)
        shootTo(star.x, star.y, 0.55f)
        loose += LooseStar(star.id, star.x, star.y, star.seed, time.now)
    }

    private suspend fun complete(con: ConState) {
        time.wait(0.45f)
        val used = HashSet<LineState>()
        val seen = hashSetOf(0)
        var level = listOf(0)
        val waves = ArrayList<List<Pair<LineState, Int>>>()
        while (level.isNotEmpty()) {
            val next = ArrayList<Int>()
            val wave = ArrayList<Pair<LineState, Int>>()
            for (a in level) for (line in con.lines) {
                if (line in used) continue
                val b = if (line.a == a) line.b else if (line.b == a) line.a else -1
                if (b < 0) continue
                used += line
                wave += line to a
                if (seen.add(b)) next += b
            }
            if (wave.isNotEmpty()) waves += wave
            level = next
        }
        con.stars.getOrNull(0)?.let { it.pop = 0.8f; it.popT = time.now }
        for (wave in waves) {
            for ((line, from) in wave) {
                val forward = line.from == from
                launch {
                    time.tween(0.42f, Ease.lin) { p -> line.surge = if (forward) p else 1f - p }
                    line.surge = -1f
                }
            }
            time.wait(0.3f)
            for ((line, from) in wave) {
                val b = if (line.a == from) line.b else line.a
                con.stars[b].pop = 0.8f
                con.stars[b].popT = time.now
            }
        }
        time.wait(0.25f)
        time.hitStop = 0.12f
        bloomKick = 0.6f
        labelOp = 1f
        tweenLater(1f, Ease.io) { labelDone = it }
        tweenLater(3.4f, Ease.lin) { p -> pushZ = 150f * Ease.sine(min(1f, p * 1.6f)) }
        tweenLater(2.4f, Ease.out) { p -> con.figDraw = p; con.figOp = min(1f, p * 1.5f) }
        launch {
            val t0 = time.now
            var n = 0
            time.tween(2.6f, Ease.lin) {
                while (n * 0.06f <= time.now - t0) {
                    val k = random.nextInt(con.stars.size)
                    emitSpark(con.x + con.px[k], con.y + con.py[k], (random.nextFloat() - 0.5f) * 40f, 60f + random.nextFloat() * 90f)
                    n++
                }
            }
        }
        time.wait(3.6f)
        time.tween(0.6f, Ease.io) { labelOp = 1f - it }
        val slotIndex = (plan.slots.size).coerceAtMost(SLOTS.lastIndex)
        val (sx, sy, sz) = SLOTS[slotIndex]
        val x0 = con.x
        val y0 = con.y
        val z0 = con.z
        time.tween(1.9f, Ease.io) { p ->
            con.x = mix(x0, sx, p)
            con.y = mix(y0, sy, p)
            con.z = mix(z0, sz, p)
            con.scale = 1f - 0.66f * p
            con.dim = 1f - 0.5f * p
            pushZ = 150f * (1f - p)
        }
        pushZ = 0f
        labelDone = 0f
        if (active === con) active = null
        plan = plan.copy(slots = plan.slots + ConPlan(con.id, con.figure.id, con.name, con.cards, con.cards.toSet(), true), center = null)
        onCompleted()
    }

    private suspend fun generate(con: ConState) {
        con.stars.forEach { it.vis = 0f }
        con.lineVis = 0f
        labelOp = 1f
        labelTyped = 0
        con.vortexPoints = vortexFor(con)
        con.vortex = 0f
        val appear = FloatArray(con.stars.size) { 1.4f + it * 0.18f + 0.95f }
        var shown = 0
        val t0 = time.now
        time.tween(3.6f, Ease.lin) {
            val t = time.now - t0
            con.vortex = t
            while (shown < appear.size && t >= appear[shown]) {
                val star = con.stars[shown]
                star.vis = 1f
                star.pop = 0.9f
                star.popT = time.now
                shown++
            }
        }
        con.vortex = -1f
        con.vortexPoints = null
        tweenLater(0.8f) { con.lineVis = it }
        val name = con.name
        time.tween(0.9f, Ease.lin) { p -> labelTyped = kotlin.math.ceil(p * name.length).toInt() }
    }

    // Focusing on a star.

    private fun targetFor(con: ConState, i: Int): Pair<Float, Float> {
        val yUp = (422f - Proto.SKY_STAR_Y) / 422f * (Proto.RC - 700f) * Proto.TAN
        return -(con.x + con.px[i]) to yUp - (con.y + con.py[i])
    }

    /** Moves the camera in on star [i] of the middle constellation and shows its find. */
    fun focusStar(i: Int) {
        if (busy) return
        val con = active ?: return
        if (focusedCon != null) {
            if (focusedIndex != i) glide(i)
            return
        }
        busy = true
        focusedCon = con
        focusedIndex = i
        focusedCard = con.stars[i].id
        val (tx, ty) = targetFor(con, i)
        focusTx = tx
        focusTy = ty
        dragX = 0f
        dragY = 0f
        val star = con.stars[i]
        star.pop = 0.9f
        star.popT = time.now
        tweenLater(0.6f) { labelOp = min(labelOp, 1f - it) }
        launch {
            tweenLater(0.95f, Ease.io) { fz = it }
            var view: FindView? = null
            val loading = scope?.launch { view = loadFind(con, i) }
            time.wait(0.45f)
            loading?.join()
            showFind(view ?: loadFind(con, i))
            busy = false
        }
    }

    private suspend fun loadFind(con: ConState, i: Int): FindView {
        val star = con.stars[i]
        val card = star.id
        val bitmap = picture(card) ?: findFigureArt("repo")
        return FindView(card, title(card), bitmap, worn = !star.lit)
    }

    private suspend fun showFind(view: FindView) {
        buildCondense(view)
        find = view
        memTitle = view.title
        view.into = 0f
        view.op = 1f
        view.condOp = 1f
        tweenLater(2f, Ease.out) { view.into = it }
        time.wait(0.6f)
        memOn = true
    }

    private suspend fun hideFind(quick: Boolean) {
        memOn = false
        val view = find ?: return
        val d = if (quick) 0.3f else 0.5f
        val o = view.op
        val oc = view.condOp
        time.tween(d, Ease.io) { q -> view.op = o * (1f - q); view.condOp = oc * (1f - q) }
        if (find === view) find = null
    }

    /** From a star back to the whole sky. */
    fun unfocus(quick: Boolean = false) {
        if (busy || focusedCon == null) return
        busy = true
        launch {
            unfocusNow(quick)
            busy = false
        }
    }

    private suspend fun unfocusNow(quick: Boolean) {
        if (focusedCon == null) return
        hideFind(quick)
        val f0 = fz
        val dx = dragX
        val dy = dragY
        val vx = viewDx
        val vy = viewDy
        time.tween(if (quick) 0.45f else 0.8f, Ease.io) { q ->
            fz = f0 * (1f - q)
            dragX = dx * (1f - q)
            dragY = dy * (1f - q)
            viewDx = vx * (1f - q)
            viewDy = vy * (1f - q)
        }
        focusedCon = null
        focusedIndex = -1
        focusedCard = null
        viewDx = 0f
        viewDy = 0f
        tweenLater(0.6f) { labelOp = max(labelOp, it) }
    }

    /** Travels along the lines from the focused star to star [j], a comet running ahead of the camera. */
    fun glide(j: Int) {
        val con = focusedCon ?: return
        val i = focusedIndex
        if (busy || j == i || j < 0) return
        busy = true
        val fromX = focusTx + dragX / max(fz, 0.001f)
        val fromY = focusTy + dragY / max(fz, 0.001f)
        val (toX, toY) = targetFor(con, j)
        focusTx = fromX
        focusTy = fromY
        dragX = 0f
        dragY = 0f
        val vx = viewDx
        val vy = viewDy
        val line = con.lines.firstOrNull { (it.a == i && it.b == j) || (it.a == j && it.b == i) }
        launch {
            val hide = scope?.launch { hideFind(true) }
            if (line != null) {
                val forward = line.from == i
                launch {
                    time.tween(0.9f, Ease.io) { p -> line.surge = if (forward) p else 1f - p }
                    line.surge = -1f
                }
            }
            val pix = con.x + con.px[i]
            val piy = con.y + con.py[i]
            val pjx = con.x + con.px[j]
            val pjy = con.y + con.py[j]
            var next: FindView? = null
            val loading = scope?.launch { next = loadFind(con, j) }
            time.tween(0.95f, Ease.io) { p ->
                focusTx = mix(fromX, toX, p)
                focusTy = mix(fromY, toY, p)
                zoomDip = sin(p * PI.toFloat()) * 0.16f
                cometX = mix(pix, pjx, p)
                cometY = mix(piy, pjy, p)
                val k = sin(p * PI.toFloat())
                cometOp = k
                cometSize = 60f + 30f * k
                cometSpike = 110f * k
                cometRot = p * 2f
                val g = min(1f, p * 2f)
                viewDx = vx * (1f - g)
                viewDy = vy * (1f - g)
            }
            zoomDip = 0f
            cometOp = 0f
            viewDx = 0f
            viewDy = 0f
            hide?.join()
            loading?.join()
            focusedIndex = j
            focusedCard = con.stars[j].id
            val star = con.stars[j]
            star.pop = 0.9f
            star.popT = time.now
            showFind(next ?: loadFind(con, j))
            busy = false
        }
    }

    /** The star that lies the way a swipe of [dx], [dy] screen units went from star [i], or -1. */
    fun starToward(con: ConState, i: Int, dx: Float, dy: Float): Int {
        val len0 = hypot(dx, dy)
        if (len0 <= 0f) return -1
        val ux = -dx / len0
        val uy = dy / len0
        var best = -1
        var bestScore = -1e9f
        for (k in con.stars.indices) {
            if (k == i || con.stars[k].vis < 0.5f) continue
            var vx = con.px[k] - con.px[i]
            var vy = con.py[k] - con.py[i]
            val len = hypot(vx, vy)
            if (len <= 0f) continue
            vx /= len
            vy /= len
            val d = vx * ux + vy * uy
            if (d < 0.3f) continue
            val linked = con.lines.any { (it.a == i && it.b == k) || (it.b == i && it.a == k) }
            val score = d + (if (linked) 0.18f else 0f) - len / 1800f
            if (score > bestScore) {
                bestScore = score
                best = k
            }
        }
        return best
    }

    /** Opens the focused star's find; an unlit one lights when the person comes back from it. */
    fun openFromSky() {
        val con = focusedCon ?: return
        if (busy) return
        val i = focusedIndex
        val card = con.stars[i].id
        if (!con.stars[i].lit) lightOnReturn = con to i
        onOpenFind(card)
    }

    /** Back from a find opened from the sky: its picture regains its colour and its star lights. */
    fun returned() {
        val (con, i) = lightOnReturn ?: return
        lightOnReturn = null
        if (busy) return
        busy = true
        launch {
            val view = find
            if (view != null) time.tween(1f) { view.wornK = 1f - it }
            memOn = false
            ignite(con, i)
            onSeen(con.stars[i].id)
            if (find != null) memOn = true
            if (con.stars.all { it.lit }) {
                time.wait(1.2f)
                unfocusNow(false)
                complete(con)
            }
            busy = false
        }
    }

    // Touches, in screen units: what the prototype's pointer handlers do in the sky.

    private var downX = 0f
    private var downY = 0f
    private var lastX = 0f
    private var lastY = 0f
    private var lastT = 0L
    private var velocityX = 0f
    private var velocityY = 0f
    private var moved = false
    private var down = false

    /** A finger went down at [x], [y] screen units, at [t] milliseconds. */
    fun pointerDown(x: Float, y: Float, t: Long) {
        down = true
        downX = x
        downY = y
        lastX = x
        lastY = y
        lastT = t
        moved = false
        velocityX = 0f
        velocityY = 0f
    }

    /** The finger moved. */
    fun pointerMove(x: Float, y: Float, t: Long) {
        if (!down) return
        val dx = x - lastX
        val dy = y - lastY
        val dtm = max(1L, t - lastT).toFloat()
        velocityX = dx / dtm * 1000f
        velocityY = dy / dtm * 1000f
        if (!moved && hypot(x - downX, y - downY) > 7f) moved = true
        if (moved && !busy) {
            if (focusedCon != null) {
                val tx = x - downX
                val ty = y - downY
                dragX = tx * PXW * 0.55f
                dragY = -ty * PXW * 0.55f
                viewDx = tx * 0.9f
                viewDy = ty * 0.6f
            } else {
                upPull = max(0f, upPull - dy)
                look = 1f - min(0.4f, upPull / 520f)
            }
        }
        lastX = x
        lastY = y
        lastT = t
    }

    /**
     * The finger lifted; [hitStar] finds which star of the middle constellation is under a point, or -1.
     */
    fun pointerUp(x: Float, y: Float, w: Float, hitStar: (Float, Float) -> Int) {
        if (!down) return
        down = false
        if (upPull > 0f) {
            val amount = upPull
            upPull = 0f
            if (amount > 140f) lookDown()
            else {
                val l0 = look
                tweenLater(0.35f, Ease.out) { look = l0 + (1f - l0) * it }
            }
            return
        }
        if (busy) return
        val con = focusedCon
        if (con != null && moved) {
            val tx = x - downX
            val ty = y - downY
            var j = -1
            if (hypot(tx, ty) > 50f || hypot(velocityX, velocityY) > 700f) j = starToward(con, focusedIndex, tx, ty)
            if (j >= 0) {
                glide(j)
                return
            }
            val dx = dragX
            val dy = dragY
            val vx = viewDx
            val vy = viewDy
            tweenLater(0.35f, Ease.out) { k ->
                dragX = dx * (1f - k)
                dragY = dy * (1f - k)
                viewDx = vx * (1f - k)
                viewDy = vy * (1f - k)
            }
            return
        }
        if (moved) return
        if (con != null && inView(x, y, w)) {
            openFromSky()
            return
        }
        val hit = hitStar(x, y)
        if (hit >= 0) {
            focusStar(hit)
            return
        }
        if (con != null) unfocus()
    }

    private fun inView(x: Float, y: Float, w: Float): Boolean {
        if (find == null) return false
        if (hypot(x - w / 2f, y - (Proto.SKY_STAR_Y + 120f)) < 200f && hypot(x - w / 2f, y - Proto.SKY_STAR_Y) > 40f) return true
        return abs(x - w / 2f) < (w - 40f) / 2f && abs(y - (Proto.H - 150f)) < 110f
    }

    // The find condensing out of the stars.

    private fun buildCondense(view: FindView) {
        val n = CONDENSE_GRID
        val scaled = coverSquare(view.picture, n)
        val pixels = IntArray(n * n)
        scaled.getPixels(pixels, 0, n, 0, 0, n, n)
        if (scaled !== view.picture) scaled.recycle()
        val r = Random(view.card.hashCode())
        val cx = Proto.SKY_STAR_Y + 120f
        val size = FIND_SIDE
        var count = 0
        for (y in 0 until n) for (x in 0 until n) {
            val p = pixels[y * n + x]
            val red = (p shr 16 and 255) / 255f
            val green = (p shr 8 and 255) / 255f
            val blue = (p and 255) / 255f
            val l = 0.2126f * red + 0.7152f * green + 0.0722f * blue
            val ox = (x + r.nextFloat()) / n - 0.5f
            val oy = 0.5f - (y + r.nextFloat()) / n
            val mask = 1f - sstep(hypot(ox, oy * 1.05f) + (r.nextFloat() - 0.5f) * 0.12f, 0.2f, 0.5f)
            if (r.nextFloat() > l.pow(1.8f) * mask * 2.6f) continue
            if (count >= FindView.MAX_CONDENSE) continue
            view.tx[count] = ox * size * 0.92f
            view.ty[count] = cx - oy * size * 0.92f
            view.sx[count] = r.nextFloat()
            view.sy[count] = r.nextFloat()
            view.cr[count] = mix(1f, red, 0.5f) * 1.1f
            view.cg[count] = mix(0.9f, green, 0.5f) * 1.1f
            view.cb[count] = mix(0.72f, blue, 0.5f) * 1.1f
            view.delay[count] = r.nextFloat() * 0.55f
            view.size[count] = 1.3f + l * 2.8f
            view.phase[count] = r.nextFloat() * 6.28f
            count++
        }
        view.count = count
    }

    private fun vortexFor(con: ConState): FloatArray {
        val per = 60
        val ambient = 240
        val n = con.stars.size * per + ambient
        // Per point: start x, y; target x, y; delay; size seed.
        val out = FloatArray(n * 6)
        var k = 0
        fun put(tx: Float, ty: Float, delay: Float) {
            val a = random.nextFloat() * 6.283f
            val r = 50f + random.nextFloat().pow(0.7f) * 260f
            out[k * 6] = cos(a) * r
            out[k * 6 + 1] = sin(a) * r
            out[k * 6 + 2] = tx + (random.nextFloat() - 0.5f) * 6f
            out[k * 6 + 3] = ty + (random.nextFloat() - 0.5f) * 6f
            out[k * 6 + 4] = delay
            out[k * 6 + 5] = random.nextFloat()
            k++
        }
        con.stars.indices.forEach { i -> repeat(per) { put(con.px[i], con.py[i], i * 0.18f + random.nextFloat() * 0.3f) } }
        repeat(ambient) { put(0f, 0f, 9f) }
        return out
    }

    companion object {
        const val SEG = 72
        const val SPARKS = 70
        const val SLOT_SCALE = 0.34f
        const val PULL_TO_LOOK = 150f
        const val CONDENSE_GRID = 110
        const val FIND_SIDE = 470f

        /** How far the slide moves per screen unit dragged while focused. */
        val PXW = (Proto.RC - 700f) * Proto.TAN / 422f

        /**
         * Where finished constellations sit: the prototype's two up top, then (Claude's choice, for skies with more) two
         * low down and one each at the top and bottom middle.
         */
        val SLOTS = listOf(
            Triple(-168f, 352f, -260f), Triple(172f, 352f, -260f), Triple(-168f, -440f, -260f),
            Triple(172f, -440f, -260f), Triple(0f, 470f, -260f), Triple(0f, -520f, -260f),
        )
    }
}

/** One of the prototype's three star layers: points at depth [z] on the slide, crowding toward the nebula's band. */
internal class StarLayer(random: Random, val count: Int, val z: Float, wx: Float, wy: Float, base: Float, bias: Float) {
    val x = FloatArray(count)
    val y = FloatArray(count)
    val size = FloatArray(count)
    val phase = FloatArray(count)
    val tint = FloatArray(count)

    init {
        var i = 0
        while (i < count) {
            val px = (random.nextFloat() * 2f - 1f) * wx
            val py = (random.nextFloat() * 2f - 1f) * wy
            val bd = abs((px * 0.55f - py * 0.85f) / wx)
            if (random.nextFloat() > (1f - bias) + bias * kotlin.math.exp(-bd * bd / 0.04f)) continue
            x[i] = px
            y[i] = py
            size[i] = base * (0.5f + random.nextFloat().pow(7) * 3.5f)
            phase[i] = random.nextFloat() * 6.28f
            tint[i] = random.nextFloat()
            i++
        }
    }
}

/**
 * The dome's finest stars, as its shader scatters them (one in a few hundred of its 1/1400 radian cells, brighter in the
 * band), kept as directions over the part of the dome the camera ever sees.
 */
internal class MicroStars(random: Random) {
    val dx: FloatArray
    val dy: FloatArray
    val dz: FloatArray
    val bright: FloatArray

    init {
        val xs = ArrayList<Float>()
        val ys = ArrayList<Float>()
        val zs = ArrayList<Float>()
        val bs = ArrayList<Float>()
        val n = normalize(0.707f, -0.198f, -0.679f)
        val minSin = sin(Math.toRadians(-22.0)).toFloat()
        // Cells per steradian times the share of cells a uniform draw over [0.97, 1] stands for.
        val cells = 1400f * 1400f * 0.03f
        val area = 2f * PI.toFloat() * (1f - minSin)
        val samples = (cells * area).toInt()
        val pitches = FloatArray(9) { Proto.PITCH * it / 8f }
        for (s in 0 until samples) {
            val z = minSin + random.nextFloat() * (1f - minSin)
            val a = random.nextFloat() * 2f * PI.toFloat()
            val rr = sqrt(max(0f, 1f - z * z))
            // y is up: the "elevation" axis is y, and the camera faces -z.
            val vx = rr * sin(a)
            val vy = z
            val vz = -rr * cos(a)
            var seen = false
            for (p in pitches) {
                val c = cos(p)
                val sn = sin(p)
                val cy = c * vy + sn * vz
                val cz = -sn * vy + c * vz
                if (cz < 0f && abs(vx) < -cz * Proto.TAN * 0.55f && abs(cy) < -cz * Proto.TAN * 1.05f) {
                    seen = true
                    break
                }
            }
            if (!seen) continue
            val h = 0.97f + random.nextFloat() * 0.03f
            val b0 = vx * n[0] + vy * n[1] + vz * n[2]
            val band = kotlin.math.exp(-b0 * b0 / 0.006f)
            val b = h.pow(160) * (band * 2f + 0.3f) * 0.9f
            if (b < 0.02f) continue
            xs += vx; ys += vy; zs += vz; bs += b
        }
        dx = xs.toFloatArray()
        dy = ys.toFloatArray()
        dz = zs.toFloatArray()
        bright = bs.toFloatArray()
    }

    private fun normalize(x: Float, y: Float, z: Float): FloatArray {
        val l = sqrt(x * x + y * y + z * z)
        return floatArrayOf(x / l, y / l, z / l)
    }
}

/** A square [side] pixels across cut from the middle of [picture] to cover it, as the prototype samples a find. */
internal fun coverSquare(picture: Bitmap, side: Int): Bitmap {
    val w = picture.width
    val h = picture.height
    val crop = min(w, h)
    val src = if (picture.config == Bitmap.Config.HARDWARE) picture.copy(Bitmap.Config.ARGB_8888, false) else picture
    val cut = Bitmap.createBitmap(src, (w - crop) / 2, (h - crop) / 2, crop, crop)
    val out = Bitmap.createScaledBitmap(cut, side, side, true)
    if (cut !== out && cut !== src) cut.recycle()
    if (src !== picture && src !== out) src.recycle()
    return out
}

/** A figure's line picture split into strokes, each in the figure's own units: x and y from -1 to 1, y up. */
internal fun figureStrokes(art: String): List<android.graphics.Path> {
    val pieces = art.trim().split(Regex("(?=M )")).filter { it.isNotBlank() }
    val toUnit = android.graphics.Matrix().apply { setValues(floatArrayOf(1f / 80f, 0f, -100f / 80f, 0f, -1f / 80f, 100f / 80f, 0f, 0f, 1f)) }
    return pieces.map { piece ->
        PathParser().parsePathString(piece).toPath().asAndroidPath().apply { transform(toUnit) }
    }
}

/** A star's own phase for its twinkle, the same each time for the same find. */
internal fun seedOf(id: String): Float = (id.hashCode() and 0xffff) / 65535f * 6.28f

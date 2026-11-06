package com.octaviantocan.mymind.ui.sky

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.Easing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlin.math.PI
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin

/**
 * The stars that leave a find opened from Back today once it closes and fly into the star button (the prototype's
 * mote): a bright four-pointed star on a curve with a trail, then the button bounces and a gold ring spreads from it.
 *
 * @param scope Where the flights run.
 */
@Stable
class StarMotes internal constructor(private val scope: CoroutineScope) {
    private class Flight(val from: Offset) {
        val p = Animatable(0f)
    }

    private val flights = mutableStateListOf<Flight>()
    private var target by mutableStateOf<Offset?>(null)
    private var origin = Offset.Zero

    /** The button's bounce as a star lands, from 0 to 1, 1 at rest. */
    internal val hit = Animatable(1f)

    /** The ring spreading from the button as a star lands, from 0 to 1, 1 once gone. */
    internal val ring = Animatable(1f)

    /**
     * Flies a star from [from], in the window's pixels, into the star button.
     *
     * @param onLanded Called as it reaches the button.
     */
    fun fly(from: Offset, onLanded: () -> Unit) {
        val flight = Flight(from)
        flights += flight
        scope.launch {
            flight.p.animateTo(1f, tween(MOTE_MS, easing = EaseIo))
            flights.remove(flight)
            onLanded()
            launch {
                hit.snapTo(0f)
                hit.animateTo(1f, tween(HIT_MS, easing = LinearEasing))
            }
            ring.snapTo(0f)
            ring.animateTo(1f, tween(RING_MS, easing = CubicBezierEasing(0f, 0f, 0.58f, 1f)))
        }
    }

    internal fun aim(at: Offset) {
        target = at
    }

    internal fun sit(at: Offset) {
        origin = at
    }

    internal fun DrawScope.drawFlights() {
        val to = target ?: return
        for (flight in flights) drawMote(flight.from - origin, to - origin, flight.p.value)
    }

    /** The button's scale for its bounce: up to 1.22 at 28% of the way, then back with an overshoot. */
    internal fun bounce(): Float {
        val t = hit.value
        if (t >= 1f) return 1f
        return if (t < 0.28f) 1f + 0.22f * HitEase.transform(t / 0.28f) else 1.22f - 0.22f * HitEase.transform((t - 0.28f) / 0.72f)
    }
}

private const val MOTE_MS = 800

/** Seconds a star takes to fly into the star button. */
internal const val MOTE_SECONDS = MOTE_MS / 1000f
private const val HIT_MS = 700
private const val RING_MS = 800

private val EaseIo = Easing { Ease.io(it) }
private val HitEase = CubicBezierEasing(0.2f, 1.6f, 0.4f, 1f)

/** Remembers the [StarMotes] for this screen. */
@Composable
fun rememberStarMotes(): StarMotes {
    val scope = rememberCoroutineScope()
    return remember(scope) { StarMotes(scope) }
}

/** Marks the star button as where [motes] land; it bounces as one lands and a gold ring spreads from it. */
fun Modifier.starMoteTarget(motes: StarMotes): Modifier = onGloballyPositioned { motes.aim(it.boundsInRoot().center) }
    .drawWithContent {
        drawContent()
        val r = motes.ring.value
        if (r >= 1f) return@drawWithContent
        val radius = 22.dp.toPx() * (1f + 1.1f * r)
        drawCircle(UP_RING, radius - 1.dp.toPx(), center, alpha = 0.9f * (1f - r), style = Stroke(2.dp.toPx() * (1f + 1.1f * r)))
    }
    .graphicsLayer {
        val s = motes.bounce()
        scaleX = s
        scaleY = s
    }

/** Draws [motes]' flights over this layer. */
fun Modifier.starMotes(motes: StarMotes): Modifier = onGloballyPositioned { motes.sit(it.positionInRoot()) }
    .drawWithContent {
        drawContent()
        with(motes) { drawFlights() }
    }

/**
 * How many finds wait to light in the sky, on the star button's corner; it pops in with the first and is gone at none.
 *
 * @param count How many wait.
 * @param modifier Modifier for the badge; callers align it to the button's top end.
 */
@Composable
fun StarBadge(count: Int, modifier: Modifier = Modifier) {
    val on by animateFloatAsState(if (count > 0) 1f else 0f, tween(450, easing = CubicBezierEasing(0.3f, 1.6f, 0.5f, 1f)), label = "badge")
    if (on <= 0.001f && count <= 0) return
    val shape = RoundedCornerShape(9.dp)
    Box(
        modifier.graphicsLayer {
            alpha = on.coerceIn(0f, 1f)
            val s = 0.4f + 0.6f * on
            scaleX = s
            scaleY = s
        }.defaultMinSize(minWidth = 22.dp).height(22.dp).border(2.dp, Color.White, RoundedCornerShape(11.dp)).padding(2.dp)
            .background(Color(0xFF1E1A10), shape).padding(horizontal = 3.dp).clearAndSetSemantics {},
        contentAlignment = Alignment.Center,
    ) {
        Text(max(count, 1).toString(), style = TextStyle(fontSize = 11.sp, lineHeight = 14.sp, fontWeight = FontWeight.Bold, color = Color(0xFFFFE3A0)))
    }
}

private val UP_RING = Color(233, 185, 73).copy(alpha = 0.85f)
private val MOTE_STAR = PathParser().parsePathString("M12 1.5L13.7 10.3 22.5 12 13.7 13.7 12 22.5 10.3 13.7 1.5 12 10.3 10.3Z").toPath()

// The prototype's flyMote: a quadratic curve lifting up and over, the star growing in the middle and turning, 14 trails.
private fun DrawScope.drawMote(a: Offset, b: Offset, p: Float) {
    if (p <= 0f || p >= 1f) return
    val ctl = Offset(a.x + (b.x - a.x) * 0.15f - 70.dp.toPx(), min(a.y, b.y) - 90.dp.toPx())
    fun bz(t: Float): Offset {
        val u = 1f - t
        return Offset(u * u * a.x + 2 * u * t * ctl.x + t * t * b.x, u * u * a.y + 2 * u * t * ctl.y + t * t * b.y)
    }
    if (p > 0.03f) for (k in 0 until TRAILS) {
        val at = bz(max(0f, p - (k + 1) * 0.014f))
        val s = (1.3f - k * 0.08f) * 1.dp.toPx()
        drawCircle(Brush.radialGradient(0f to TRAIL_CORE, 0.22f / 0.72f to TRAIL_CORE, 0.48f / 0.72f to TRAIL_MID, 1f to TRAIL_MID.copy(alpha = 0f), center = at, radius = 6.1f * s), 6.1f * s, at,
            alpha = (1f - k / TRAILS.toFloat()) * 0.7f)
    }
    val q = bz(p)
    val s = 0.7f + sin(p * PI.toFloat()) * 0.6f
    translate(q.x, q.y) {
        scale(s, s, Offset.Zero) {
            rotate(p * 260f, Offset.Zero) {
                val r = 20.8f.dp.toPx()
                drawCircle(Brush.radialGradient(0f to GLOW_CORE, 0.12f / 0.7f to GLOW_CORE, 0.24f / 0.7f to GLOW_MID, 0.44f / 0.7f to GLOW_EDGE, 1f to GLOW_EDGE.copy(alpha = 0f), center = Offset.Zero, radius = r), r, Offset.Zero)
                val k = 30.dp.toPx() / 24f
                translate(-15.dp.toPx(), -15.dp.toPx()) { scale(k, k, Offset.Zero) { drawPath(MOTE_STAR, Color(0xFFFFF6DC)) } }
            }
        }
    }
}

private const val TRAILS = 14
private val TRAIL_CORE = Color(255, 240, 200)
private val TRAIL_MID = Color(233, 170, 60).copy(alpha = 0.55f)
private val GLOW_CORE = Color(255, 252, 240)
private val GLOW_MID = Color(247, 200, 110).copy(alpha = 0.95f)
private val GLOW_EDGE = Color(233, 170, 60).copy(alpha = 0.4f)

package com.octaviantocan.mymind.ui.sky

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.AnimationVector1D
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** How long a star takes to fly from an uncovered find to the star button: there before the find has opened over it. */
internal const val FLIGHT_MS = 480

/** How long the star button glows once a star reaches it. */
private const val ARRIVAL_MS = 700

/**
 * The stars flying from Back today to the star button as finds are uncovered. The button marks itself with
 * [starFlightTarget], the layer they fly over with [starFlights]; [launch] sends one.
 *
 * @param scope Where the flights run.
 */
@Stable
class StarFlight internal constructor(private val scope: CoroutineScope) {
    private class Flight(val from: Offset, val progress: Animatable<Float, AnimationVector1D>)

    private val flights = mutableStateListOf<Flight>()

    // Where the button's middle and the layer's corner are in the window, kept as they move; read only while drawing.
    private var target: Offset? = null
    private var origin = Offset.Zero

    /** How brightly the button glows from the last star to reach it, 1 on arrival fading to 0. */
    internal val arrival = Animatable(0f)

    /**
     * Sends a star to the button.
     *
     * @param from Where it starts, in the window's pixels: where the finger lifted the leaves.
     */
    fun launch(from: Offset) {
        val flight = Flight(from, Animatable(0f))
        flights += flight
        scope.launch {
            flight.progress.animateTo(1f, tween(FLIGHT_MS, easing = FastOutSlowInEasing))
            flights.remove(flight)
            arrival.snapTo(1f)
            arrival.animateTo(0f, tween(ARRIVAL_MS, easing = LinearEasing))
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
        for (flight in flights) drawFlight(flight.from - origin, to - origin, flight.progress.value)
    }
}

/** Remembers a [StarFlight] for this screen. */
@Composable
fun rememberStarFlight(): StarFlight {
    val scope = rememberCoroutineScope()
    return remember(scope) { StarFlight(scope) }
}

/** Marks the star button as where [flight]'s stars land, and glows around it as one does. */
fun Modifier.starFlightTarget(flight: StarFlight): Modifier = onGloballyPositioned { flight.aim(it.boundsInRoot().center) }
    .drawBehind {
        val glow = flight.arrival.value
        if (glow <= 0f) return@drawBehind
        val radius = size.minDimension / 2f * (1.6f + (1f - glow) * 0.8f)
        drawCircle(Brush.radialGradient(listOf(EMBER.copy(alpha = 0.7f), EMBER.copy(alpha = 0f)), center, radius), radius, alpha = glow)
    }

/** Draws [flight]'s stars over this layer. */
fun Modifier.starFlights(flight: StarFlight): Modifier = onGloballyPositioned { flight.sit(it.positionInRoot()) }
    .drawWithContent {
        drawContent()
        with(flight) { drawFlights() }
    }

// Warm enough to read on the paper page as well as on a dark one.
private val EMBER = Color(0xFFFFA93A)

// A shooting star on a curve that rises from the find and sweeps across to the button, with a tail of fading sparks.
private fun DrawScope.drawFlight(from: Offset, to: Offset, t: Float) {
    if (t <= 0f || t >= 1f) return
    val control = Offset(from.x, to.y)
    fun at(s: Float): Offset {
        val u = s.coerceIn(0f, 1f)
        val v = 1f - u
        return from * (v * v) + control * (2f * v * u) + to * (u * u)
    }
    for (step in TAIL downTo 1) {
        val s = t - step * 0.022f
        if (s <= 0f) continue
        val fade = 1f - step / (TAIL + 1f)
        drawCircle(EMBER, (0.8f + 2.6f * fade) * density, at(s), alpha = 0.8f * fade * fade)
    }
    val head = at(t)
    val glow = 16f * density
    drawCircle(Brush.radialGradient(listOf(EMBER.copy(alpha = 0.85f), EMBER.copy(alpha = 0.25f), Color.Transparent), head, glow), glow, head)
    drawCircle(Color.White, 3.2f * density, head)
}

private const val TAIL = 14

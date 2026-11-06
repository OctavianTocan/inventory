package com.octaviantocan.mymind.ui.sky

import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.time.Duration
import java.time.Instant

/** How long after the sky comes back into view a star waits to light, so the find's sheet has gone first. */
private const val LIGHT_AFTER_MS = 450L

/** How long a star takes to flare as it lights. */
private const val FLARE_MS = 900

/** How long a finished constellation's figure takes to draw in. */
private const val DRAW_IN_MS = 2200

/**
 * What the sky shows as lit and drawn in, which trails the sky itself: a star opened from the sky lights only once the
 * sky is back in view, with a flare, and a constellation whose stars are all lit then draws its figure in.
 */
@Stable
internal class SkyReveal(initiallyLit: Set<String>) {
    private val shownLit = mutableStateMapOf<String, Boolean>().apply { initiallyLit.forEach { put(it, true) } }
    private val flares = mutableStateMapOf<String, Float>()
    private val drawing = mutableStateMapOf<Long, Float>()

    /** Whether [star] shows as lit. */
    fun lit(star: PlacedStar): Boolean = star.lit && shownLit[star.id] == true

    /** How far [id]'s flare has run, from 0 to 1, or null when it is not flaring. */
    fun pop(id: String): Float? = flares[id]

    /** How far [group]'s figure has drawn in, from 0 to 1, or null while any of its stars shows dark. */
    fun drawn(group: PlacedConstellation): Float? {
        if (group.stars.isEmpty() || !group.stars.all(::lit)) return null
        return drawing[group.constellation.id] ?: 1f
    }

    internal fun light(id: String) {
        shownLit[id] = true
    }

    internal fun flare(id: String, value: Float?) {
        if (value == null) flares.remove(id) else flares[id] = value
    }

    internal fun isDrawing(id: Long): Boolean = id in drawing

    internal fun draw(id: Long, value: Float?) {
        if (value == null) drawing.remove(id) else drawing[id] = value
    }
}

// Figures already drawn in while the app has run, so reopening the sky does not draw them in again.
private val drawnIn = HashSet<Long>()

/**
 * Remembers what the sky shows as lit and drawn in for [layout], lighting new stars and drawing in finished figures
 * whenever nothing [covered] the sky.
 */
@Composable
internal fun rememberSkyReveal(layout: SkyLayout, covered: Boolean): SkyReveal {
    val lit = layout.stars.filter { it.lit }.mapTo(HashSet()) { it.id }
    val reveal = remember { SkyReveal(lit) }
    // The flares and draw-ins run on their own, so a later change to the sky never leaves one half done.
    val scope = rememberCoroutineScope()
    LaunchedEffect(lit, covered) {
        if (covered) return@LaunchedEffect
        val fresh = lit.filter { id -> layout.stars.any { it.id == id && !reveal.lit(it) } }
        if (fresh.isNotEmpty()) {
            delay(LIGHT_AFTER_MS)
            for (id in fresh) {
                reveal.light(id)
                scope.launch {
                    animate(0f, 1f, animationSpec = tween(FLARE_MS)) { value, _ -> reveal.flare(id, value) }
                    reveal.flare(id, null)
                }
            }
            delay(FLARE_MS.toLong())
        }
        for (group in layout.constellations) {
            val id = group.constellation.id
            if (reveal.drawn(group) == null || id in drawnIn || reveal.isDrawing(id)) continue
            // A figure finished long ago is simply there; one finished lately, or just now, draws in.
            val finished = group.constellation.completed?.let { runCatching { Instant.parse(it) }.getOrNull() }
            if (finished != null && finished.isBefore(Instant.now().minus(Duration.ofDays(1)))) {
                drawnIn += id
                continue
            }
            reveal.draw(id, 0f)
            scope.launch {
                animate(0f, 1f, animationSpec = tween(DRAW_IN_MS, easing = LinearOutSlowInEasing)) { value, _ -> reveal.draw(id, value) }
                drawnIn += id
                reveal.draw(id, null)
            }
        }
    }
    return reveal
}

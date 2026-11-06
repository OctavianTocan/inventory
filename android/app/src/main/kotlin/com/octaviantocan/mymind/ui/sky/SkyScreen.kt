package com.octaviantocan.mymind.ui.sky

import androidx.compose.animation.core.withInfiniteAnimationFrameNanos
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Constellation
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlin.math.abs
import kotlin.math.exp
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.random.Random

// The sky's own colours: it is night whatever the page's scheme.
private val NIGHT = Color(0xFF070B18)
private val INK = Color(0xFFEDE6D6)
private val GOLD = Color(0xFFFFDB8F)
private val DIM = Color(0xFF8CA3C2)

/** How many finished constellations the sky shows, newest first. */
private const val DONE_SHOWN = 6

/** How many loose stars the sky shows, newest first. */
private const val LOOSE_SHOWN = 40

/** A star as the sky places it: [at] in pixels on the sky. */
internal class PlacedStar(val id: String, val at: Offset, val lit: Boolean, val big: Boolean)

/** A constellation as the sky places it: its figure in a square [size] wide around [center]. */
internal class PlacedConstellation(
    val constellation: Constellation,
    val figure: SkyFigure,
    val center: Offset,
    val size: Float,
    val active: Boolean,
    val stars: List<PlacedStar>,
) {
    /** Every star lit, so its figure draws in. */
    val complete: Boolean get() = stars.isNotEmpty() && stars.all { it.lit }
}

/**
 * Where everything on the sky sits for one screen.
 *
 * @property constellations The active one first, if any, then the finished ones.
 * @property loose Finds opened from Back today that belong to no constellation shown.
 * @property nameAt Where the active constellation's name sits, or null without one.
 */
internal class SkyLayout(val constellations: List<PlacedConstellation>, val loose: List<PlacedStar>, val nameAt: Offset?) {
    /** Every star, tappable. */
    val stars: List<PlacedStar> get() = constellations.flatMap { it.stars } + loose
}

/**
 * Lays the sky out on a [width] by [height] pixel screen, keeping [top] pixels clear for the title and [bottom] for the
 * navigation bar: the active constellation large in the upper middle with its name under it, up to six finished ones
 * small below, and loose stars scattered where nothing else is, each always in the same place for the same find.
 */
internal fun skyLayout(sky: Sky, width: Float, height: Float, top: Float, bottom: Float): SkyLayout {
    val room = max(1f, height - top - bottom)
    val placed = mutableListOf<PlacedConstellation>()
    var below = top
    var nameAt: Offset? = null
    sky.active?.let { active ->
        val size = min(width * 0.8f, room * 0.46f)
        val center = Offset(width / 2f, top + size * 0.55f)
        placed += place(active, center, size, active = true)
        nameAt = Offset(width / 2f, center.y + size / 2f + size * 0.06f)
        below = center.y + size / 2f + size * 0.2f
    }
    val done = sky.done.takeLast(DONE_SHOWN).reversed()
    if (done.isNotEmpty()) {
        val rows = (done.size + 2) / 3
        val rowHeight = max(1f, (height - bottom - below) / rows)
        val size = min(width / 3f * 0.8f, rowHeight * 0.8f)
        done.forEachIndexed { index, constellation ->
            // A row that is not full is spread across the width, not left at one side.
            val inRow = min(3, done.size - index / 3 * 3)
            val center = Offset(width * (index % 3 + 0.5f) / inRow, below + rowHeight * (index / 3 + 0.5f))
            placed += place(constellation, center, size, active = false)
        }
    }
    val inConstellations = placed.flatMap { it.constellation.cards }.toSet()
    val activeBox = placed.firstOrNull { it.active }?.let { it.center to it.size / 2f * 1.1f }
    val loose = sky.stars.filter { it.id !in inConstellations }.takeLast(LOOSE_SHOWN).map { star ->
        val random = Random(star.id.hashCode())
        val x = width * (0.08f + random.nextFloat() * 0.84f)
        var y = top + room * (0.04f + random.nextFloat() * 0.92f)
        // A loose star never sits inside the active constellation, where it would read as one of its stars.
        if (activeBox != null && abs(x - activeBox.first.x) < activeBox.second && abs(y - activeBox.first.y) < activeBox.second) {
            y = below + (height - bottom - below) * random.nextFloat()
        }
        PlacedStar(star.id, Offset(x, y), lit = true, big = false)
    }
    return SkyLayout(placed, loose, nameAt)
}

private fun place(constellation: Constellation, center: Offset, size: Float, active: Boolean): PlacedConstellation {
    val figure = figureOf(constellation.figure)
    val scale = size / FIGURE_BOX
    val corner = center - Offset(size / 2f, size / 2f)
    val lit = constellation.lit.toSet()
    val stars = constellation.cards.take(figure.stars.size).mapIndexed { index, id ->
        PlacedStar(id, corner + figure.stars[index] * scale, lit = !active || id in lit, big = active)
    }
    return PlacedConstellation(constellation, figure, center, size, active, stars)
}

/** A faint star of the field behind the constellations: where it is as a share of the screen, how big, how it twinkles. */
private class FieldStar(val x: Float, val y: Float, val radius: Float, val phase: Float, val tint: Color)

// The same field every time, thicker along the nebula's band.
private fun fieldStars(): List<FieldStar> {
    val random = Random(7)
    val stars = mutableListOf<FieldStar>()
    while (stars.size < 320) {
        val x = random.nextFloat()
        val y = random.nextFloat()
        val across = (x - 0.5f) * -0.62f + (y - 0.52f) * 2.17f * -0.39f
        if (random.nextFloat() > 0.35f + 0.65f * exp(-across * across / 0.05f)) continue
        val big = random.nextFloat().let { it * it * it * it * it * it * it }
        val tint = random.nextFloat().let { if (it < 0.33f) Color(0xFFB8D1FF) else if (it < 0.8f) Color(0xFFFFF8EB) else Color(0xFFFFD49E) }
        stars += FieldStar(x, y, 0.5f + big * 3.5f, random.nextFloat() * 6.28f, tint)
    }
    return stars
}

/**
 * Your sky: the active constellation, whose stars light as its finds are opened again, the finished ones with their
 * figures drawn in, and the loose stars of other finds opened from Back today, over a still nebula. Its only words
 * are its title and the active constellation's name. Stars twinkle while it shows.
 *
 * @param sky The sky.
 * @param finds The sky's finds that are known, by id, for what each star says to accessibility.
 * @param onOpenStar Called with a star's find when it is tapped.
 * @param onClose Called when Close is tapped.
 * @param modifier Modifier for the full-screen sky.
 */
@Composable
fun SkyScreen(sky: Sky, finds: Map<String, Card>, onOpenStar: (String) -> Unit, onClose: () -> Unit, modifier: Modifier = Modifier) {
    val density = LocalDensity.current
    val nebula by produceState<ImageBitmap?>(null) { value = Nebula.picture() }
    val field = remember { fieldStars() }
    val time = remember { mutableFloatStateOf(0f) }
    LaunchedEffect(Unit) {
        var start = -1L
        while (true) withInfiniteAnimationFrameNanos {
            if (start < 0) start = it
            time.floatValue = (it - start) / 1_000_000_000f
        }
    }
    BoxWithConstraints(modifier.fillMaxSize().background(NIGHT).testTag("sky")) {
        val width = constraints.maxWidth.toFloat()
        val height = constraints.maxHeight.toFloat()
        val top = with(density) { 84.dp.toPx() }
        val bottom = WindowInsets.navigationBars.getBottom(density) + with(density) { 24.dp.toPx() }
        val layout = remember(sky, width, height, top, bottom) { skyLayout(sky, width, height, top, bottom) }
        Canvas(Modifier.fillMaxSize()) {
            nebula?.let { drawCovering(it) }
            val t = time.floatValue
            for (star in field) {
                val twinkle = 0.7f + 0.3f * sin(t * (1.1f + star.phase * 0.1f) + star.phase * 9f)
                val at = Offset(star.x * size.width, star.y * size.height)
                val radius = star.radius * density.density * 0.5f
                if (star.radius > 1.6f) drawCircle(star.tint, radius * 3f, at, alpha = 0.12f * twinkle)
                drawCircle(star.tint, radius, at, alpha = 0.85f * twinkle)
            }
            for (constellation in layout.constellations) drawConstellation(constellation, t)
            for (star in layout.loose) drawStar(star, t)
        }
        val target = with(density) { STAR_TARGET.roundToPx() }
        for (star in layout.stars) {
            val heading = finds[star.id]?.let(::cardHeading) ?: "A find"
            Box(
                Modifier
                    .offset { IntOffset(star.at.x.roundToInt() - target / 2, star.at.y.roundToInt() - target / 2) }
                    .size(STAR_TARGET)
                    .semantics { contentDescription = if (star.lit) "$heading, a lit star" else "$heading, a dark star" }
                    .clickable(remember { MutableInteractionSource() }, indication = null, role = Role.Button) { onOpenStar(star.id) },
            )
        }
        val name = sky.active?.name
        val nameAt = layout.nameAt
        if (name != null && nameAt != null) {
            Text(name, Modifier.fillMaxWidth().offset { IntOffset(0, nameAt.y.roundToInt()) }, style = LibraryStyle.label,
                color = INK.copy(alpha = 0.72f), textAlign = TextAlign.Center)
        }
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 16.dp, top = 12.dp).height(44.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("Your sky", Modifier.weight(1f).semantics { heading() }, style = LibraryStyle.title, color = INK)
            Box(
                Modifier.size(44.dp).clip(CircleShape).background(INK.copy(alpha = 0.1f))
                    .semantics { contentDescription = "Close the sky" }
                    .clickable(role = Role.Button, onClick = onClose),
                contentAlignment = Alignment.Center,
            ) { Image(painterResource(R.drawable.ic_sheet_close), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(INK)) }
        }
    }
}

/** How big each star's tap target is. */
private val STAR_TARGET = 44.dp

// The nebula fills the screen like a cover photo: stretched evenly and cut at the sides or ends.
private fun DrawScope.drawCovering(image: ImageBitmap) {
    val scale = max(size.width / image.width, size.height / image.height)
    val srcWidth = (size.width / scale).roundToInt().coerceAtMost(image.width)
    val srcHeight = (size.height / scale).roundToInt().coerceAtMost(image.height)
    drawImage(
        image,
        srcOffset = IntOffset((image.width - srcWidth) / 2, (image.height - srcHeight) / 2),
        srcSize = IntSize(srcWidth, srcHeight),
        dstSize = IntSize(size.width.roundToInt(), size.height.roundToInt()),
        filterQuality = FilterQuality.Low,
    )
}

private fun DrawScope.drawConstellation(constellation: PlacedConstellation, t: Float) {
    val unit = density
    val strength = if (constellation.active) 1f else 0.7f
    if (constellation.complete) {
        val scale = constellation.size / FIGURE_BOX
        val corner = constellation.center - Offset(constellation.size / 2f, constellation.size / 2f)
        withTransform({
            translate(corner.x, corner.y)
            scale(scale, scale, Offset.Zero)
        }) {
            val path = constellation.figure.path
            drawPath(path, GOLD, alpha = 0.06f * strength, style = Stroke(8f * unit / scale, cap = StrokeCap.Round, join = StrokeJoin.Round))
            drawPath(path, GOLD, alpha = 0.14f * strength, style = Stroke(3.5f * unit / scale, cap = StrokeCap.Round, join = StrokeJoin.Round))
            drawPath(path, INK, alpha = 0.55f * strength, style = Stroke(1.1f * unit / scale, cap = StrokeCap.Round, join = StrokeJoin.Round))
        }
    }
    val stars = constellation.stars
    val trim = (if (constellation.active) 10f else 5f) * unit
    for ((a, b) in constellation.figure.edges) {
        val from = stars.getOrNull(a) ?: continue
        val to = stars.getOrNull(b) ?: continue
        val along = to.at - from.at
        val length = along.getDistance()
        if (length <= trim * 2f) continue
        val start = from.at + along * (trim / length)
        val end = to.at - along * (trim / length)
        if (from.lit && to.lit) {
            // Once the figure has drawn in, the lines step back so the picture reads first.
            val lines = if (constellation.complete) 0.35f * strength else strength
            drawLine(GOLD, start, end, strokeWidth = 5f * unit, cap = StrokeCap.Round, alpha = 0.12f * lines)
            drawLine(GOLD, start, end, strokeWidth = 1.4f * unit, cap = StrokeCap.Round, alpha = 0.85f * lines)
        } else {
            drawLine(DIM, start, end, strokeWidth = 1.6f * unit, cap = StrokeCap.Round, alpha = 0.55f,
                pathEffect = PathEffect.dashPathEffect(floatArrayOf(0.1f, 7f * unit)))
        }
    }
    for (star in stars) drawStar(star, t)
}

private fun DrawScope.drawStar(star: PlacedStar, t: Float) {
    val unit = density
    val pulse = 0.88f + 0.12f * sin(t * 1.3f + abs(star.id.hashCode() % 628) / 100f)
    if (!star.lit) {
        drawCircle(DIM, 6f * unit, star.at, alpha = 0.75f, style = Stroke(1.2f * unit))
        drawCircle(INK, 1.8f * unit, star.at, alpha = 0.9f)
        return
    }
    val glow = (if (star.big) 18f else 9f) * unit * pulse
    drawCircle(
        Brush.radialGradient(
            0f to Color.White,
            0.08f to Color(0xF2FFEBC4),
            0.22f to Color(0x52FFC878),
            0.5f to Color(0x12FFAA50),
            1f to Color.Transparent,
            center = star.at,
            radius = glow,
        ),
        glow,
        star.at,
    )
    if (star.big) {
        val spike = 15f * unit * pulse
        drawLine(Color.White, star.at - Offset(spike, 0f), star.at + Offset(spike, 0f), strokeWidth = 0.8f * unit, alpha = 0.3f)
        drawLine(Color.White, star.at - Offset(0f, spike), star.at + Offset(0f, spike), strokeWidth = 0.8f * unit, alpha = 0.3f)
    }
}

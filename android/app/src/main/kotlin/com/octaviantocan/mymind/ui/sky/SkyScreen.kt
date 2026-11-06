package com.octaviantocan.mymind.ui.sky

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.withInfiniteAnimationFrameNanos
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
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
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.changedToUpIgnoreConsumed
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChangeIgnoreConsumed
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Constellation
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.ui.library.GalleryTile
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryTileShowsWords
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.math.PI
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
 * Tapping a star moves in on it and its find condenses out of starlight under it: stars from all over the sky fly
 * together and become the find's tile, with its name below. A swipe travels along the constellation to the star that
 * lies that way; tapping the find opens it; Back, or a tap on empty sky, moves out again. A star opened again lights
 * once the sky is back in view, and a constellation whose stars are all lit draws its figure in.
 *
 * @param sky The sky.
 * @param finds The sky's finds that are known, by id.
 * @param onOpenStar Called with a find's id when the find is tapped.
 * @param onClose Called when Close is tapped.
 * @param modifier Modifier for the full-screen sky.
 * @param covered True while something covers the sky, such as a find opened from it; stars wait to light until then.
 */
@Composable
fun SkyScreen(
    sky: Sky,
    finds: Map<String, Card>,
    onOpenStar: (String) -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
    covered: Boolean = false,
) {
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
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
    val camera = remember { SkyCamera() }
    val gather = remember { Animatable(0f) }
    val shown = remember { Animatable(0f) }
    val glide = remember { Animatable(0f) }
    var focus by remember { mutableStateOf<PlacedStar?>(null) }
    var condense by remember { mutableStateOf<Condense?>(null) }
    var line by remember { mutableStateOf<Pair<Offset, Offset>?>(null) }
    val moving = remember { arrayOfNulls<Job>(1) }
    val swipe = remember { arrayOfNulls<(Offset) -> Unit>(1) }
    val tapEmpty = remember { arrayOfNulls<() -> Unit>(1) }
    // On the whole sky, so a swipe that starts on the find or a star still travels.
    BoxWithConstraints(modifier.fillMaxSize().background(NIGHT).testTag("sky").skyGestures({ swipe[0]?.invoke(it) }, { tapEmpty[0]?.invoke() })) {
        val width = constraints.maxWidth.toFloat()
        val height = constraints.maxHeight.toFloat()
        val top = with(density) { 84.dp.toPx() }
        val bottom = WindowInsets.navigationBars.getBottom(density) + with(density) { 24.dp.toPx() }
        val layout = remember(sky, width, height, top, bottom) { skyLayout(sky, width, height, top, bottom) }
        camera.anchor = Offset(width / 2f, top + (height - top - bottom) * 0.1f)
        val tileWidth = width * 0.58f
        val tile = Rect(Offset((width - tileWidth) / 2f, camera.anchor.y + with(density) { 44.dp.toPx() }), Size(tileWidth, tileWidth / 0.8f))
        val reveal = rememberSkyReveal(layout, covered)

        fun groupOf(star: PlacedStar) = layout.constellations.firstOrNull { group -> group.stars.any { it.id == star.id && it.at == star.at } }
        fun zoomFor(star: PlacedStar): Float = groupOf(star)?.let { (width * 1.25f / it.size).coerceIn(1.2f, 4f) } ?: 1.8f
        fun move(block: suspend CoroutineScope.() -> Unit) {
            moving[0]?.cancel()
            moving[0] = scope.launch(block = block)
        }
        suspend fun condenseOn(star: PlacedStar) {
            condense = Condense(star.id, tile, Size(width, height))
            shown.snapTo(1f)
            gather.snapTo(0f)
            gather.animateTo(1f, tween(CONDENSE_MS, easing = LinearOutSlowInEasing))
        }
        fun glideTo(star: PlacedStar) {
            val from = focus ?: return
            if (from.id == star.id && from.at == star.at) return
            focus = star
            move {
                launch { shown.animateTo(0f, tween(RELEASE_MS)) }
                launch { camera.zoom.animateTo(zoomFor(star), tween(GLIDE_MS, easing = FastOutSlowInEasing)) }
                line = from.at to star.at
                glide.snapTo(0f)
                launch { glide.animateTo(1f, tween(GLIDE_MS, easing = FastOutSlowInEasing)) }
                camera.look.animateTo(star.at, tween(GLIDE_MS, easing = FastOutSlowInEasing))
                line = null
                condenseOn(star)
            }
        }
        fun focusOn(star: PlacedStar) {
            if (focus != null) return glideTo(star)
            focus = star
            move {
                // From rest the camera only needs to lift; moving back in while it is still moving out turns it around.
                if (camera.lift.value == 0f) {
                    camera.look.snapTo(star.at)
                    camera.zoom.snapTo(zoomFor(star))
                } else {
                    launch { camera.look.animateTo(star.at, tween(FOCUS_MS, easing = FastOutSlowInEasing)) }
                    launch { camera.zoom.animateTo(zoomFor(star), tween(FOCUS_MS, easing = FastOutSlowInEasing)) }
                }
                launch { camera.lift.animateTo(1f, tween(FOCUS_MS, easing = FastOutSlowInEasing)) }
                delay(FOCUS_MS / 2L)
                condenseOn(star)
            }
        }
        fun moveOut() {
            if (focus == null) return
            focus = null
            move {
                launch { shown.animateTo(0f, tween(RELEASE_MS)) }
                camera.lift.animateTo(0f, tween(FOCUS_MS, easing = FastOutSlowInEasing))
                condense = null
            }
        }
        fun step(by: Int) {
            val star = focus ?: return
            val group = groupOf(star) ?: return
            val index = group.stars.indexOfFirst { it.id == star.id && it.at == star.at }
            group.stars.getOrNull(Math.floorMod(index + by, group.stars.size))?.let(::glideTo)
        }
        swipe[0] = { drag: Offset ->
            val star = focus
            val group = star?.let(::groupOf)
            if (star != null && group != null) {
                val index = group.stars.indexOfFirst { it.id == star.id && it.at == star.at }
                // Dragging the sky one way brings the star that lies the other way into view, as a map pans.
                starToward(group, index, -drag)?.let { glideTo(group.stars[it]) }
            }
        }
        tapEmpty[0] = { moveOut() }
        BackHandler(enabled = focus != null) { moveOut() }

        Canvas(Modifier.fillMaxSize()) {
            val t = time.floatValue
            val scale = camera.scale()
            val shift = camera.shift()
            // The nebula and the faint stars lie far behind, so they move less than the constellations as the camera does.
            nebula?.let { image ->
                withTransform({
                    translate(shift.x * 0.12f, shift.y * 0.12f)
                    scale(1.1f + (scale - 1f) * 0.12f, 1.1f + (scale - 1f) * 0.12f, center)
                }) { drawCovering(image) }
            }
            val far = 1f + (scale - 1f) * 0.3f
            for (star in field) {
                val twinkle = 0.7f + 0.3f * sin(t * (1.1f + star.phase * 0.1f) + star.phase * 9f)
                val at = Offset(star.x * size.width, star.y * size.height) * far + shift * 0.3f
                val radius = star.radius * density.density * 0.5f
                if (star.radius > 1.6f) drawCircle(star.tint, radius * 3f, at, alpha = 0.12f * twinkle)
                drawCircle(star.tint, radius, at, alpha = 0.85f * twinkle)
            }
            withTransform({
                translate(shift.x, shift.y)
                scale(scale, scale, Offset.Zero)
            }) {
                for (constellation in layout.constellations) drawConstellation(constellation, t, reveal)
                for (star in layout.loose) drawStar(star, t, reveal)
                line?.let { (from, to) -> drawComet(from, to, glide.value) }
            }
        }
        val target = with(density) { STAR_TARGET.roundToPx() }
        for (star in layout.stars) {
            val heading = finds[star.id]?.let(::cardHeading) ?: "A find"
            Box(
                Modifier
                    .offset {
                        val at = camera.map(star.at)
                        IntOffset(at.x.roundToInt() - target / 2, at.y.roundToInt() - target / 2)
                    }
                    .size(STAR_TARGET)
                    .semantics { contentDescription = if (star.lit) "$heading, a lit star" else "$heading, a dark star" }
                    .clickable(remember { MutableInteractionSource() }, indication = null, role = Role.Button) { focusOn(star) },
            )
        }
        val name = sky.active?.name
        val nameAt = layout.nameAt
        if (name != null && nameAt != null) {
            Text(name, Modifier.fillMaxWidth().offset { IntOffset(0, nameAt.y.roundToInt()) }.graphicsLayer { alpha = 1f - camera.lift.value },
                style = LibraryStyle.label, color = INK.copy(alpha = 0.72f), textAlign = TextAlign.Center)
        }
        val card = focus?.let { finds[it.id] }
        if (card != null) {
            key(card.id) {
                val formed = { smoothStep(0.45f, 0.95f, gather.value) * shown.value }
                val tileSize = with(density) { DpSize(tile.width.toDp(), tile.height.toDp()) }
                GalleryTile(
                    card, onClick = { onOpenStar(card.id) },
                    modifier = Modifier.offset { IntOffset(tile.left.roundToInt(), tile.top.roundToInt()) }.size(tileSize)
                        .graphicsLayer { alpha = formed() }
                        .semantics {
                            customActions = listOf(
                                CustomAccessibilityAction("Next star") { step(1); true },
                                CustomAccessibilityAction("Previous star") { step(-1); true },
                            )
                        },
                )
                // A picture is named under it; a tile of words already shows its name.
                if (remember(card) { !galleryTileShowsWords(card) }) {
                    Text(cardHeading(card), Modifier.fillMaxWidth().padding(horizontal = 32.dp)
                        .offset { IntOffset(0, (tile.bottom + 14.dp.toPx()).roundToInt()) }.graphicsLayer { alpha = formed() },
                        style = LibraryStyle.body, color = INK.copy(alpha = 0.85f), textAlign = TextAlign.Center, maxLines = 2,
                        overflow = TextOverflow.Ellipsis)
                }
            }
        }
        Canvas(Modifier.fillMaxSize()) {
            val gathering = condense ?: return@Canvas
            if (shown.value < 1f) return@Canvas
            with(gathering) { drawGathering(gather.value, time.floatValue) }
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

/**
 * The star of [group] at [from] that lies most the way of [direction], preferring one joined to it by a line and a
 * nearer one, or null when none lies that way.
 */
internal fun starToward(group: PlacedConstellation, from: Int, direction: Offset): Int? {
    val origin = group.stars.getOrNull(from)?.at ?: return null
    val length = direction.getDistance()
    if (length == 0f) return null
    val way = direction / length
    var best: Int? = null
    var bestScore = -Float.MAX_VALUE
    group.stars.forEachIndexed { index, star ->
        if (index == from) return@forEachIndexed
        val apart = star.at - origin
        val distance = apart.getDistance()
        if (distance == 0f) return@forEachIndexed
        val along = (apart.x * way.x + apart.y * way.y) / distance
        if (along < 0.3f) return@forEachIndexed
        val joined = group.figure.edges.any { (a, b) -> (a == from && b == index) || (a == index && b == from) }
        val score = along + (if (joined) 0.18f else 0f) - distance / (group.size * 2f)
        if (score > bestScore) {
            bestScore = score
            best = index
        }
    }
    return best
}

// A swipe of more than a little way calls onSwipe with how far the finger went; a tap no star or find took calls onTap.
private fun Modifier.skyGestures(onSwipe: (Offset) -> Unit, onTap: () -> Unit): Modifier = pointerInput(Unit) {
    awaitEachGesture {
        val down = awaitFirstDown(requireUnconsumed = false)
        var travelled = Offset.Zero
        while (true) {
            val change = awaitPointerEvent().changes.firstOrNull { it.id == down.id } ?: return@awaitEachGesture
            travelled += change.positionChangeIgnoreConsumed()
            // Once it is a drag, the sky takes it, so the find or star it began on does not also take it as a tap.
            if (travelled.getDistance() >= viewConfiguration.touchSlop && !change.changedToUpIgnoreConsumed()) change.consume()
            if (change.changedToUpIgnoreConsumed()) {
                if (travelled.getDistance() > SWIPE.toPx()) onSwipe(travelled)
                else if (travelled.getDistance() < viewConfiguration.touchSlop && !change.isConsumed) onTap()
                return@awaitEachGesture
            }
        }
    }
}

private val SWIPE = 48.dp

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

private fun DrawScope.drawConstellation(constellation: PlacedConstellation, t: Float, reveal: SkyReveal) {
    val unit = density
    val strength = if (constellation.active) 1f else 0.7f
    val drawn = reveal.drawn(constellation)
    if (drawn != null) {
        val scale = constellation.size / FIGURE_BOX
        val corner = constellation.center - Offset(constellation.size / 2f, constellation.size / 2f)
        // Drawing in, each stroke of the figure grows from its start.
        val effect = if (drawn < 1f) PathEffect.dashPathEffect(floatArrayOf(FIGURE_STROKE * drawn + 0.01f, FIGURE_STROKE * 2f)) else null
        withTransform({
            translate(corner.x, corner.y)
            scale(scale, scale, Offset.Zero)
        }) {
            val path = constellation.figure.path
            drawPath(path, GOLD, alpha = 0.06f * strength, style = Stroke(8f * unit / scale, cap = StrokeCap.Round, join = StrokeJoin.Round, pathEffect = effect))
            drawPath(path, GOLD, alpha = 0.14f * strength, style = Stroke(3.5f * unit / scale, cap = StrokeCap.Round, join = StrokeJoin.Round, pathEffect = effect))
            drawPath(path, INK, alpha = 0.55f * strength, style = Stroke(1.1f * unit / scale, cap = StrokeCap.Round, join = StrokeJoin.Round, pathEffect = effect))
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
        if (reveal.lit(from) && reveal.lit(to)) {
            // Once the figure has drawn in, the lines step back so the picture reads first.
            val lines = strength * (1f - 0.65f * (drawn ?: 0f))
            drawLine(GOLD, start, end, strokeWidth = 5f * unit, cap = StrokeCap.Round, alpha = 0.12f * lines)
            drawLine(GOLD, start, end, strokeWidth = 1.4f * unit, cap = StrokeCap.Round, alpha = 0.85f * lines)
        } else {
            drawLine(DIM, start, end, strokeWidth = 1.6f * unit, cap = StrokeCap.Round, alpha = 0.55f,
                pathEffect = PathEffect.dashPathEffect(floatArrayOf(0.1f, 7f * unit)))
        }
    }
    for (star in stars) drawStar(star, t, reveal)
}

// Longer than any one stroke of a figure, in figure units.
private const val FIGURE_STROKE = 700f

private fun DrawScope.drawStar(star: PlacedStar, t: Float, reveal: SkyReveal) {
    val unit = density
    val pulse = 0.88f + 0.12f * sin(t * 1.3f + abs(star.id.hashCode() % 628) / 100f)
    if (!reveal.lit(star)) {
        drawCircle(DIM, 6f * unit, star.at, alpha = 0.75f, style = Stroke(1.2f * unit))
        drawCircle(INK, 1.8f * unit, star.at, alpha = 0.9f)
        return
    }
    // A star that has just lit flares, then settles, with a ring running out from it.
    val pop = reveal.pop(star.id)
    val flare = if (pop != null) 1f + 1.4f * sin(pop * PI.toFloat()) else 1f
    if (pop != null) drawCircle(GOLD, (8f + 40f * pop) * unit, star.at, alpha = 0.6f * (1f - pop), style = Stroke(1.5f * unit))
    val glow = (if (star.big) 18f else 9f) * unit * pulse * flare
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
        val spike = 15f * unit * pulse * flare
        drawLine(Color.White, star.at - Offset(spike, 0f), star.at + Offset(spike, 0f), strokeWidth = 0.8f * unit, alpha = 0.3f)
        drawLine(Color.White, star.at - Offset(0f, spike), star.at + Offset(0f, spike), strokeWidth = 0.8f * unit, alpha = 0.3f)
    }
}

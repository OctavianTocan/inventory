package com.octaviantocan.mymind.ui.sky

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Matrix
import android.graphics.Paint
import android.os.Build
import androidx.activity.compose.BackHandler
import androidx.annotation.RequiresApi
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Constellation
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryText
import com.octaviantocan.mymind.ui.shell.LocalShellImmersion
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.isActive
import kotlin.math.max
import kotlin.math.min

/** The sky's colour at the top of the screen while looking up, laid over the status bar. */
private val SKY_TOP = Color(0xFF0C1320)

/** How many finds opened on their own show as loose stars, the latest ones. */
private const val MAX_LOOSE = 12

/**
 * The sky over the gallery: the camera looks up at it from the page and back down again, as the prototype's does.
 * Holds the scene, which runs the sky's sequences, and the painter that draws it.
 */
@Stable
class SkyState internal constructor() {
    internal val scene = SkyScene()
    internal val painter = SkyPainter()

    /** Pixels per screen unit, the prototype's 844 units to the sky's height; set once the sky is laid out. */
    internal var unit = 1f

    /** Loads a find's picture for it to condense into; tests swap it. */
    internal var loadPicture: suspend (Context, Card) -> Bitmap? = { context, card -> com.octaviantocan.mymind.ui.menu.loadPicture(context, card) }

    private var page: SkyProjector? = null

    /** True from looking up until the camera is back down on the page. */
    val lookingUp: Boolean get() = scene.lookingUp

    /**
     * How much of the page's own header and controls show, from 1 on the page to 0 once the camera starts to look up,
     * so the top of the page opens straight onto the sky (the prototype's `hk`).
     */
    val head: Float get() = 1f - sstep(scene.look, 0.005f, 0.07f)

    /** Looks up from the page to the sky; any finds waiting light their stars once it is there. */
    fun lookUp() = scene.lookUp()

    /** The camera for the page under the sky, [width] by [height] pixels, placed for this frame. */
    internal fun pageCamera(width: Float, height: Float): SkyProjector {
        val p = page?.takeIf { it.widthPx == width && it.heightPx == height } ?: SkyProjector(width, height).also { page = it }
        p.place(scene.look, scene.punch, scene.jitterX, scene.jitterY)
        return p
    }
}

/** Remembers the sky over the gallery. */
@Composable
fun rememberSkyState(): SkyState {
    return remember { SkyState() }
}

/**
 * Turns a pull down at the top of the page into looking up, as the prototype does: the camera tilts up as far as the
 * finger pulls, and letting go past [SkyScene.PULL_TO_LOOK] units looks up the rest of the way.
 *
 * @param sky The sky.
 */
internal class SkyPull(private val sky: SkyState) : NestedScrollConnection {
    /** False hands the pull to the page, as when there is no sky yet or reels are open; read as the finger moves. */
    var enabled: () -> Boolean = { false }

    override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
        val scene = sky.scene
        if (scene.pull <= 0f || available.y >= 0f || source != NestedScrollSource.UserInput) return Offset.Zero
        val before = scene.pull
        scene.pullBy(available.y / sky.unit)
        return Offset(0f, (scene.pull - before) * sky.unit)
    }

    override fun onPostScroll(consumed: Offset, available: Offset, source: NestedScrollSource): Offset {
        if (!enabled() || source != NestedScrollSource.UserInput || available.y <= 0f || sky.scene.lookingUp) return Offset.Zero
        sky.scene.pullBy(available.y / sky.unit)
        return Offset(0f, available.y)
    }

    override suspend fun onPreFling(available: Velocity): Velocity {
        if (sky.scene.pull <= 0f) return Velocity.Zero
        sky.scene.releasePull(available.y / sky.unit)
        return available
    }
}

/** Remembers [SkyPull] for [sky], on while [enabled] says so. */
@Composable
fun rememberSkyPull(sky: SkyState, enabled: () -> Boolean): NestedScrollConnection {
    val pull = remember(sky) { SkyPull(sky) }
    pull.enabled = enabled
    return pull
}

/**
 * Lays the page on the ground the camera looks up from: as it looks up, the page tilts away below and the paper under
 * it fades out into the sky above its top (the prototype's paper plane). On the page it draws as it is.
 *
 * @param sky The sky.
 * @param paper The page's colour.
 * @param scrolled How far the page has scrolled past its top, in pixels.
 */
fun Modifier.skyTilt(sky: SkyState, paper: Color, scrolled: () -> Float): Modifier = graphicsLayer().drawWithContent {
    val scene = sky.scene
    val look = scene.look
    if (look <= 0.0005f && scene.punch <= 0f) {
        drawContent()
        return@drawWithContent
    }
    val p = sky.pageCamera(size.width, size.height)
    val at = Projected()
    val top = Proto.H / 2f + min(scrolled() / p.unit, 4000f) + 4f
    val solid = if (p.projectPage(0f, top, at)) at.y else Float.NaN
    val clear = if (p.projectPage(0f, top + 170f, at)) at.y else Float.NaN
    if (!solid.isNaN() && solid < size.height) {
        drawRect(paper, Offset(0f, max(0f, solid)), Size(size.width, size.height - max(0f, solid)))
        if (!clear.isNaN() && clear < solid) {
            drawRect(Brush.verticalGradient(listOf(paper.copy(alpha = 0f), paper), clear, solid), Offset(0f, clear), Size(size.width, solid - clear))
        }
    }
    // Once the page's top has dropped below the screen there is nothing of it to draw.
    if (!solid.isNaN() && solid > size.height) return@drawWithContent
    val src = TILT_SRC.get()!!
    val dst = TILT_DST.get()!!
    src[0] = 0f; src[1] = 0f; src[2] = size.width; src[3] = 0f; src[4] = size.width; src[5] = size.height; src[6] = 0f; src[7] = size.height
    for (k in 0 until 4) {
        val x = src[k * 2] / p.unit - p.w / 2f
        val y = Proto.H / 2f - src[k * 2 + 1] / p.unit
        if (!p.projectPage(x, y, at)) return@drawWithContent
        dst[k * 2] = at.x
        dst[k * 2 + 1] = at.y
    }
    val matrix = TILT_MATRIX.get()!!
    matrix.setPolyToPoly(src, 0, dst, 0, 4)
    val canvas = drawContext.canvas.nativeCanvas
    canvas.save()
    canvas.concat(matrix)
    drawContent()
    canvas.restore()
}

private val TILT_SRC = ThreadLocal.withInitial { FloatArray(8) }
private val TILT_DST = ThreadLocal.withInitial { FloatArray(8) }
private val TILT_MATRIX = ThreadLocal.withInitial { Matrix() }

/**
 * The sky itself, behind the page: the dome, the stars and constellations, and the find a star condenses into. Runs
 * the sky's clock while anything in it moves.
 *
 * @param sky The sky.
 * @param plan What the sky shows, from [skyPlan].
 * @param finds The sky's finds the library holds, by id, for their pictures and titles.
 * @param paused True while something covers the sky, such as a find opened from it; its clock then stops.
 * @param onOpenFind Opens a find from its star.
 * @param onSeen Called with a find's id once its star has lit in front of the person.
 * @param onLookUp Called as the camera starts to look up.
 * @param modifier Modifier for the sky.
 * @param onCompleted Called once a finished constellation has taken its place, for the sky to be read again with the
 *   next one.
 */
@Composable
fun SkyBackdrop(
    sky: SkyState,
    plan: SkyPlanHolder,
    finds: Map<String, Card>,
    paused: Boolean,
    onOpenFind: (String) -> Unit,
    onSeen: (String) -> Unit,
    onLookUp: () -> Unit,
    modifier: Modifier = Modifier,
    onCompleted: () -> Unit = {},
) {
    val scene = sky.scene
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val latestFinds by rememberUpdatedState(finds)
    SideEffect {
        scene.scope = scope
        scene.onOpenFind = onOpenFind
        scene.onSeen = onSeen
        scene.onLookUp = onLookUp
        scene.onCompleted = onCompleted
        scene.picture = { id -> latestFinds[id]?.let { card -> sky.loadPicture(context, card) ?: findFigureArt(findFigureKind(card)) } }
        scene.title = { id -> latestFinds[id]?.let(::findTitle).orEmpty() }
    }
    LaunchedEffect(scene, plan.plan) { scene.show(plan.plan) }
    LaunchedEffect(scene, paused) {
        if (paused) return@LaunchedEffect
        var last = 0L
        while (isActive) {
            if (!scene.moving) {
                last = 0L
                val kicks = scene.kicks.intValue
                val started = scene.time.started.intValue
                snapshotFlow {
                    scene.look > 0f || scene.pull > 0f || scene.punch > 0f || scene.kicks.intValue != kicks || scene.time.started.intValue != started
                }.first { it }
            }
            withFrameNanos { now ->
                val dt = if (last == 0L) 1f / 60f else (now - last) / 1e9f
                last = now
                scene.frame(dt)
            }
        }
    }
    val measurer = rememberTextMeasurer()
    val label = remember { TextStyle(fontSize = 15.sp, lineHeight = 20.sp, fontWeight = FontWeight.Medium, textAlign = TextAlign.Center) }
    val about = remember { TextStyle(fontSize = 12.sp, lineHeight = 16.sp, textAlign = TextAlign.Center) }
    val skyOn by animateFloatAsState(if (scene.skyOn) 1f else 0f, tween(800), label = "sky on")
    Box(modifier.fillMaxSize().onSizeChanged { sky.unit = it.height / Proto.H; scene.viewW = it.width / sky.unit }) {
        Dome(scene)
        Canvas(Modifier.fillMaxSize().graphicsLayer()) {
            scene.frames.intValue
            if (scene.look <= 0.001f) return@Canvas
            sky.painter.draw(drawContext.canvas.nativeCanvas, scene, size.width, size.height)
            // The middle constellation's name under it, typed in as it forms, gold once it is finished.
            val shown = scene.labelOp * skyOn
            val name = scene.labelName.take(scene.labelTyped)
            val place = sky.painter.labelAt(scene)
            if (shown > 0.01f && name.isNotEmpty() && place != null) {
                val done = scene.labelDone
                val width = 300.dp.toPx().toInt()
                val color = lerp(Color(237, 232, 220).copy(alpha = 0.66f), Color(0xFFFFE9B8), done)
                val glow = Shadow(Color(255, 196, 100).copy(alpha = 0.55f * done), Offset.Zero, 18.dp.toPx())
                val dark = Shadow(Color.Black.copy(alpha = 0.7f), Offset(0f, 1.dp.toPx()), 10.dp.toPx())
                val measured = measurer.measure(name, label.copy(color = color, shadow = dark), constraints = Constraints.fixedWidth(width))
                val x = place.first - width / 2f
                if (done > 0.01f) {
                    val gold = measurer.measure(name, label.copy(color = color, shadow = glow), constraints = Constraints.fixedWidth(width))
                    drawText(gold, topLeft = Offset(x, place.second), alpha = shown)
                }
                drawText(measured, topLeft = Offset(x, place.second), alpha = shown)
                val words = scene.middle?.figure?.id?.let(::aboutOf).orEmpty()
                if (words.isNotEmpty() && scene.labelTyped >= scene.labelName.length) {
                    val line = measurer.measure(words, about.copy(color = Color(237, 232, 220).copy(alpha = 0.5f), shadow = dark), constraints = Constraints.fixedWidth(width))
                    drawText(line, topLeft = Offset(x, place.second + measured.size.height + 2.dp.toPx()), alpha = shown)
                }
            }
        }
    }
}

/** A [SkyPlan] for [SkyBackdrop], compared by what it holds. */
class SkyPlanHolder internal constructor(internal val plan: SkyPlan) {
    override fun equals(other: Any?): Boolean = other is SkyPlanHolder && other.plan == plan
    override fun hashCode(): Int = plan.hashCode()
}

// The dome: the shader drawn at half size, since the nebula is soft, and only again when the camera moves.
@Composable
private fun Dome(scene: SkyScene) {
    if (SkyShaders.supported) DomeShader(scene) else DomePicture(scene)
}

@RequiresApi(Build.VERSION_CODES.TIRAMISU)
@Composable
private fun DomeShader(scene: SkyScene) {
    val shader = remember { SkyShaders.dome() }
    val paint = remember { Paint() }
    Canvas(
        Modifier.fillMaxSize(0.5f).graphicsLayer {
            scaleX = 2f
            scaleY = 2f
            transformOrigin = TransformOrigin(0f, 0f)
            compositingStrategy = CompositingStrategy.Offscreen
        },
    ) {
        val look = scene.look
        if (look <= 0.001f) return@Canvas
        drawDome(drawContext.canvas.nativeCanvas, shader, paint, size.width, size.height, look, scene.domeSpin)
    }
}

// Before Android 13 the nebula is a picture painted once, faded in as the camera looks up.
@Composable
private fun DomePicture(scene: SkyScene) {
    val picture = remember { androidx.compose.runtime.mutableStateOf<androidx.compose.ui.graphics.ImageBitmap?>(null) }
    LaunchedEffect(Unit) { picture.value = Nebula.picture() }
    Canvas(Modifier.fillMaxSize()) {
        val look = scene.look
        if (look <= 0.001f) return@Canvas
        drawRect(Color(0xFF05080F), alpha = sstep(look, 0f, 0.3f))
        val image = picture.value ?: return@Canvas
        drawImage(image, dstSize = androidx.compose.ui.unit.IntSize(size.width.toInt(), size.height.toInt()), alpha = sstep(look, 0.2f, 1f))
    }
}

/**
 * What lies over the sky: the pull's star while pulling, and once looking up the sky's title, the find's title under
 * its star, Back, and the touches that tap stars, swipe between them and swipe up to come back down.
 *
 * @param sky The sky.
 * @param modifier Modifier for the layer.
 */
@Composable
fun BoxScope.SkyOverlay(sky: SkyState, modifier: Modifier = Modifier) {
    val scene = sky.scene
    PullHint(scene, Modifier.align(Alignment.TopCenter).padding(top = 64.dp))
    SkyImmersion(sky)
    if (!scene.lookingUp) return
    BackHandler { scene.back() }
    val skyOn by animateFloatAsState(if (scene.skyOn) 1f else 0f, tween(800), label = "sky title")
    val mem by animateFloatAsState(if (scene.memOn) 1f else 0f, tween(500), label = "find title")
    val rise by animateFloatAsState(if (scene.memOn) 0f else 1f, tween(800, easing = CubicBezierEasing(0.2f, 1f, 0.3f, 1f)), label = "find rise")
    // Read here so what accessibility offers changes as a finished constellation comes back and goes.
    val visiting = scene.visiting
    Box(
        modifier.fillMaxSize()
            .pointerInput(scene) {
                awaitEachGesture {
                    val down = awaitFirstDown(requireUnconsumed = false)
                    val u = sky.unit
                    var x = down.position.x
                    var y = down.position.y
                    scene.pointerDown(x / u, y / u, down.uptimeMillis)
                    down.consume()
                    var ended = false
                    try {
                        while (true) {
                            val event = awaitPointerEvent()
                            val change = event.changes.firstOrNull { it.id == down.id } ?: break
                            x = change.position.x
                            y = change.position.y
                            change.consume()
                            if (!change.pressed) break
                            scene.pointerMove(x / u, y / u, change.uptimeMillis)
                        }
                    } finally {
                        if (!ended) {
                            ended = true
                            val w = size.width.toFloat()
                            val h = size.height.toFloat()
                            scene.pointerUp(
                                x / u, y / u, w / u,
                                hitStar = { sx, sy -> sky.painter.starAt(scene, sx * u, sy * u, w, h) },
                                hitCon = { sx, sy -> sky.painter.finishedAt(scene, sx * u, sy * u, w, h) },
                            )
                        }
                    }
                }
            }
            .semantics {
                contentDescription = "Your sky"
                customActions = skyActions(scene, visiting)
            },
    ) {
        Box(Modifier.fillMaxWidth().padding(start = 20.dp, top = 12.dp).height(44.dp).graphicsLayer { alpha = skyOn }, contentAlignment = Alignment.CenterStart) {
            Text("Your sky", style = LibraryStyle.title.copy(color = Color(0xFFEDE8DC), shadow = Shadow(Color.Black.copy(alpha = 0.6f), Offset(0f, 6f), 42f)))
        }
        Text(
            scene.memTitle,
            Modifier.align(Alignment.BottomCenter).fillMaxWidth().padding(start = 28.dp, end = 28.dp, bottom = 64.dp)
                .graphicsLayer { alpha = mem; translationY = rise * 10.dp.toPx() },
            style = TextStyle(fontSize = 22.sp, lineHeight = 28.sp, fontWeight = FontWeight.Medium, textAlign = TextAlign.Center,
                color = Color(0xFFF4EFE4), shadow = Shadow(Color.Black.copy(alpha = 0.85f), Offset(0f, 6f), 36f)),
            maxLines = 3,
        )
    }
}

// What accessibility offers in the sky: each star of the middle constellation by its find, each finished constellation
// to bring back, and while a star's find shows, open it, the next and previous stars, and back out.
private fun skyActions(scene: SkyScene, visiting: ConState?): List<CustomAccessibilityAction> {
    val con = scene.middle
    val focused = scene.focusedCard
    val actions = ArrayList<CustomAccessibilityAction>()
    if (con != null && focused != null) {
        val i = scene.focusedIndex
        val n = con.stars.size
        actions += CustomAccessibilityAction("Open ${scene.memTitle}") { scene.openFromSky(); true }
        actions += CustomAccessibilityAction("Next star") { scene.glide((i + 1) % n); true }
        actions += CustomAccessibilityAction("Previous star") { scene.glide((i - 1 + n) % n); true }
        actions += CustomAccessibilityAction("Back to the whole sky") { scene.back(); true }
    } else {
        con?.stars?.forEachIndexed { i, star ->
            val lit = if (star.lit) "a lit star" else "a dark star"
            actions += CustomAccessibilityAction("${scene.title(star.id).ifEmpty { "A find" }}, $lit") { scene.focusStar(i); true }
        }
        scene.finished.forEachIndexed { index, done ->
            if (done.id != visiting?.id) actions += CustomAccessibilityAction("${done.name}, finished") { scene.visit(index); true }
        }
        val back = if (visiting != null) "Back to ${scene.active?.name ?: "the whole sky"}" else "Back to your library"
        actions += CustomAccessibilityAction(back) { scene.back(); true }
    }
    return actions
}

// The star that shows while pulling down at the top: faint at first, gold once letting go would look up.
@Composable
private fun PullHint(scene: SkyScene, modifier: Modifier) {
    val pull = scene.pull
    val ready = pull > SkyScene.PULL_TO_LOOK
    val grow by animateFloatAsState(if (ready) 1.25f else 1f, spring(dampingRatio = 0.45f, stiffness = 700f), label = "pull star")
    if (scene.lookingUp || pull <= 0f) return
    val star = remember { PathParser().parsePathString(PULL_STAR).toPath() }
    Canvas(modifier.size(22.dp).graphicsLayer { alpha = (pull / 90f).coerceIn(0f, 1f); scaleX = grow; scaleY = grow }) {
        val k = size.width / 24f
        scale(k, k, Offset.Zero) {
            if (ready) drawPath(star, Color(0xFFE9B949), style = Fill)
            drawPath(star, if (ready) Color(0xFFB98522) else if (pull > 50f) Color(0xFFFFF4DC) else Color(0xFF716E64),
                style = Stroke(1.7f, join = StrokeJoin.Round))
        }
    }
}

private const val PULL_STAR = "M12 2.5l2.2 6.6 6.8.1-5.5 4.1 2 6.6-5.5-4-5.5 4 2-6.6L3 9.2l6.8-.1z"

// While looking up, the shell's bar slides away and the status bar takes the sky's colour.
@Composable
private fun SkyImmersion(sky: SkyState) {
    val immersion = LocalShellImmersion.current ?: return
    DisposableEffect(immersion, sky) {
        val removePage = immersion.addPage { sstep(sky.scene.look, 0.005f, 0.09f) }
        val removeStrip = immersion.addStrip { SKY_TOP.copy(alpha = sstep(sky.scene.look, 0.02f, 0.3f)) }
        onDispose {
            removePage()
            removeStrip()
        }
    }
}

/**
 * What the sky shows for [sky]: the constellation forming in the middle, finished ones in their places, finds opened on
 * their own as loose stars, and the finds waiting to light. A constellation finished while the sky was away shows in
 * the middle first, so its last stars light there before it takes its place.
 *
 * @param sky The day's sky, or null.
 * @param seen The finds whose stars the person has seen light, or null before that is known.
 * @param arriving Finds opened from Back today whose star has not reached the star button yet.
 * @return The plan.
 */
internal fun skyPlan(sky: Sky?, seen: Set<String>?, arriving: Set<String>): SkyPlanHolder {
    if (sky == null) return SkyPlanHolder(SkyPlan())
    val visits = sky.stars.map { it.id }.distinct()
    val pending = if (seen == null) emptyList() else visits.filter { it !in seen && it !in arriving }
    val pendingSet = pending.toHashSet()
    fun plan(c: Constellation, done: Boolean) = ConPlan(c.id, c.figure, c.name, c.cards, c.lit.toSet() - arriving, done)
    val waiting = sky.done.filter { c -> c.cards.any { it in pendingSet } }
    val settled = sky.done.filter { it !in waiting }
    val center = waiting.firstOrNull()?.let { plan(it, true) } ?: sky.active?.let { plan(it, false) }
    val next = waiting.drop(1).map { plan(it, true) } + listOfNotNull(sky.active?.takeIf { waiting.isNotEmpty() }?.let { plan(it, false) })
    val inConstellations = (sky.done.flatMap { it.cards } + sky.active?.cards.orEmpty()).toHashSet()
    val loose = visits.filter { it !in inConstellations && it !in arriving && it !in pendingSet }.takeLast(MAX_LOOSE)
    return SkyPlanHolder(SkyPlan(center, next, settled.map { plan(it, true) }, loose, pending))
}

/** What the sky calls a find: its title, else the first line of its words, cut to fit one line under its star. */
internal fun findTitle(card: Card): String {
    val text = card.title?.takeIf { it.isNotBlank() }
        ?: galleryText(card).lineSequence().firstOrNull { it.isNotBlank() }
        ?: card.siteName?.takeIf { it.isNotBlank() }
        ?: cardHeading(card)
    val line = text.trim().replace(Regex("\\s+"), " ")
    return if (line.length > TITLE_CHARS) line.take(TITLE_CHARS - 1).trimEnd() + "…" else line
}

private const val TITLE_CHARS = 46

package com.octaviantocan.mymind.ui.moment

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.theme.AppFontFamily
import java.text.NumberFormat
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.roundToInt

/**
 * A shooting star moment as it plays: its simulation once the screen is measured, and its sounds and feel.
 *
 * @param sound The sound output, or null for none.
 * @param haptics The vibration motor, or null for none.
 * @param seed Seed for the particles, fixed in tests.
 */
@Stable
class StarMoment internal constructor(private val sound: MomentSound?, private val haptics: MomentHaptics?, private val seed: Long) {
    internal var sim by mutableStateOf<StarSim?>(null)
        private set

    /** The find's rarity once the library is counted; the star's colours follow it if it is known before it breaks off. */
    var rarity: Rarity? = null
        set(value) { field = value; sim?.rarity = value }

    /** The rarity the moment shows: the counted one, or Common once the star broke off without it. */
    val shownRarity: Rarity get() = sim?.shownRarity ?: rarity ?: Rarity.Common

    /** Where the moment is. */
    val phase: StarPhase get() = sim?.phase ?: StarPhase.Playing

    /** True once a tap skipped the moment, which skips the tag step too. */
    val skipped: Boolean get() = sim?.skipped == true

    internal fun begin(width: Float, height: Float, top: Float) {
        if (sim != null) return
        sim = StarSim(width, height, top, seed) { cue -> sound?.play(cue); haptics?.feel(cue) }.also { it.rarity = rarity }
        sound?.start()
    }

    /** Skips to the backpack, as a tap does while it plays. */
    fun skip() { sim?.skip() }

    /** Turns the card face up again and flies it into the backpack, once the tag step is done. */
    fun closeTags() { sim?.turnToFace() }

    /** Fades it all away without the backpack, when nothing was saved after all. */
    fun abort() { sim?.abort() }

    internal fun close() { sound?.stop() }
}

/**
 * @param seed Seed for the particles.
 * @return A moment that plays its sounds and feel on this phone.
 */
@Composable
fun rememberStarMoment(seed: Long = System.nanoTime()): StarMoment {
    val context = LocalContext.current.applicationContext
    val moment = remember { StarMoment(MomentSound(context), MomentHaptics(context), seed) }
    DisposableEffect(moment) { onDispose { moment.close() } }
    return moment
}

/**
 * The shooting star moment over the app that shared (Tavi's pick of 03, 5 November 2026; share-moment-shooting-star):
 * the page goes under a night sky, a constellation in the shape of the gem draws itself, one star breaks off and falls
 * changing colour by rarity, lands in a white-out, and the find's card spins out of it face up. Its stars stamp in and
 * its points count up; then the card turns over and grows into [tagStep], and once that is done it turns back and flies
 * into the backpack at the top while the page comes back. A tap while it plays skips straight to the backpack.
 *
 * @param moment The moment.
 * @param find What the card shows.
 * @param backpack How many finds the library held before this one, for the backpack's count, or null if unknown.
 * @param saved How many finds this share added.
 * @param tagOpen True while the tag step should show, which turns the settled card over.
 * @param onOutside Called for a tap on the sky while the tag step shows.
 * @param onSky Called each frame with how much of the sky covers the page, from 0 to 1, for blurring the page behind.
 * @param tagStep The tag step, drawn on the card's back at the size it is given.
 */
@Composable
fun ShootingStar(
    moment: StarMoment,
    find: MomentFind,
    backpack: Int?,
    saved: Int,
    tagOpen: Boolean,
    onOutside: () -> Unit,
    onSky: (Float) -> Unit,
    modifier: Modifier = Modifier,
    tagStep: @Composable (Modifier) -> Unit,
) {
    BoxWithConstraints(modifier.fillMaxSize()) {
        val density = LocalDensity.current
        val wPx = constraints.maxWidth.toFloat()
        val hPx = constraints.maxHeight.toFloat()
        val unit = min(wPx / 360f, hPx / 760f)
        val topPx = WindowInsets.statusBars.getTop(density).toFloat()
        val navPx = WindowInsets.navigationBars.getBottom(density).toFloat()
        val imePx = WindowInsets.ime.getBottom(density).toFloat()
        LaunchedEffect(Unit) { moment.begin(wPx / unit, hPx / unit, topPx / unit) }
        val sim = moment.sim ?: return@BoxWithConstraints
        val latestSky by rememberUpdatedState(onSky)
        LaunchedEffect(sim) {
            var last = withFrameNanos { it }
            while (sim.phase != StarPhase.Done) {
                withFrameNanos { now -> sim.step((now - last) / 1_000_000f); last = now }
                latestSky(sim.skyA * sim.stage)
            }
            latestSky(0f)
        }
        // M9's card is the find's card grown to fill the screen, kept to the card's own shape.
        val gap = with(density) { 18.dp.toPx() }
        val gridW = min(min(wPx - gap * 2f, (hPx - topPx - navPx - gap * 2f) * CARD_RATIO), with(density) { 440.dp.toPx() })
        val gridH = gridW / CARD_RATIO
        val bigScale = gridH / (284f * unit)
        val bigY = (topPx + (hPx - topPx - navPx) / 2f) / unit
        LaunchedEffect(tagOpen, sim.phase) { if (tagOpen && sim.phase == StarPhase.Settled) sim.turnToBack(bigY, bigScale) }
        val units = remember(unit, density) { CardUnits(unit, density.density, density.fontScale) }
        val latestOutside by rememberUpdatedState(onOutside)

        Box(
            Modifier.fillMaxSize()
                .pointerInput(sim) { detectTapGestures { if (sim.phase == StarPhase.Playing) sim.skip() else if (sim.showingBack) latestOutside() } }
                .semantics { contentDescription = "Saved. Tap to skip" },
        ) {
            Box(
                Modifier.fillMaxSize().graphicsLayer {
                    sim.frame
                    scaleX = sim.camScale; scaleY = sim.camScale
                    translationX = sim.shakeX * unit
                    translationY = (sim.camY + sim.shakeY) * unit
                    rotationZ = sim.shakeRot
                },
            ) {
                val painter = remember { StarPaint() }
                Canvas(Modifier.fillMaxSize()) {
                    sim.frame
                    drawIntoCanvas { c -> c.nativeCanvas.apply { save(); scale(unit, unit); painter.back(this, sim); restore() } }
                }
                FaceCard(sim, find, units, unit, density.density)
                if (sim.cardOn && sim.showingBack) {
                    val lifted = imePx > 0f
                    tagStep(
                        Modifier.offset { if (lifted) IntOffset(((wPx - gridW) / 2f).roundToInt(), (topPx + gap / 2f).roundToInt()) else IntOffset.Zero }
                            .size(with(density) { gridW.toDp() }, with(density) { (if (lifted) min(gridH, hPx - imePx - topPx - gap) else gridH).toDp() })
                            .graphicsLayer {
                                sim.frame
                                if (!lifted) {
                                    val s = 284f * sim.cardScale * unit / gridH
                                    translationX = sim.cardX * unit - gridW / 2f
                                    translationY = sim.cardY * unit - gridH / 2f
                                    scaleX = s; scaleY = s
                                    rotationY = sim.turnDeg - 180f
                                    cameraDistance = 14f * density.density * 72f
                                }
                            },
                    )
                }
                Canvas(Modifier.fillMaxSize()) {
                    sim.frame
                    drawIntoCanvas { c -> c.nativeCanvas.apply { save(); scale(unit, unit); painter.front(this, sim); restore() } }
                }
                Points(sim, units, unit)
            }
            Backpack(sim, units, unit, backpack, saved)
        }
    }
}

/** The find's card, face up or face down as it turns. */
@Composable
private fun FaceCard(sim: StarSim, find: MomentFind, units: CardUnits, unit: Float, density: Float) {
    if (!sim.cardOn || sim.showingBack) return
    val rarity = sim.shownRarity
    Box(
        Modifier.size(units.dp(200f), units.dp(284f)).graphicsLayer {
            sim.frame
            translationX = (sim.cardX - 100f) * unit
            translationY = (sim.cardY - 142f) * unit
            scaleX = sim.cardScale; scaleY = sim.cardScale
            rotationX = Math.toDegrees(-sim.cardRx.toDouble()).toFloat()
            rotationY = Math.toDegrees((sim.cardFlip + sim.cardRy).toDouble()).toFloat() + sim.turnDeg
            rotationZ = sim.cardRz
            alpha = sim.cardA
            cameraDistance = 14f * density * 72f
            transformOrigin = TransformOrigin.Center
        },
    ) {
        val faceUp = { cos(sim.cardFlip + Math.toRadians(sim.turnDeg.toDouble()).toFloat()) * cos(sim.cardRx) > 0f }
        // Both sides stay composed; which one shows follows the turn each frame without composing again.
        FindFront(find, rarity, sim.stars, units, light = {
            floatArrayOf(kotlin.math.sin(sim.cardRy), kotlin.math.sin(sim.cardRx), sim.sheen, sim.sheenA)
        }, modifier = Modifier.graphicsLayer { sim.frame; alpha = if (faceUp()) 1f else 0f })
        FindBack(units, Modifier.graphicsLayer { sim.frame; rotationY = 180f; alpha = if (faceUp()) 0f else 1f })
    }
}

/** The points counting up under the card. */
@Composable
private fun Points(sim: StarSim, units: CardUnits, unit: Float) {
    if (!sim.pointsOn) return
    val r = sim.shownRarity
    Row(
        Modifier.fillMaxWidth().graphicsLayer {
            sim.frame
            translationY = (sim.cardY + 164f + sim.pointsDy) * unit - units.px(16f)
            alpha = sim.pointsA
            scaleX = sim.pointsScale; scaleY = sim.pointsScale
        },
        horizontalArrangement = Arrangement.spacedBy(units.dp(8f), Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Gem(units, 24f, gemTones(r))
        Text(
            "+${sim.pointsValue}",
            style = TextStyle(color = r.color, fontSize = units.sp(26f), fontWeight = FontWeight.Bold, fontFamily = AppFontFamily,
                shadow = Shadow(r.color.copy(alpha = 0.6f), Offset.Zero, units.px(18f))),
        )
    }
}

/** The backpack at the top right: its count of finds, and a label with the points that slides out from it. */
@Composable
private fun Backpack(sim: StarSim, units: CardUnits, unit: Float, before: Int?, saved: Int) {
    if (!sim.bagOn) return
    val r = sim.shownRarity
    val bag = remember { PathParser().parsePathString(BAG).toPath() }
    Box(
        Modifier.offset { IntOffset(((sim.bagX + 24f) * unit - units.px(48f) - units.px(140f)).roundToInt(), ((sim.bagY - 24f) * unit).roundToInt()) }
            .graphicsLayer {
                sim.frame
                alpha = sim.bagA
                translationY = sim.bagDy * unit
            },
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(units.dp(140f + 14f), units.dp(48f)), contentAlignment = Alignment.CenterEnd) {
                Row(
                    Modifier.offset(x = units.dp(14f)).drawWithContent {
                        sim.frame
                        val p = sim.bagLabel
                        drawContext.canvas.save()
                        drawContext.canvas.clipRect(size.width * (1f - p), 0f, size.width, size.height)
                        drawContent()
                        drawContext.canvas.restore()
                    }.clip(RoundedCornerShape(topStart = units.dp(16f), bottomStart = units.dp(16f))).background(Color(0xEB0B2227))
                        .padding(start = units.dp(12f), end = units.dp(22f), top = units.dp(7f), bottom = units.dp(7f)),
                    horizontalArrangement = Arrangement.spacedBy(units.dp(6f)),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Gem(units, 12f, gemTones(r))
                    Text("+${r.points}", style = TextStyle(color = r.color, fontSize = units.sp(12.5f), fontWeight = FontWeight.SemiBold, fontFamily = AppFontFamily))
                }
            }
            Box(
                Modifier.size(units.dp(48f)).graphicsLayer { sim.frame; scaleX = sim.bagScale; scaleY = sim.bagScale }
                    .drawBehind {
                        drawCircle(r.color.copy(alpha = 0.45f), size.width * 0.75f, blendMode = androidx.compose.ui.graphics.BlendMode.Plus, alpha = 0.35f)
                    }
                    .clip(RoundedCornerShape(units.dp(16f)))
                    .background(Brush.radialGradient(listOf(Color(0xFF1E555D), Color(0xFF0B2227)), center = Offset(units.px(14f), units.px(10f)), radius = units.px(56f)))
                    .drawWithContent {
                        drawContent()
                        drawRoundRect(r.color, cornerRadius = androidx.compose.ui.geometry.CornerRadius(units.px(16f)), style = Stroke(units.px(2f)))
                        val k = units.px(26f) / 24f
                        translate(units.px(11f), units.px(11f)) {
                            scale(k, k, pivot = Offset.Zero) { drawPath(bag, Color(0xFFF4F2EF), style = Stroke(1.6f, cap = StrokeCap.Round, join = StrokeJoin.Round)) }
                        }
                    },
            )
        }
        if (before != null) {
            val count = before + if (sim.bagCounted) saved else 0
            Text(
                NumberFormat.getIntegerInstance().format(count),
                Modifier.align(Alignment.BottomEnd).offset(x = -units.dp(24f) + units.dp(0f), y = units.dp(9f)).graphicsLayer { translationX = size.width / 2f }
                    .clip(CircleShape).background(Color(0xFFF4F2EF)).padding(horizontal = units.dp(6f)),
                style = TextStyle(color = Color(0xFF0E333A), fontSize = units.sp(10.5f), fontWeight = FontWeight.Bold, fontFamily = AppFontFamily),
            )
        }
    }
}

/** The card's width over its height, kept by the tag step's card too. */
internal const val CARD_RATIO = 200f / 284f

private const val BAG = "M9.5 4.5V3.8a2.5 2.5 0 0 1 5 0v.7M6 9.5a4.5 4.5 0 0 1 4.5-4.5h3A4.5 4.5 0 0 1 18 9.5V19a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2zM6 11.5h12M8.5 15h7v3.5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1zM12 11.5v1.6"


package com.octaviantocan.mymind.ui.sky

import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.drawscope.rotateRad
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.unit.Dp
import kotlin.math.PI
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.pow
import kotlin.math.sin
import kotlin.random.Random

/**
 * One leaf of a cover, in the cover's own units: [x] and [y] from 0 to 1 across its width and height, [length] and
 * [width] as shares of its width, [depth] from 0 at the bottom of the pile to 1 on top.
 */
private class Leaf(
    val x: Float,
    val y: Float,
    val length: Float,
    val width: Float,
    val angle: Float,
    val color: Color,
    val lean: Float,
    val spin: Float,
    val drift: Float,
    val depth: Float,
)

// Dry autumn colours, a little muted, so a pile reads as fallen leaves in both page schemes and not as confetti.
private val LEAF_COLORS = listOf(
    Color(0xFFC08A3E), Color(0xFFD2A04E), Color(0xFFAD5A31), Color(0xFF97482A), Color(0xFF7F3424),
    Color(0xFF7D7535), Color(0xFF8A6440), Color(0xFF6F4C30), Color(0xFFCDB067), Color(0xFFB27735),
)

// The litter under the leaves, so nothing of the find shows between them.
private val LITTER_TOP = Color(0xFF5B3D22)
private val LITTER_BOTTOM = Color(0xFF3A2716)

private const val COLUMNS = 6
private const val ROWS = 8
private const val EXTRA = 26

/**
 * The leaves covering one find, the same each time for the same [seed]: a jittered grid so the pile has no holes,
 * and more on top, from the bottom of the pile up.
 */
private fun leavesFor(seed: String): List<Leaf> {
    val random = Random(seed.hashCode())
    fun leaf(x: Float, y: Float) = Leaf(
        x = x,
        y = y,
        length = 0.24f + random.nextFloat() * 0.12f,
        width = 0.34f + random.nextFloat() * 0.22f,
        angle = (random.nextFloat() * 2f * PI).toFloat(),
        color = LEAF_COLORS[random.nextInt(LEAF_COLORS.size)],
        lean = (random.nextFloat() - 0.5f) * 0.3f,
        spin = (random.nextFloat() - 0.5f) * 9f,
        drift = random.nextFloat(),
        depth = 0f,
    )
    val pile = buildList {
        for (row in 0 until ROWS) for (column in 0 until COLUMNS) {
            add(leaf((column + 0.5f + (random.nextFloat() - 0.5f) * 0.8f) / COLUMNS, (row + 0.5f + (random.nextFloat() - 0.5f) * 0.8f) / ROWS))
        }
        repeat(EXTRA) { add(leaf(random.nextFloat(), random.nextFloat())) }
    }.shuffled(random)
    return pile.mapIndexed { index, it -> Leaf(it.x, it.y, it.length, it.width, it.angle, it.color, it.lean, it.spin, it.drift, index / (pile.size - 1f)) }
}

// A leaf's outline along +x from its stem at the origin to its tip at `length`, `half` wide at its widest.
private fun leafPath(length: Float, half: Float, lean: Float): Path = Path().apply {
    val upper = half * (1f + lean)
    val lower = half * (1f - lean)
    moveTo(0f, 0f)
    cubicTo(length * 0.2f, -upper * 1.2f, length * 0.66f, -upper * 1.0f, length, 0f)
    cubicTo(length * 0.66f, lower * 1.0f, length * 0.2f, lower * 1.2f, 0f, 0f)
    close()
}

// The half of a leaf on one side of its midrib, shaded to show the leaf folding along it.
private fun foldPath(length: Float, half: Float, lean: Float): Path = Path().apply {
    val upper = half * (1f + lean)
    moveTo(0f, 0f)
    cubicTo(length * 0.2f, -upper * 1.2f, length * 0.66f, -upper * 1.0f, length, 0f)
    close()
}

private fun Color.shade(by: Float): Color = if (by >= 0f) lerp(this, Color.White, by) else lerp(this, Color.Black, -by)

private fun smooth(value: Float): Float {
    val t = value.coerceIn(0f, 1f)
    return t * t * (3f - 2f * t)
}

// How far one leaf has blown away, 0 still and 1 gone: the leaves nearest the wind's start go first.
private fun blownOf(leaf: Leaf, progress: Float, origin: Offset, aspect: Float): Float {
    val distance = hypot(leaf.x - origin.x, (leaf.y - origin.y) * aspect)
    val delay = (distance / 1.4f).coerceIn(0f, 0.42f) + leaf.drift * 0.08f
    return ((progress - delay) / 0.5f).coerceIn(0f, 1f)
}

/**
 * Covers the content in a pile of fallen leaves, Back today's look for a find not yet opened, and blows them away as
 * [blown] goes from 0 to 1: the leaves nearest [from] go first, lifting, turning and fading as they fly off past the
 * edges, and the litter under them fades to show the find.
 *
 * At rest the pile stays inside the content's rounded corners; a leaf leaves them once it starts to fly. Nothing is
 * drawn once [blown] reaches 1. The leaves are laid out once per [seed] and size, so a cover never changes between
 * frames or launches.
 *
 * @param seed What the leaves are laid out from, such as the card's id.
 * @param corner The content's corner radius.
 * @param blown How far the leaves have blown away, 0 still and 1 gone.
 * @param from Where the wind starts, as a share of the width and height; the tap that uncovers the find.
 */
fun Modifier.leafCover(seed: String, corner: Dp, blown: () -> Float, from: () -> Offset = { Offset(0.5f, 0.5f) }): Modifier = drawWithCache {
    val aspect = if (size.width > 0f) size.height / size.width else 1.25f
    val unit = size.width
    val leaves = leavesFor(seed)
    val halves = leaves.map { it.width * unit / 2f * 0.62f }
    val paths = leaves.mapIndexed { index, leaf -> leafPath(leaf.length * unit, halves[index], leaf.lean) }
    val folds = leaves.mapIndexed { index, leaf -> foldPath(leaf.length * unit, halves[index], leaf.lean) }
    val brushes = leaves.map { leaf ->
        // Leaves deeper in the pile lie in the shade of those above.
        val base = leaf.color.shade(-0.32f * (1f - leaf.depth))
        val length = leaf.length * unit
        Brush.linearGradient(listOf(base.shade(0.14f), base, base.shade(-0.2f)), start = Offset(0f, -length * 0.25f), end = Offset(length, length * 0.25f))
    }
    val veins = leaves.map { leaf -> leaf.color.shade(-0.45f).copy(alpha = 0.5f) }
    val litter = Brush.verticalGradient(listOf(LITTER_TOP, LITTER_BOTTOM))
    val radius = corner.toPx()
    val outline = Path().apply { addRoundRect(RoundRect(0f, 0f, size.width, size.height, CornerRadius(radius))) }
    val hairline = max(1f, unit * 0.005f)
    onDrawWithContent {
        drawContent()
        val progress = blown().coerceIn(0f, 1f)
        if (progress >= 1f) return@onDrawWithContent
        val origin = from()
        clipPath(outline) {
            drawRect(litter, alpha = 1f - smooth(progress / 0.55f))
            leaves.forEachIndexed { index, leaf ->
                if (blownOf(leaf, progress, origin, aspect) <= 0f) {
                    drawPlacedLeaf(leaf, 0f, origin, aspect, paths[index], folds[index], brushes[index], veins[index], halves[index], hairline)
                }
            }
        }
        leaves.forEachIndexed { index, leaf ->
            val t = blownOf(leaf, progress, origin, aspect)
            if (t > 0f && t < 1f) drawPlacedLeaf(leaf, t, origin, aspect, paths[index], folds[index], brushes[index], veins[index], halves[index], hairline)
        }
    }
}

// One leaf where it lies, or on its way off once `t` is above 0.
private fun DrawScope.drawPlacedLeaf(
    leaf: Leaf, t: Float, origin: Offset, aspect: Float, path: Path, fold: Path, brush: Brush, vein: Color, half: Float, hairline: Float,
) {
    val unit = size.width
    val away = atan2((leaf.y - origin.y) * aspect, leaf.x - origin.x)
    val travel = t.pow(1.7f) * unit * (1.3f + leaf.drift * 0.6f)
    val dx = cos(away) * travel + t * unit * 0.25f
    val dy = sin(away) * travel - t.pow(1.3f) * unit * 0.55f
    val alpha = 1f - smooth((t - 0.55f) / 0.45f)
    if (alpha <= 0f) return
    val length = leaf.length * unit
    translate(leaf.x * size.width + dx, leaf.y * size.height + dy) {
        rotateRad(leaf.angle + leaf.spin * t * t, Offset.Zero) {
            scale(1f + 0.3f * t, Offset.Zero) {
                translate(-length / 2f, 0f) {
                    val lift = 1f + t * 3f
                    translate(hairline * 1.2f * lift, hairline * 2f * lift) { drawPath(path, Color.Black, alpha = alpha * 0.3f / lift) }
                    drawPath(path, brush, alpha = alpha)
                    drawPath(fold, Color.Black, alpha = alpha * 0.14f)
                    drawLine(vein, Offset(-length * 0.1f, 0f), Offset(length * 0.9f, 0f), strokeWidth = hairline, cap = StrokeCap.Round, alpha = alpha)
                    for (rib in 1..4) {
                        val at = length * (0.12f + rib * 0.15f)
                        val reach = half * (0.85f - rib * 0.14f)
                        drawLine(vein, Offset(at, 0f), Offset(at + length * 0.13f, -reach), strokeWidth = hairline * 0.6f, alpha = alpha * 0.7f)
                        drawLine(vein, Offset(at, 0f), Offset(at + length * 0.13f, reach), strokeWidth = hairline * 0.6f, alpha = alpha * 0.7f)
                    }
                }
            }
        }
    }
}

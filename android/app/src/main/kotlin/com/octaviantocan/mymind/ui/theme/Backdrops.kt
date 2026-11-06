package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Canvas
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.ImageShader
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.ShaderBrush
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.drawscope.CanvasDrawScope
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import kotlin.random.Random

/**
 * What a theme paints behind the gallery and the Spaces page instead of one flat colour, as a chat app's wallpaper:
 * a gradient, and over it, for a doodle theme, a repeating pattern of small line drawings that stays still while the
 * cards scroll. Tiles, sheets and opened cards keep their solid colours.
 *
 * @property colors The gradient from the top left corner to the bottom right; one colour paints it flat.
 * @property doodles The drawings scattered over it, or null for a gradient alone.
 * @property ink The drawings' line colour, alpha included, so they stay a faint pattern under the cards.
 */
@Immutable
class Backdrop(val colors: List<Color>, val doodles: DoodleSet? = null, val ink: Color = Color.Transparent) {
    /** The gradient across a box of [size]. */
    fun brush(size: Size): Brush =
        if (colors.size == 1) Brush.linearGradient(listOf(colors[0], colors[0]))
        else Brush.linearGradient(colors, start = Offset.Zero, end = Offset(size.width, size.height))
}

/**
 * Paints the screen behind the gallery in the current theme: its [Backdrop] when it has one, else [color]. The
 * backdrop is painted once for the whole window, behind the status bar too, so it stays still as a wallpaper does while
 * the pages swipe and scroll over it; a page inside the shell passes [backdrop] false and lets it show through.
 *
 * @param color The flat colour a theme without a backdrop paints.
 * @param backdrop False for a page over the shell's backdrop: it paints nothing under a backdrop theme.
 */
@Composable
fun Modifier.themeBackground(color: Color, backdrop: Boolean = true): Modifier {
    val painted = MymindTheme.palette.backdrop ?: return drawBehind { drawRect(color) }
    return if (backdrop) backdropBehind(painted) else this
}

/** Paints [backdrop] behind the content, its doodle tile drawn once and repeated. */
fun Modifier.backdropBehind(backdrop: Backdrop): Modifier = drawWithCache {
    val brush = backdrop.brush(size)
    val tile = backdrop.doodles?.let { doodleTile(it, backdrop.ink, this) }
    onDrawBehind {
        drawRect(brush)
        if (tile != null) drawRect(ShaderBrush(ImageShader(tile, TileMode.Repeated, TileMode.Repeated)))
    }
}

/** How wide one repeat of a doodle pattern is. */
private val TILE = 240.dp

/** The doodles across one tile: a jittered 4 by 4 grid, so the pattern looks scattered yet nothing touches. */
private const val GRID = 4

/** One repeat of [set] drawn in [ink] at the screen's density. */
private fun doodleTile(set: DoodleSet, ink: Color, density: Density): ImageBitmap {
    val px = with(density) { TILE.toPx() }.roundToInt().coerceAtLeast(1)
    val bitmap = ImageBitmap(px, px)
    CanvasDrawScope().draw(density, LayoutDirection.Ltr, Canvas(bitmap), Size(px.toFloat(), px.toFloat())) {
        drawDoodles(set, ink, px.toFloat())
    }
    return bitmap
}

/** Draws [set] across a [tile]-wide square; the same set always lands the same way. */
private fun DrawScope.drawDoodles(set: DoodleSet, ink: Color, tile: Float) {
    val random = Random(set.seed)
    val cell = tile / GRID
    val stroke = Stroke(width = 1.6.dp.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round)
    val shapes = set.shapes.map(::parse)
    val order = List(GRID * GRID) { it % shapes.size }.shuffled(random)
    for (i in 0 until GRID * GRID) {
        val side = cell * (0.46f + random.nextFloat() * 0.12f)
        val slack = (cell - side) / 2f
        val x = (i % GRID) * cell + slack + (random.nextFloat() * 2f - 1f) * slack * 0.8f
        val y = (i / GRID) * cell + slack + (random.nextFloat() * 2f - 1f) * slack * 0.8f
        val turn = random.nextFloat() * 50f - 25f
        translate(x, y) {
            rotate(turn, pivot = Offset(side / 2f, side / 2f)) {
                // The shapes are drawn on a 24 unit square; the stroke is divided back so every line is one width.
                scale(side / 24f, side / 24f, pivot = Offset.Zero) {
                    drawPath(shapes[order[i]], ink, style = Stroke(stroke.width * 24f / side, cap = stroke.cap, join = stroke.join))
                }
            }
        }
    }
}

private fun parse(data: String): Path = PathParser().parsePathString(data).toPath()

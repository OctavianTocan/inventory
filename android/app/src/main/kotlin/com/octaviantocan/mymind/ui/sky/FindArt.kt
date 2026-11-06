package com.octaviantocan.mymind.ui.sky

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import com.octaviantocan.mymind.data.Card

/**
 * A line figure for what a find is about, for a find with no picture, as the prototype draws one (white light on black,
 * 512 square): `M x y`, `L x y`, `Q`, `C`, `O x y r` a circle, `E x y rx ry` a tilted ellipse, `Z`; x and y from -1 to
 * 1, y up. [dots] are the bright points on it.
 */
private class FindFigure(val paths: List<String>, val dots: List<Pair<Float, Float>>)

private val FIGURES_BY_KIND = mapOf(
    "thread" to FindFigure(
        listOf("M -1 .08 Q -.62 .34 -.24 .14 Q .12 -.02 .46 .18 Q .76 .34 1 .1", "M -.82 -1 C -.1 -.64 -.6 -.3 -.07 -.05 Q .01 .05 .01 .13", "M .4 -1 C .66 -.62 -.1 -.32 .11 -.05 Q .07 .05 .04 .13", "O .58 .6 .15"),
        listOf(.58f to .6f, -.82f to -1f, .4f to -1f, .02f to .13f),
    ),
    "audio" to FindFigure(
        listOf("M -.5 -.7 L -.5 .32 Q -.5 .86 0 .86 Q .5 .86 .5 .32 L .5 -.7 Z", "M 0 -.7 L 0 .86", "M -.5 .08 L .5 .08", "M -.68 -.7 L .68 -.7", "M .56 .32 L .98 -.12", "M .56 -.06 L .98 -.5", "M .56 -.44 L .9 -.84"),
        listOf(0f to .86f, -.5f to -.7f, .5f to -.7f),
    ),
    "note" to FindFigure(
        listOf("M -.46 -.84 L -.46 .86 L .46 .86 L .46 -.84", "M -.46 .86 L -.06 .7 L -.06 -.7 L -.46 -.84", "M -1 -.84 L 1 -.84", "M -.06 -.7 L .72 -1", "M .46 -.84 L .98 -1"),
        listOf(-.14f to 0f, .46f to .86f, -.46f to .86f),
    ),
    "quote" to FindFigure(
        listOf("M -1 -.6 L -.44 .36 L -.18 0 L .22 .66 L 1 -.6", "M .06 .4 L .22 .66 L .36 .44", "M -.7 -.96 L -.7 -.52", "M -.96 -.56 L -.7 -.1 L -.44 -.56 Z", "M -.9 -.3 L -.7 .1 L -.5 -.3", "M .12 -1 Q .34 -.76 .16 -.56 Q 0 -.42 .22 -.3"),
        listOf(.22f to .66f, -.44f to .36f, -.7f to .1f),
    ),
    "repo" to FindFigure(
        listOf("M -.82 -.42 L -.36 .1 L .04 -.16 L .46 .46 L .84 .16", "M .04 -.16 L .26 -.72"),
        listOf(-.82f to -.42f, -.36f to .1f, .04f to -.16f, .46f to .46f, .84f to .16f, .26f to -.72f),
    ),
    "pdf" to FindFigure(
        listOf("M -.82 -.6 L -.82 .6 L -.27 .76 L .27 .6 L .82 .76 L .82 -.44 L .27 -.6 L -.27 -.44 Z", "M -.27 .76 L -.27 -.44", "M .27 .6 L .27 -.6", "M -.62 -.24 Q -.32 .42 0 .02 Q .32 -.38 .6 .38"),
        listOf(-.62f to -.24f, .6f to .38f),
    ),
    "app" to FindFigure(
        listOf("M -.4 -.86 L -.4 .86 L .4 .86 L .4 -.86 Z", "M -.12 -.72 L .12 -.72", "M .2 .98 L .62 .62"),
        listOf(.66f to .58f, -.1f to .3f, .12f to .06f, -.18f to -.12f),
    ),
)

/** Which figure a find with no picture is drawn as, by its type, as the prototype picks one by kind. */
internal fun findFigureKind(card: Card): String = when (card.type) {
    "Note", "Content" -> "note"
    "Quote", "Quotation" -> "quote"
    "XPost", "ThreadsPost", "SubstackNote", "RedditPost", "BlueskyPost", "MastodonPost" -> "thread"
    "Audio" -> "audio"
    "SoftwareApplication" -> "app"
    "Document" -> "pdf"
    else -> if (card.blob?.type?.startsWith("audio") == true) "audio" else "repo"
}

private val figureCache = HashMap<String, Bitmap>()

/** The 512 square line figure for [kind], drawn once a process. */
internal fun findFigureArt(kind: String): Bitmap = figureCache.getOrPut(kind) {
    val figure = FIGURES_BY_KIND[kind] ?: FIGURES_BY_KIND.getValue("repo")
    val bitmap = Bitmap.createBitmap(512, 512, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bitmap)
    canvas.drawColor(Color.BLACK)
    val paths = figure.paths.map(::figurePath)
    val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeCap = Paint.Cap.ROUND; strokeJoin = Paint.Join.ROUND }
    for ((width, alpha) in listOf(22f to 0.1f, 10f to 0.26f, 3.4f to 0.95f)) {
        paint.strokeWidth = width
        paint.color = argb(alpha, 255, 255, 255)
        paths.forEach { canvas.drawPath(it, paint) }
    }
    val dot = Paint(Paint.ANTI_ALIAS_FLAG)
    for ((x, y) in figure.dots) {
        val cx = 256f + x * 150f
        val cy = 256f - y * 150f
        dot.shader = RadialGradient(cx, cy, 22f, intArrayOf(argb(1f, 255, 255, 255), argb(0.55f, 255, 255, 255), argb(0f, 255, 255, 255)), floatArrayOf(0f, 0.25f, 1f), Shader.TileMode.CLAMP)
        canvas.drawCircle(cx, cy, 22f, dot)
    }
    bitmap
}

// One figure path in the 512 square.
private fun figurePath(source: String): Path {
    val tokens = source.trim().split(Regex("\\s+"))
    val path = Path()
    var k = 0
    fun x(i: Int) = 256f + tokens[i].toFloat() * 150f
    fun y(i: Int) = 256f - tokens[i].toFloat() * 150f
    fun r(i: Int) = tokens[i].toFloat() * 150f
    while (k < tokens.size) {
        when (tokens[k]) {
            "M" -> { path.moveTo(x(k + 1), y(k + 2)); k += 3 }
            "L" -> { path.lineTo(x(k + 1), y(k + 2)); k += 3 }
            "Q" -> { path.quadTo(x(k + 1), y(k + 2), x(k + 3), y(k + 4)); k += 5 }
            "C" -> { path.cubicTo(x(k + 1), y(k + 2), x(k + 3), y(k + 4), x(k + 5), y(k + 6)); k += 7 }
            "O" -> { path.addCircle(x(k + 1), y(k + 2), r(k + 3), Path.Direction.CW); k += 4 }
            "E" -> {
                val cx = x(k + 1)
                val cy = y(k + 2)
                val ellipse = Path().apply { addOval(RectF(cx - r(k + 3), cy - r(k + 4), cx + r(k + 3), cy + r(k + 4)), Path.Direction.CW) }
                ellipse.transform(android.graphics.Matrix().apply { setRotate(Math.toDegrees(0.32).toFloat(), cx, cy) })
                path.addPath(ellipse)
                k += 5
            }
            "Z" -> { path.close(); k += 1 }
            else -> k += 1
        }
    }
    return path
}

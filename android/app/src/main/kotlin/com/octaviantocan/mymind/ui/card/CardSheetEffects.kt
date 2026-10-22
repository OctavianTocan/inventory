package com.octaviantocan.mymind.ui.card

import android.provider.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.layer.CompositingStrategy
import androidx.compose.ui.graphics.layer.GraphicsLayer
import androidx.compose.ui.graphics.layer.drawLayer
import androidx.compose.ui.graphics.rememberGraphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import kotlin.math.ceil

/** Follow the native reduced-motion setting for the sheet's position and reveal. */
@Composable
internal fun rememberSheetMotionEnabled(): Boolean {
    val resolver = LocalContext.current.contentResolver
    return remember(resolver) { Settings.Global.getFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) != 0f }
}

/** Pully's progressively blurred fade under a stationary sheet header. */
@Composable
fun Modifier.sheetEdgeFade(
    top: Dp = 20.dp,
    bottom: Dp = 0.dp,
    color: Color,
    blur: Dp = 3.dp,
    topStrength: () -> Float = { 1f },
    bottomStrength: () -> Float = { 1f },
    topInset: () -> Float = { 0f },
): Modifier {
    if (top <= 0.dp && bottom <= 0.dp) return this
    val source = rememberGraphicsLayer()
    val topLayers = List(FADE_STEPS) { rememberGraphicsLayer() }
    val bottomLayers = List(FADE_STEPS) { rememberGraphicsLayer() }
    return this.drawWithContent {
        source.record { this@drawWithContent.drawContent() }
        drawLayer(source)
        if (top > 0.dp) fadeBand(source, topLayers, color, topInset(), top.toPx(), true, blur.toPx(), topStrength())
        if (bottom > 0.dp) fadeBand(source, bottomLayers, color, size.height - bottom.toPx(), bottom.toPx(), false, blur.toPx(), bottomStrength())
    }
}

/** How many blur layers make a band: radius a third, two thirds and all of the blur, reaching all, two thirds and a third of the way in. */
private const val FADE_STEPS = 3

/** How opaque the page colour is at each tenth of the way out from the inside of a band to its edge: an ease, so the rows go quietly. */
private val FADE_CURVE = floatArrayOf(0f, 0.03f, 0.12f, 0.26f, 0.44f, 0.62f, 0.78f, 0.9f, 0.97f, 1f)

private fun DrawScope.fadeBand(source: GraphicsLayer, layers: List<GraphicsLayer>, color: Color, y: Float, height: Float, atTop: Boolean, blur: Float, strength: Float) {
    if (strength <= 0f || height < 1f || size.width < 1f) return
    val amount = strength.coerceIn(0f, 1f)
    if (true && blur > 0f) {
        layers.forEachIndexed { i, layer ->
            val radius = blur * (i + 1) / layers.size
            val reach = height * (layers.size - i) / layers.size
            val pad = ceil(radius * 3f)
            val edge = if (atTop) pad else pad + height
            layer.renderEffect = BlurEffect(radius, radius, TileMode.Decal)
            layer.compositingStrategy = CompositingStrategy.Offscreen
            layer.alpha = amount
            layer.record(size = IntSize(size.width.toInt(), (height + 2f * pad).toInt())) {
                translate(top = pad - y) { drawLayer(source) }
                // raw-value-ok: an alpha mask, never drawn as a colour
                val keep = Brush.verticalGradient(0f to Color.Black, 1f to Color.Transparent, startY = edge, endY = if (atTop) edge + reach else edge - reach)
                drawRect(keep, blendMode = BlendMode.DstIn)
            }
            clipRect(0f, y, size.width, y + height) { translate(top = y - pad) { drawLayer(layer) } }
        }
    }
    val stops = Array(FADE_CURVE.size) { n ->
        val outward = n / (FADE_CURVE.size - 1f)
        // Index 0 is the inside of the band and the last is the page edge; a top band runs the other way round.
        val at = if (atTop) 1f - outward else outward
        at to color.copy(alpha = FADE_CURVE[n])
    }.sortedBy { it.first }.toTypedArray()
    drawRect(Brush.verticalGradient(*stops, startY = y, endY = y + height), topLeft = Offset(0f, y), size = Size(size.width, height), alpha = amount)
}


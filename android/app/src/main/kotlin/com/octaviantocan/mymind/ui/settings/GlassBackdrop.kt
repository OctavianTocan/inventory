package com.octaviantocan.mymind.ui.settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.layer.GraphicsLayer
import androidx.compose.ui.graphics.layer.drawLayer
import androidx.compose.ui.graphics.rememberGraphicsLayer
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInWindow
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp

/**
 * The picture that glass surfaces blur, recorded once by [glassSource] and read by every [glass] surface.
 */
@Stable
class GlassBackdrop internal constructor(internal val layer: GraphicsLayer, private val scale: State<Float>) {
    internal var origin by mutableStateOf(Offset.Zero)

    internal fun zoom(): Float = scale.value
}

/**
 * @param scale Zoom applied to the recorded picture; surfaces redraw whenever it changes.
 * @return A backdrop that lives as long as the calling composition.
 */
@Composable
fun rememberGlassBackdrop(scale: State<Float>): GlassBackdrop {
    val layer = rememberGraphicsLayer()
    return remember(layer, scale) { GlassBackdrop(layer, scale) }
}

/**
 * Records the content of this node as the picture behind glass surfaces, zoomed around its centre.
 *
 * @param backdrop Backdrop that receives the content.
 */
fun Modifier.glassSource(backdrop: GlassBackdrop): Modifier = this
    .onGloballyPositioned { backdrop.origin = it.positionInWindow() }
    .drawWithContent {
        backdrop.layer.record {
            scale(backdrop.zoom(), pivot = center) { this@drawWithContent.drawContent() }
        }
        drawLayer(backdrop.layer)
    }

/**
 * Draws a frosted surface: the blurred backdrop under a tint, clipped to [shape].
 *
 * @param backdrop Backdrop to blur.
 * @param shape Outline of the surface.
 * @param background Colour filled under the blurred picture.
 * @param tint Colour laid over the blurred picture.
 * @param blurRadius Blur radius.
 */
@Composable
fun Modifier.glass(backdrop: GlassBackdrop, shape: Shape, background: Color, tint: Color, blurRadius: Dp): Modifier {
    val layer = rememberGraphicsLayer()
    var origin by remember { mutableStateOf(Offset.Zero) }
    val radius = with(LocalDensity.current) { blurRadius.toPx() }
    return this
        .clip(shape)
        .onGloballyPositioned { origin = it.positionInWindow() }
        .drawBehind {
            drawRect(background)
            // Reading the zoom makes the row redraw while the backdrop zoom animates.
            backdrop.zoom()
            layer.renderEffect = BlurEffect(radius, radius, TileMode.Clamp)
            layer.record {
                val shift = backdrop.origin - origin
                translate(shift.x, shift.y) { drawLayer(backdrop.layer) }
            }
            drawLayer(layer)
            drawRect(tint)
        }
}

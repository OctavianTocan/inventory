package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.settings.GlassBackdrop
import com.octaviantocan.mymind.ui.settings.glass

/**
 * A non-interactive sample of the scrolling viewport, blurred into the top edge of the page once content passes under it.
 *
 * @param visible Pixels the page has scrolled; the edge fades in over its own height so the title at rest stays sharp.
 */
@Composable
internal fun GalleryScrollEdge(backdrop: GlassBackdrop, paper: Color, modifier: Modifier = Modifier, visible: () -> Float = { Float.MAX_VALUE }) {
    Box(
        modifier.fillMaxWidth().height(32.dp)
            .graphicsLayer {
                compositingStrategy = CompositingStrategy.Offscreen
                alpha = (visible() / 32.dp.toPx()).coerceIn(0f, 1f)
            }
            .drawWithCache {
                val fade = Brush.verticalGradient(listOf(Color.Black, Color.Transparent))
                onDrawWithContent {
                    drawContent()
                    drawRect(fade, blendMode = BlendMode.DstIn)
                }
            }
            .glass(backdrop, RectangleShape, Color.Transparent, paper.copy(alpha = 0.60f), blurRadius = 8.dp),
    )
}

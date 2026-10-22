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

/** A non-interactive sample of the scrolling viewport, blurred into the fixed header's lower edge. */
@Composable
internal fun GalleryScrollEdge(backdrop: GlassBackdrop, paper: Color, modifier: Modifier = Modifier) {
    Box(
        modifier.fillMaxWidth().height(32.dp)
            .graphicsLayer { compositingStrategy = CompositingStrategy.Offscreen }
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

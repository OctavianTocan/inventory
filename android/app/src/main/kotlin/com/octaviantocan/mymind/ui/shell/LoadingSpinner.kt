package com.octaviantocan.mymind.ui.shell

import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
/**
 * The 38dp tall looping loading animation of the original app.
 *
 * @param modifier Modifier applied before the fixed height.
 * @param color Colour every layer of the animation is tinted with; white as in the original.
 */
@Composable
fun LoadingSpinner(modifier: Modifier = Modifier, color: Color = Color.White) {
    CircularProgressIndicator(modifier = modifier.size(38.dp), color = color, strokeWidth = 3.dp)
}

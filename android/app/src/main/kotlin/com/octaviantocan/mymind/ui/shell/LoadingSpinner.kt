package com.octaviantocan.mymind.ui.shell

import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.dp
import com.airbnb.lottie.LottieProperty
import com.airbnb.lottie.SimpleColorFilter
import com.airbnb.lottie.compose.LottieAnimation
import com.airbnb.lottie.compose.LottieCompositionSpec
import com.airbnb.lottie.compose.LottieConstants
import com.airbnb.lottie.compose.animateLottieCompositionAsState
import com.airbnb.lottie.compose.rememberLottieComposition
import com.airbnb.lottie.compose.rememberLottieDynamicProperties
import com.airbnb.lottie.compose.rememberLottieDynamicProperty
import com.octaviantocan.mymind.R

/**
 * The 38dp tall looping loading animation of the original app.
 *
 * @param modifier Modifier applied before the fixed height.
 * @param color Colour every layer of the animation is tinted with; white as in the original.
 */
@Composable
fun LoadingSpinner(modifier: Modifier = Modifier, color: Color = Color.White) {
    val composition by rememberLottieComposition(LottieCompositionSpec.RawRes(R.raw.load_spin))
    val progress by animateLottieCompositionAsState(composition, iterations = LottieConstants.IterateForever)
    val tint = rememberLottieDynamicProperties(
        rememberLottieDynamicProperty(LottieProperty.COLOR_FILTER, SimpleColorFilter(color.toArgb()), "**"),
    )
    LottieAnimation(composition, { progress }, modifier.height(38.dp), dynamicProperties = tint)
}

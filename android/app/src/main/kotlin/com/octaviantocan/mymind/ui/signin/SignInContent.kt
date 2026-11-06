package com.octaviantocan.mymind.ui.signin

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val TITLE = "Access\nyour inventory"
private const val GOOGLE_LABEL = "Sign in with Google"
private const val SPRING_STIFFNESS = 400f

/**
 * Title and Google button of the sign-in screen, centred over the splash backdrop.
 *
 * @param showSignIn True while the Google button is shown.
 * @param showLoading True while the loading spinner replaces the button.
 * @param onGoogle Called when the Google button is tapped.
 * @param modifier Modifier for the column.
 * @param googleLogo Logo on the Google button; the shipped picture by default.
 */
@Composable
fun SignInContent(
    showSignIn: Boolean,
    showLoading: Boolean,
    onGoogle: () -> Unit,
    modifier: Modifier = Modifier,
    googleLogo: Painter = painterResource(R.drawable.svg_google),
) {
    Column(
        modifier.padding(horizontal = 64.dp),
        verticalArrangement = Arrangement.spacedBy(40.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            TITLE,
            style = TextStyle(
                color = Color.Black,
                fontSize = 64.sp,
                fontFamily = MymindTheme.fonts.heading,
                letterSpacing = (-0.05).em,
                textAlign = TextAlign.Center,
                lineHeight = 50.sp,
            ),
        )
        SignInAction(showSignIn, showLoading, onGoogle, googleLogo)
    }
}

@Composable
private fun SignInAction(showSignIn: Boolean, showLoading: Boolean, onGoogle: () -> Unit, googleLogo: Painter) {
    Box(contentAlignment = Alignment.Center) {
        AnimatedVisibility(showSignIn, enter = fadeIn(), exit = fadeOut()) { GoogleButton(onGoogle, googleLogo) }
        AnimatedVisibility(
            showLoading,
            enter = fadeIn() + scaleIn(spring(stiffness = SPRING_STIFFNESS), initialScale = 0f, transformOrigin = TransformOrigin.Center),
            exit = fadeOut() + scaleOut(spring(stiffness = SPRING_STIFFNESS), targetScale = 0f, transformOrigin = TransformOrigin.Center),
        ) {
            CircularProgressIndicator(
                modifier = Modifier.size(64.dp),
                color = Color.Black,
                trackColor = MymindTheme.colors.accent,
                strokeCap = StrokeCap.Round,
            )
        }
    }
}

@Composable
private fun GoogleButton(onClick: () -> Unit, logo: Painter) {
    val shape = RoundedCornerShape(64.dp)
    Row(
        Modifier
            .heightIn(min = 48.dp)
            .background(Color.White, shape)
            .clip(shape)
            .clickable(onClick = onClick)
            .padding(horizontal = 30.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(logo, contentDescription = null, modifier = Modifier.size(24.dp))
        Text(GOOGLE_LABEL, style = TextStyle(color = Color.Black, fontSize = 18.sp, fontFamily = MymindTheme.fonts.body))
    }
}

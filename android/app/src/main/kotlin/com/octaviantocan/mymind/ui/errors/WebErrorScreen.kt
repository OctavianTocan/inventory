package com.octaviantocan.mymind.ui.errors

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val HEADING = "Something went wrong"
private const val BODY = "We couldn't load your content.\nPlease check your connection and try again."
private const val RETRY_LABEL = "Retry"
private const val BODY_ALPHA = 0.7f

/**
 * Full-screen notice shown when a page of content could not be loaded.
 *
 * @param onRetry Called when the user taps the retry button.
 * @param modifier Modifier for the screen.
 */
@Composable
fun WebErrorScreen(onRetry: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MymindTheme.colors
    val bodyFont = MymindTheme.fonts.body
    Box(modifier.fillMaxSize().background(colors.page), contentAlignment = Alignment.Center) {
        Column(Modifier.padding(32.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(HEADING, style = TextStyle(color = colors.muted, fontSize = 20.sp, fontFamily = bodyFont, textAlign = TextAlign.Center))
            Spacer(Modifier.height(8.dp))
            Text(
                BODY,
                style = TextStyle(color = colors.muted.copy(alpha = BODY_ALPHA), fontSize = 14.sp, fontFamily = bodyFont, textAlign = TextAlign.Center),
            )
            Spacer(Modifier.height(24.dp))
            Button(
                onClick = onRetry,
                shape = RoundedCornerShape(8.dp),
                colors = ButtonDefaults.buttonColors(containerColor = colors.accent, contentColor = Color.White),
            ) {
                Text(RETRY_LABEL, modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp), style = TextStyle(fontSize = 16.sp, fontFamily = bodyFont))
            }
        }
    }
}

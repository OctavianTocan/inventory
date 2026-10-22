package com.octaviantocan.mymind.ui.errors

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.painter.ColorPainter
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val MESSAGE = "Please reconnect to the internet\nto use the mymind app."

/**
 * The two pictures of the offline screen.
 *
 * @property background Full-screen backdrop, cropped to fill.
 * @property notice Notice drawn above the message, shrunk to fit between 96 dp side margins.
 */
@Immutable
class OfflinePictures(val background: Painter, val notice: Painter)

/**
 * @return The pictures that ship with the app resources, with the backdrop of the current appearance.
 */
@Composable
fun defaultOfflinePictures() = OfflinePictures(
    background = ColorPainter(MymindTheme.colors.page),
    notice = painterResource(R.drawable.offline_notice),
)

/**
 * Full-screen notice shown while the phone has no internet; it swallows taps.
 *
 * @param modifier Modifier for the screen.
 * @param pictures Backdrop and notice picture.
 */
@Composable
fun OfflineScreen(modifier: Modifier = Modifier, pictures: OfflinePictures = defaultOfflinePictures()) {
    Box(
        modifier.clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) {},
        contentAlignment = Alignment.TopStart,
    ) {
        Image(pictures.background, contentDescription = "no network notice background", modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        Column(
            Modifier.wrapContentWidth(Alignment.CenterHorizontally).fillMaxHeight(),
            verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Image(pictures.notice, contentDescription = "you're offline notice", modifier = Modifier.padding(horizontal = 96.dp), contentScale = ContentScale.Inside)
            Text(
                MESSAGE,
                modifier = Modifier.padding(horizontal = 48.dp),
                style = TextStyle(
                    color = Color.White,
                    fontSize = 18.sp,
                    fontFamily = MymindTheme.fonts.body,
                    letterSpacing = (-0.01).em,
                    textAlign = TextAlign.Center,
                    lineHeight = 24.sp,
                ),
            )
        }
    }
}

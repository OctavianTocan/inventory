package com.octaviantocan.mymind.ui.upload

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Label of the row that opens the system photo picker. */
const val PHOTO_LIBRARY = "Photo library"

/** Label of the row that opens the camera. */
const val TAKE_PHOTO = "Take photo"

/**
 * The popup the + button opens (S21): a dimmed screen with a small panel at the top end offering the photo library
 * and, when the phone has a camera app, the camera. A tap outside the rows closes it.
 *
 * @param onPhotoLibrary Called when "Photo library" is tapped.
 * @param onTakePhoto Called when "Take photo" is tapped; null leaves the row out.
 * @param onDismiss Called when the dimmed area is tapped.
 * @param modifier Modifier for the full-screen box.
 */
@Composable
fun AddMediaPopup(onPhotoLibrary: () -> Unit, onTakePhoto: (() -> Unit)?, onDismiss: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MymindTheme.colors
    Box(
        modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.7f))
            .clickable(interactionSource = null, indication = null, onClick = onDismiss),
    ) {
        Column(
            Modifier
                .align(Alignment.TopEnd)
                .windowInsetsPadding(WindowInsets.statusBars)
                .fillMaxWidth(0.6f)
                .padding(16.dp)
                .background(colors.page, RoundedCornerShape(8.dp)),
        ) {
            SourceRow(PHOTO_LIBRARY, R.drawable.baseline_photo_library_24, onPhotoLibrary)
            if (onTakePhoto != null) {
                HorizontalDivider(color = colors.raised)
                SourceRow(TAKE_PHOTO, R.drawable.baseline_camera_alt_24, onTakePhoto)
            }
        }
    }
}

/** One row of the popup: the label, then the icon at the end. */
@Composable
private fun SourceRow(label: String, @DrawableRes icon: Int, onClick: () -> Unit) {
    val ink = MymindTheme.colors.muted
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(
            label,
            Modifier.weight(1f),
            style = TextStyle(color = ink, fontSize = 16.sp, fontWeight = FontWeight.Medium, fontFamily = MymindTheme.fonts.nunito),
        )
        Icon(painterResource(icon), contentDescription = null, tint = ink)
    }
}

package com.octaviantocan.mymind.ui.upload

import androidx.annotation.DrawableRes
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.library.QueuedUpload
import com.octaviantocan.mymind.ui.library.UploadQueueState
import com.octaviantocan.mymind.ui.theme.MymindTheme
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/** Distance from the bottom of the screen to the strip: the tab bar's 80 dp and 40 dp more. */
val UPLOAD_STRIP_BOTTOM = 120.dp

private val STRIP_SHAPE = RoundedCornerShape(8.dp)
private val ITEM_SHAPE = RoundedCornerShape(4.dp)
private val STRIP_SHADOW = Shadow(radius = 24.dp, color = Color.Black, offset = DpOffset(0.dp, 8.dp), alpha = 0.2f)

/**
 * The upload queue strip over the bottom of the page: a count of the queued files with a turning upload icon, and
 * after a tap the list of files with each one's state. It fades in while the queue holds files and out when it
 * empties.
 *
 * @param state The queue.
 * @param onToggle Called when the strip is tapped, to open or close the list.
 * @param onDismiss Called with the id of a failed file whose delete button was tapped.
 * @param modifier Modifier that places the strip.
 * @param hidden True fades the strip out whatever the queue holds, as on the Serendipity tab.
 * @param bottom Distance from the strip to the bottom of its parent.
 */
@Composable
fun UploadQueueStrip(
    state: UploadQueueState,
    onToggle: () -> Unit,
    onDismiss: (Long) -> Unit,
    modifier: Modifier = Modifier,
    hidden: Boolean = false,
    bottom: Dp = UPLOAD_STRIP_BOTTOM,
) {
    AnimatedVisibility(state.items.isNotEmpty() && !hidden, modifier, enter = fadeIn(), exit = fadeOut()) {
        val colors = MymindTheme.colors
        val sidePadding by animateDpAsState(if (state.details) 24.dp else 16.dp, label = "strip side")
        val topPadding by animateDpAsState(if (state.details) 20.dp else 0.dp, label = "strip top")
        Column(
            Modifier
                .fillMaxWidth()
                .padding(start = 8.dp, end = 8.dp, bottom = bottom)
                .dropShadow(STRIP_SHAPE, STRIP_SHADOW)
                .clip(STRIP_SHAPE)
                .background(colors.raised)
                .clickable(onClick = onToggle),
        ) {
            Row(
                Modifier.fillMaxWidth().heightIn(min = 48.dp).padding(horizontal = sidePadding, vertical = topPadding),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TurningUploadIcon(18)
                Text(
                    queueCountText(state.items.size),
                    style = queueTextStyle(colors.muted),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            if (state.details) {
                state.items.forEach { item ->
                    QueueRow(item, onDismiss = { onDismiss(item.id) })
                    Spacer(Modifier.height(6.dp))
                }
                Spacer(Modifier.height(10.dp))
            }
        }
    }
}

/**
 * The strip's heading.
 *
 * @param count Files in the queue.
 * @return "All items uploaded" for none, otherwise the count of queued items.
 */
fun queueCountText(count: Int): String = when (count) {
    0 -> "All items uploaded"
    1 -> "1 item in your upload queue"
    else -> "$count items in your upload queue"
}

/**
 * The text of one queue row while it waits or runs.
 *
 * @param mime The file's media type.
 * @param added When the file was added, in milliseconds since the epoch.
 * @return Such as "Image saved on 2026.10.17".
 */
fun queueItemText(mime: String, added: Long): String =
    "${kindLabel(mime)} saved on ${SimpleDateFormat("yyyy.M.d", Locale.US).format(Date(added))}"

/** One file of the list: its kind's icon, the date or the error, and a turning icon or a delete button. */
@Composable
private fun QueueRow(item: QueuedUpload, onDismiss: () -> Unit) {
    val colors = MymindTheme.colors
    val failed = item.error != null
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .padding(horizontal = 12.dp)
            .background(colors.queueItem, ITEM_SHAPE)
            .padding(start = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(
            painterResource(kindIcon(item.mime)),
            contentDescription = null,
            modifier = Modifier.size(20.dp),
            colorFilter = if (failed) ColorFilter.tint(colors.danger) else null,
        )
        Text(
            item.error ?: queueItemText(item.mime, item.added),
            Modifier.weight(1f),
            style = queueTextStyle(if (failed) colors.danger else colors.muted),
        )
        when {
            failed -> Box(
                Modifier
                    .padding(end = 6.dp)
                    .size(32.dp)
                    .clip(ITEM_SHAPE)
                    .background(colors.danger)
                    .semantics { contentDescription = "Remove ${item.name}" }
                    .clickable(role = Role.Button, onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) { Image(painterResource(R.drawable.ic_delete), contentDescription = null) }
            item.running -> Box(Modifier.padding(end = 12.dp)) { TurningUploadIcon(14) }
        }
    }
}

/** The upload icon turning once a second. */
@Composable
private fun TurningUploadIcon(size: Int) {
    val turn by rememberInfiniteTransition(label = "upload").animateFloat(
        0f,
        360f,
        infiniteRepeatable(tween(1000, easing = LinearEasing), RepeatMode.Restart),
        label = "upload turn",
    )
    Image(painterResource(R.drawable.ic_upload_progress), contentDescription = null, modifier = Modifier.size(size.dp).rotate(turn))
}

/** Nunito 16 sp on a 20 sp line, as the strip writes everything. */
@Composable
private fun queueTextStyle(color: Color) = TextStyle(
    color = color,
    fontSize = 16.sp,
    lineHeight = 20.sp,
    letterSpacing = (-0.01).em,
    fontFamily = MymindTheme.fonts.nunito,
)

/** Label of a file's kind. */
private fun kindLabel(mime: String) = when {
    mime.startsWith("image/") -> "Image"
    mime.startsWith("video/") -> "Video"
    "pdf" in mime -> "PDF"
    else -> "Text"
}

/** Icon of a file's kind. */
@DrawableRes
private fun kindIcon(mime: String) = when {
    mime.startsWith("image/") -> R.drawable.ic_image
    mime.startsWith("video/") -> R.drawable.ic_video
    "pdf" in mime -> R.drawable.ic_pdf
    else -> R.drawable.ic_text
}

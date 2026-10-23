package com.octaviantocan.mymind.ui.card

import androidx.annotation.OptIn
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.media3.common.util.UnstableApi
import androidx.media3.ui.compose.ContentFrame
import androidx.media3.ui.compose.SURFACE_TYPE_TEXTURE_VIEW
import androidx.media3.ui.compose.state.rememberMuteButtonState
import androidx.media3.ui.compose.state.rememberPlayPauseButtonState
import androidx.media3.ui.compose.state.rememberProgressStateWithTickInterval
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.library.VideoPreviews
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.videoLength
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val TICK_MS = 250L

/**
 * A video the library server keeps, played in the reader as Google Photos plays one: it starts by itself with sound
 * over its still, a tap on the picture pauses or resumes it, and below it sit play or pause, the time played of the
 * whole, sound on or off, and a bar to move through it. It pauses while the app is in the background.
 *
 * @param card The video's card, for its still and proportions.
 * @param url The video to play.
 * @param previews The app's players, which make this one with the library's session.
 * @param modifier Modifier for the player column.
 */
@OptIn(UnstableApi::class)
@Composable
internal fun CardVideo(card: Card, url: String, previews: VideoPreviews, modifier: Modifier = Modifier) {
    val colors = LocalCardSheetColors.current
    val player = remember(url, previews) { previews.player(url) }
    DisposableEffect(player) { onDispose { player.release() } }
    LifecycleResumeEffect(player) { onPauseOrDispose { player.pause() } }
    val playPause = rememberPlayPauseButtonState(player)
    val mute = rememberMuteButtonState(player)
    val progress = rememberProgressStateWithTickInterval(player, TICK_MS)
    var dragging by remember { mutableStateOf<Float?>(null) }
    val ratio = videoRatio(card)
    val still = remember(card) { galleryPicture(card)?.let { resolveMediaUrl(BuildConfig.SERVER_URL, it) } }
    val time = TextStyle(color = colors.text, fontSize = 15.sp, lineHeight = 20.sp, fontFamily = MymindTheme.fonts.body, textAlign = TextAlign.Center)
    Column(modifier.fillMaxWidth()) {
        Box(
            Modifier.fillMaxWidth().aspectRatio(ratio).clip(RoundedCornerShape(8.dp)).background(Color.Black)
                .clickable(onClickLabel = if (playPause.showPlay) "Play" else "Pause", role = Role.Button, onClick = playPause::onClick),
        ) {
            ContentFrame(player, Modifier.fillMaxSize(), SURFACE_TYPE_TEXTURE_VIEW, ContentScale.Fit, false) {
                if (still != null) AsyncImage(still, null, Modifier.fillMaxSize(), contentScale = ContentScale.Fit)
            }
        }
        Spacer(Modifier.height(8.dp))
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            VideoControl(if (playPause.showPlay) R.drawable.ic_play_fill else R.drawable.ic_pause_fill,
                if (playPause.showPlay) "Play" else "Pause", playPause.isEnabled, playPause::onClick)
            val duration = progress.durationMs.takeIf { it > 0 }
            val position = dragging?.let { at -> duration?.let { (it * at).toLong() } } ?: progress.currentPositionMs
            Text(listOfNotNull(videoLength(position), duration?.let(::videoLength)).joinToString(" / "), Modifier.weight(1f), style = time)
            VideoControl(if (mute.showMuted) R.drawable.ic_speaker_slash else R.drawable.ic_speaker_high,
                if (mute.showMuted) "Sound on" else "Sound off", mute.isEnabled, mute::onClick)
        }
        val duration = progress.durationMs
        Slider(
            value = dragging ?: if (duration > 0) (progress.currentPositionMs.toFloat() / duration).coerceIn(0f, 1f) else 0f,
            onValueChange = { dragging = it },
            onValueChangeFinished = {
                dragging?.let { at -> if (duration > 0) player.seekTo((duration * at).toLong()) }
                dragging = null
            },
            enabled = duration > 0,
            modifier = Modifier.fillMaxWidth().semantics { contentDescription = "Video position" },
            colors = SliderDefaults.colors(
                thumbColor = colors.text, activeTrackColor = colors.text, inactiveTrackColor = colors.text.copy(alpha = 0.22f),
                disabledThumbColor = colors.text.copy(alpha = 0.3f), disabledInactiveTrackColor = colors.text.copy(alpha = 0.12f),
            ),
        )
    }
}

/** A round 44 dp control under the video. */
@Composable
private fun VideoControl(icon: Int, label: String, enabled: Boolean, onClick: () -> Unit) {
    val colors = LocalCardSheetColors.current
    Box(
        Modifier.size(44.dp).clip(CircleShape).semantics { contentDescription = label }
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Image(painterResource(icon), null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(colors.text.copy(alpha = if (enabled) 1f else 0.4f)))
    }
}

/** Width over height of the card's video, kept between a tall phone video and a wide film. */
private fun videoRatio(card: Card): Float {
    val w = card.blob?.width ?: card.cover?.width
    val h = card.blob?.height ?: card.cover?.height
    return if (w != null && h != null && w > 0 && h > 0) (w.toFloat() / h).coerceIn(0.5f, 2.4f) else 16f / 9f
}

package com.octaviantocan.mymind.ui.card

import androidx.annotation.OptIn
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.graphics.ColorFilter
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
import androidx.media3.ui.compose.state.rememberPlayPauseButtonState
import androidx.media3.ui.compose.state.rememberProgressStateWithTickInterval
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.library.VideoPreviews
import com.octaviantocan.mymind.ui.library.videoLength
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val TICK_MS = 250L

/**
 * @param card A card.
 * @return The address of the audio file the library server keeps for [card], or null when it has none.
 */
internal fun readerAudio(card: Card): String? = card.blob?.takeIf { it.type?.startsWith("audio/") == true }?.path
    ?.takeIf { it.isNotBlank() && !it.startsWith("http") }
    ?.let { resolveMediaUrl(BuildConfig.SERVER_URL, "/media/${it.removePrefix("/media/").trimStart('/')}") }

/**
 * A compact player for an audio file the library server keeps, shown above the title in the reader: a round play or
 * pause button, a bar to drag through the file, and the time played of the whole. It waits to be played, pauses while
 * the app is in the background and releases its player when it leaves the screen.
 *
 * @param url The audio file to play.
 * @param knownSeconds The file's length as the server read it, shown until the player knows it; null when unknown.
 * @param previews The app's players, which make this one with the library's session.
 * @param modifier Modifier for the player row.
 */
@OptIn(UnstableApi::class)
@Composable
internal fun CardAudio(url: String, knownSeconds: Double?, previews: VideoPreviews, modifier: Modifier = Modifier) {
    val colors = LocalCardSheetColors.current
    val player = remember(url, previews) { previews.player(url).also { it.playWhenReady = false } }
    DisposableEffect(player) { onDispose { player.release() } }
    LifecycleResumeEffect(player) { onPauseOrDispose { player.pause() } }
    val playPause = rememberPlayPauseButtonState(player)
    val progress = rememberProgressStateWithTickInterval(player, TICK_MS)
    var dragging by remember { mutableStateOf<Float?>(null) }
    val duration = progress.durationMs.takeIf { it > 0 } ?: knownSeconds?.takeIf { it.isFinite() && it > 0.0 }?.let { (it * 1000).toLong() } ?: 0L
    val position = dragging?.let { (duration * it).toLong() } ?: progress.currentPositionMs.coerceAtLeast(0L)
    val label = if (playPause.showPlay) "Play" else "Pause"
    val time = TextStyle(color = colors.text, fontSize = 15.sp, lineHeight = 20.sp, fontFamily = MymindTheme.fonts.body, textAlign = TextAlign.End)
    Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Box(
            Modifier.size(44.dp).clip(CircleShape).background(colors.text).semantics { contentDescription = label }
                .clickable(enabled = playPause.isEnabled, role = Role.Button, onClick = playPause::onClick),
            contentAlignment = Alignment.Center,
        ) {
            Image(painterResource(if (playPause.showPlay) R.drawable.ic_play_fill else R.drawable.ic_pause_fill), null, Modifier.size(22.dp),
                colorFilter = ColorFilter.tint(colors.raised))
        }
        Slider(
            value = dragging ?: if (duration > 0) (position.toFloat() / duration).coerceIn(0f, 1f) else 0f,
            onValueChange = { dragging = it },
            onValueChangeFinished = {
                dragging?.let { at -> if (duration > 0) player.seekTo((duration * at).toLong()) }
                dragging = null
            },
            enabled = duration > 0,
            modifier = Modifier.weight(1f).semantics { contentDescription = "Audio position" },
            colors = SliderDefaults.colors(
                thumbColor = colors.text, activeTrackColor = colors.text, inactiveTrackColor = colors.text.copy(alpha = 0.22f),
                disabledThumbColor = colors.text.copy(alpha = 0.3f), disabledInactiveTrackColor = colors.text.copy(alpha = 0.12f),
            ),
        )
        Text(listOfNotNull(videoLength(position), duration.takeIf { it > 0 }?.let(::videoLength)).joinToString(" / "), style = time)
    }
}

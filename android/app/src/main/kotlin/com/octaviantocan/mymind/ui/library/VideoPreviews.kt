package com.octaviantocan.mymind.ui.library

import android.content.Context
import androidx.annotation.OptIn
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.okhttp.OkHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.compose.ContentFrame
import androidx.media3.ui.compose.SURFACE_TYPE_TEXTURE_VIEW
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import okhttp3.OkHttpClient

private const val MAX_PLAYERS = 3
private const val POSTER = ";800x1000-t.webp"
private const val CLIP = "480x600.mp4"
private val RENDITION = Regex(""";(\d+)x(\d+)(-[tc])?(\.mp4|/hevc\.mp4)$""")

/**
 * Muted, looping previews for the gallery's video tiles, as Google Photos plays them: at most three play at once, and
 * each video's length is remembered once a player has read it.
 *
 * @param context Context the players run in.
 * @param client The library's HTTP client; the media lives on the library server and needs its session.
 */
@OptIn(UnstableApi::class)
class VideoPreviews(private val context: Context, client: OkHttpClient) {
    private val sources = DefaultMediaSourceFactory(OkHttpDataSource.Factory(client))
    private val idle = ArrayDeque<ExoPlayer>()
    private var playing = 0
    private val lengths = HashMap<String, Long>()
    private var closed = false

    /**
     * @param url The video to play.
     * @return A muted player already preparing [url], or null when three previews are playing.
     */
    fun acquire(url: String): ExoPlayer? {
        if (closed || playing >= MAX_PLAYERS) return null
        playing++
        val player = idle.removeFirstOrNull() ?: ExoPlayer.Builder(context).setMediaSourceFactory(sources).build().apply {
            volume = 0f
            repeatMode = Player.REPEAT_MODE_ONE
        }
        player.setMediaItem(MediaItem.fromUri(url))
        player.prepare()
        player.playWhenReady = true
        return player
    }

    /** Stops [player] and keeps it for the next tile, or frees it once the previews are closed. */
    fun release(player: ExoPlayer) {
        playing--
        if (closed) return player.release()
        player.stop()
        player.clearMediaItems()
        idle.addLast(player)
    }

    /** Records the length of the video at [url] in milliseconds. */
    fun rememberLength(url: String, millis: Long) {
        lengths[url] = millis
    }

    /** @return The remembered length of the video at [url] in milliseconds, or null. */
    fun length(url: String): Long? = lengths[url]

    /** Releases every player; the previews cannot be used afterwards. */
    fun close() {
        closed = true
        idle.forEach(ExoPlayer::release)
        idle.clear()
    }
}

/** The previews of the signed-in app; null in screenshots and tests, which then draw still pictures. */
val LocalVideoPreviews = staticCompositionLocalOf<VideoPreviews?> { null }

/**
 * @param card A card.
 * @return The server video a preview plays: a moving cover as it is, else a small silent copy of the video file; null
 *   for videos on other sites such as YouTube.
 */
internal fun previewVideo(card: Card): String? {
    val cover = card.cover?.takeIf { it.type == "video" }?.src
    val file = card.blob?.takeIf { it.type?.startsWith("video/") == true }?.path?.takeIf { !it.startsWith("http") }
        ?.let { "/media/${it.removePrefix("/media/").trimStart('/')};$CLIP" }
    val src = (cover ?: file)?.takeIf { it.isNotBlank() && !it.startsWith("http") } ?: return null
    return resolveMediaUrl(BuildConfig.SERVER_URL, src)
}

/**
 * @param src A moving cover's path on the library server.
 * @return A still frame of it that an image loader can draw, or null when the server cannot make one.
 */
internal fun videoPoster(src: String): String? = when {
    RENDITION.containsMatchIn(src) -> src.replace(RENDITION) { ";${it.groupValues[1]}x${it.groupValues[2]}${it.groupValues[3]}.webp" }
    src.startsWith("http") || ';' in src -> null
    else -> src + POSTER
}

/**
 * A muted preview of [url] over the tile, with [still] beneath until its first frame shows. Nothing plays while the app
 * is in the background or when [previews] has no free player.
 *
 * @param url The video.
 * @param previews The shared players.
 * @param onLength Called with the video's length in milliseconds once it is known.
 * @param still The picture shown before the video's first frame.
 */
@OptIn(UnstableApi::class)
@Composable
internal fun VideoPreview(url: String, previews: VideoPreviews, onLength: (Long) -> Unit, still: @Composable () -> Unit) {
    var player by remember(url) { mutableStateOf<ExoPlayer?>(null) }
    LifecycleResumeEffect(url, previews) {
        val acquired = previews.acquire(url)
        player = acquired
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(playbackState: Int) {
                val length = acquired?.duration ?: C.TIME_UNSET
                if (playbackState == Player.STATE_READY && length != C.TIME_UNSET) {
                    previews.rememberLength(url, length)
                    onLength(length)
                }
            }
        }
        acquired?.addListener(listener)
        onPauseOrDispose {
            acquired?.removeListener(listener)
            acquired?.let(previews::release)
            player = null
        }
    }
    val current = player
    if (current == null) still() else ContentFrame(current, Modifier.fillMaxSize(), SURFACE_TYPE_TEXTURE_VIEW, ContentScale.Crop, false) { still() }
}

/**
 * The corner badge of a video tile: the YouTube mark for YouTube, else a play mark, then the length when known.
 *
 * @param youtube True for a YouTube video.
 * @param millis The video's length in milliseconds, or null.
 * @param modifier Modifier for the badge; callers align it to a corner.
 */
@Composable
internal fun VideoBadge(youtube: Boolean, millis: Long?, modifier: Modifier = Modifier) {
    Row(
        modifier.clip(RoundedCornerShape(50)).background(Color.Black.copy(alpha = 0.55f)).padding(horizontal = 7.dp, vertical = 3.dp)
            .clearAndSetSemantics {
                contentDescription = listOfNotNull(if (youtube) "YouTube video" else "Video", millis?.let(::videoLength)).joinToString(", ")
            },
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(painterResource(if (youtube) R.drawable.ic_youtube else R.drawable.ic_play_fill), null,
            Modifier.size(if (youtube) 16.dp else 12.dp), colorFilter = ColorFilter.tint(Color.White))
        if (millis != null) Text(videoLength(millis), color = Color.White, style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp))
    }
}

/**
 * @param millis A length in milliseconds.
 * @return It as `m:ss`, or `h:mm:ss` from an hour.
 */
internal fun videoLength(millis: Long): String {
    val seconds = (millis.coerceAtLeast(0L) + 500L) / 1000L
    val (hours, minutes, rest) = Triple(seconds / 3600, seconds / 60 % 60, seconds % 60)
    return if (hours > 0) "%d:%02d:%02d".format(hours, minutes, rest) else "%d:%02d".format(minutes, rest)
}

/** Releases [previews] when the composition that made it leaves. */
@Composable
internal fun ClosePreviewsOnDispose(previews: VideoPreviews) {
    DisposableEffect(previews) { onDispose { previews.close() } }
}

package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.shell.FootCircleButton
import com.octaviantocan.mymind.ui.shell.FootPressTheme
import com.octaviantocan.mymind.ui.shell.footShadow
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import okhttp3.OkHttpClient

/** Width the fixed controls cover at the end of the title row. */
internal val CONTROLS_WIDTH = 44.dp * 3 + 8.dp * 2 + 16.dp

/**
 * A page title that scrolls away with the page beneath the fixed [GalleryControls], in the same row geometry.
 *
 * @param text The title.
 * @param modifier Modifier for the row.
 * @param start Inset before the title, such as room for a floating Back button.
 * @param end Room kept clear for the fixed controls at the end of the row.
 */
@Composable
fun GalleryTitle(text: String, modifier: Modifier = Modifier, start: Dp = 20.dp, end: Dp = CONTROLS_WIDTH) {
    Box(modifier.fillMaxWidth().padding(start = start, top = 12.dp, bottom = 16.dp).height(44.dp), contentAlignment = Alignment.CenterStart) {
        Text(text, Modifier.padding(end = end).semantics { heading() }, style = LibraryStyle.title,
            color = LibraryStyle.colors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/**
 * Add, Rediscover and the account, fixed over the scrolling page; their own raised circles and shadows replace a solid
 * header strip. While [LocalOffline] is true an [OfflinePill] hangs under them.
 *
 * @param onSettings Called when the account is tapped.
 * @param onAdd Called when Add is tapped.
 * @param onSerendipity Called when Rediscover is tapped.
 * @param modifier Modifier for the row; callers align it to the top end.
 * @param photoUrl The account's Google photo, or null for the profile glyph.
 * @param photoLoader Loader for [photoUrl] that never carries the library session.
 */
@Composable
fun GalleryControls(
    onSettings: () -> Unit,
    onAdd: () -> Unit,
    onSerendipity: () -> Unit,
    modifier: Modifier = Modifier,
    photoUrl: String? = null,
    photoLoader: ImageLoader? = null,
) {
    val colors = LibraryStyle.colors
    Column(modifier.padding(end = 16.dp, top = 12.dp), horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            FootCircleButton(R.drawable.ic_library_add, "Add", onAdd, size = 44.dp)
            FootCircleButton(R.drawable.ic_library_shuffle, "Rediscover cards", onSerendipity, size = 44.dp)
            FootPressTheme {
                Box(Modifier.size(44.dp).footShadow(CircleShape).clip(CircleShape)
                    .background(colors.selected).border(2.dp, colors.accent, CircleShape)
                    .semantics { contentDescription = "Account settings" }
                    .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onSettings)
                    .padding(4.dp).clip(CircleShape), contentAlignment = Alignment.Center) {
                    Image(painterResource(R.drawable.ic_library_profile), null, Modifier.size(24.dp), colorFilter = ColorFilter.tint(colors.text))
                    if (photoUrl != null && photoLoader != null) AsyncImage(model = photoUrl, imageLoader = photoLoader, contentDescription = null,
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop, modifier = Modifier.fillMaxSize())
                }
            }
        }
        if (LocalOffline.current) OfflinePill()
    }
}

/** True while the phone has no internet; the gallery then runs from the phone's copy. */
val LocalOffline = compositionLocalOf { false }

/** A small raised pill under the controls saying the app is working without internet. */
@Composable
fun OfflinePill(modifier: Modifier = Modifier) {
    val colors = LibraryStyle.colors
    val shape = RoundedCornerShape(50)
    Row(
        modifier.height(28.dp).footShadow(shape).clip(shape).background(colors.raised).padding(horizontal = 12.dp)
            .semantics { liveRegion = LiveRegionMode.Polite },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(Modifier.size(6.dp).clip(CircleShape).background(colors.muted))
        Text("Offline", style = LibraryStyle.label, color = colors.text)
    }
}

/** Google photos must never use the library's HTTP client, which attaches its session bearer. */
@Composable
internal fun rememberPublicAccountPhotoLoader(): ImageLoader {
    val context = LocalContext.current.applicationContext
    val client = remember { OkHttpClient() }
    val loader = remember(context, client) {
        ImageLoader.Builder(context).components { add(OkHttpNetworkFetcherFactory(callFactory = { client })) }.build()
    }
    DisposableEffect(loader, client) {
        onDispose {
            loader.shutdown()
            client.connectionPool.evictAll()
            client.dispatcher.executorService.shutdown()
        }
    }
    return loader
}

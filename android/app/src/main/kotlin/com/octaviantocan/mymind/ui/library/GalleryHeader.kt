package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
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

/** Account and library actions above Top of Mind, with no search field competing with the gallery. */
@Composable
fun GalleryHeader(
    onSettings: () -> Unit,
    onAdd: () -> Unit,
    onSerendipity: () -> Unit,
    modifier: Modifier = Modifier,
    photoUrl: String? = null,
    photoLoader: ImageLoader? = null,
) {
    val colors = LibraryStyle.colors
    Row(modifier.fillMaxWidth().padding(start = 20.dp, end = 16.dp, top = 12.dp, bottom = 16.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("Your mind", Modifier.weight(1f).semantics { heading() }, style = LibraryStyle.title,
            color = colors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
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

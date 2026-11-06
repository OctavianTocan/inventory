package com.octaviantocan.mymind.ui.menu

import android.content.ClipData
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import androidx.core.content.FileProvider
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.request.allowHardware
import coil3.toBitmap
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryPicture
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.ui.card.siteOf

/**
 * The Share sheet's page: what of the card can be sent, as large choices, then Copy link. A null action leaves its
 * choice out.
 *
 * @param card Card being shared.
 * @param onShareLink Sends the card's link to another app.
 * @param onShareText Sends the card's own words to another app.
 * @param onCopyLink Puts the card's link on the clipboard.
 * @param onSharePicture Sends the card's picture to another app.
 * @param onCardImage Opens the card as an image.
 */
@Composable
internal fun ShareContent(
    card: Card,
    onShareLink: (() -> Unit)?,
    onShareText: (() -> Unit)?,
    onCopyLink: (() -> Unit)?,
    onSharePicture: (() -> Unit)? = null,
    onCardImage: (() -> Unit)? = null,
) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 20.dp, bottom = 30.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        onShareLink?.let { SheetChoice(painterResource(R.drawable.ic_link), "The link", siteOf(card) ?: card.url, it) }
        onShareText?.let { SheetChoice(painterResource(R.drawable.ic_summary), "The words", "As text, to paste anywhere", it) }
        onSharePicture?.let { SheetChoice(painterResource(R.drawable.ic_image), "The picture", pictureDetail(card), it) }
        onCardImage?.let { SheetChoice(painterResource(R.drawable.ic_card_image), "The card as an image", cardImageDetail(card), it) }
        if (onCopyLink != null) {
            Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.Center) {
                SheetPill(painterResource(R.drawable.ic_link), "Copy link", onCopyLink)
            }
        }
    }
}

/** "Photo, 1600 × 1067" when the picture's size is known, else "Photo". */
private fun pictureDetail(card: Card): String {
    val width = card.cover?.width?.takeIf { it > 0 }
    val height = card.cover?.height?.takeIf { it > 0 }
    return if (width != null && height != null) "Photo, $width × $height" else "Photo"
}

/** What the card as an image starts with. */
private fun cardImageDetail(card: Card): String {
    val parts = listOfNotNull("picture".takeIf { galleryPicture(card) != null }, "title", "your note".takeIf { !card.note.isNullOrBlank() })
    val text = if (parts.size == 1) parts[0] else parts.dropLast(1).joinToString(", ") + " and " + parts.last()
    return text.replaceFirstChar { it.uppercase() }
}

/**
 * Loads the card's picture whole through the app's image loader, from its cache when the gallery has drawn it.
 *
 * @param context Any context of the app.
 * @param card Card whose picture is wanted.
 * @return The picture, or null when the card has none or it could not be loaded.
 */
internal suspend fun loadPicture(context: Context, card: Card): Bitmap? {
    val src = galleryPicture(card) ?: return null
    val request = ImageRequest.Builder(context).data(resolveMediaUrl(BuildConfig.SERVER_URL, src)).allowHardware(false).build()
    val result = SingletonImageLoader.get(context).execute(request) as? SuccessResult ?: return null
    return result.image.toBitmap()
}

/**
 * Writes a picture to the app's cache, where the app's file provider lends it to other apps.
 *
 * @param context Any context of the app.
 * @param bitmap The picture.
 * @param name File name, which other apps may show.
 * @param png True keeps every pixel (for drawn text); false writes a JPEG (for photos).
 * @return A content address other apps can read once granted.
 */
internal suspend fun cacheImage(context: Context, bitmap: Bitmap, name: String, png: Boolean): Uri = withContext(Dispatchers.IO) {
    val dir = File(context.cacheDir, SHARED_DIR).apply { mkdirs() }
    val file = File(dir, name)
    file.outputStream().use { bitmap.compress(if (png) Bitmap.CompressFormat.PNG else Bitmap.CompressFormat.JPEG, if (png) 100 else JPEG_QUALITY, it) }
    FileProvider.getUriForFile(context, "${context.packageName}.provider", file)
}

/**
 * Opens the system share sheet for a picture in the app's cache.
 *
 * @param context A context that can start activities.
 * @param uri The picture, from [cacheImage].
 * @param mime Its type, such as `image/png`.
 */
internal fun sharePicture(context: Context, uri: Uri, mime: String) {
    val intent = Intent(Intent.ACTION_SEND).setType(mime).putExtra(Intent.EXTRA_STREAM, uri).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    intent.clipData = ClipData.newRawUri(null, uri)
    Track.event("share", detail = "picture")
    runCatching { context.startActivity(Intent.createChooser(intent, null)) }
}

/**
 * @param card A card.
 * @param suffix What follows the name, such as ".png".
 * @return A file name for its shared picture: its name made safe, else its id.
 */
internal fun sharedImageName(card: Card, suffix: String): String {
    val base = cardHeading(card).replace(Regex("[^A-Za-z0-9 _-]"), "").trim().take(40).ifEmpty { card.id }
    return "$base$suffix"
}

private const val SHARED_DIR = "shared"
private const val JPEG_QUALITY = 92

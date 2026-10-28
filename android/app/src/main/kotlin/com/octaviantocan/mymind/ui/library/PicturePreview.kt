package com.octaviantocan.mymind.ui.library

import android.graphics.Bitmap
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.graphics.painter.Painter
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.thumbHashPicture

/**
 * The blurred stand-in for [card]'s gallery picture, drawn from the ThumbHash the server keeps for it, which a picture
 * shows until it loads and keeps when it cannot load, as without internet.
 *
 * @param card A card.
 * @return The stand-in, scaled up smoothly so it reads as a blur, or null when the card has no hash yet.
 */
@Composable
internal fun rememberPicturePreview(card: Card): Painter? {
    val hash = card.cover?.thumbHash?.takeIf { it.isNotBlank() } ?: card.blob?.thumbHash?.takeIf { it.isNotBlank() }
    return remember(hash) {
        hash?.let(::thumbHashPicture)?.let { BitmapPainter(Bitmap.createBitmap(it.argb, it.width, it.height, Bitmap.Config.ARGB_8888).asImageBitmap()) }
    }
}

package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.Hyphens
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.card.savedLabel
import com.octaviantocan.mymind.ui.theme.LibraryStyle

/** The most tags a page in reels lists; the card itself shows the rest. */
private const val REEL_TAGS = 6

/**
 * What a page in reels says about its card: [source], where it is from and when it was saved; its [title]; its [words];
 * and its [tags]. A null or empty part is left out.
 */
internal class ReelInfo(val source: String, val title: String?, val words: String?, val tags: List<String>)

/**
 * @param card A card.
 * @return What its page in reels says: a post's author and handle, a product's shop and price, or the card's site or kind,
 *   then when it was saved; its title; its words, or its description when the words only repeat the title; and its
 *   first tags.
 */
internal fun reelInfo(card: Card): ReelInfo {
    val from = when {
        tileKind(card) == TileKind.Post -> postByline(card)?.let { listOfNotNull(it.name, it.handle).joinToString(" · ") }
        card.type == "Product" -> listOfNotNull(productShop(card), productPrice(card)).joinToString(" · ").ifEmpty { null }
        else -> siteName(card)
    } ?: galleryKind(card)
    val source = listOfNotNull(from, savedLabel(card.created)).joinToString(" · ")
    val title = card.title?.trim()?.takeIf { it.isNotEmpty() && !sameWords(it, from) }
    // A link's tile shows its title, so its page goes on to the description.
    val words = listOf(galleryText(card), card.description.orEmpty()).map { it.trim() }
        .firstOrNull { it.isNotEmpty() && (title == null || !sameWords(title, it)) }
    val tags = card.tags.orEmpty().mapNotNull { tag -> tag.name?.trim()?.takeIf { it.isNotEmpty() } }.take(REEL_TAGS)
    return ReelInfo(source, title, words, tags)
}

/** The site a card was saved from, when the card names one. */
private fun siteName(card: Card): String? = listOf(card.siteName, card.domain).firstOrNull { !it.isNullOrBlank() }?.trim()

/**
 * A card on its own full-screen page in reels, fuller than its tile ([GalleryTile]): its picture or video whole, at its
 * own shape in the middle of the page, as its tile comes to rest there, then over a shade at the foot where it is from
 * and when it was saved, its title, its words and its tags ([reelInfo]). A card with no picture shows its words large
 * in the middle instead. A tap opens the card and a hold its actions.
 *
 * @param card The card.
 * @param onClick Called when the page is tapped.
 * @param onLongClick Called when the page is held, or null when holding does nothing.
 * @param imageRequestSizePx Size, in pixels, to decode the card's picture at.
 * @param modifier Modifier for the page, sized to it.
 */
@Composable
internal fun GalleryReelPage(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, imageRequestSizePx: Int, modifier: Modifier) {
    val info = remember(card) { reelInfo(card) }
    val picture = galleryPicture(card)
    val image = rememberGalleryImage(picture, imageRequestSizePx)
    val preview = rememberPicturePreview(card)
    val previews = LocalVideoPreviews.current
    val video = remember(card) { previewVideo(card) }
    val moving = rememberSheetMotionEnabled()
    val ratio = remember(card) { storedRatio(card) ?: mosaicAspectRatio(card) }
    Box(
        modifier.fillMaxSize()
            .semantics(mergeDescendants = true) { contentDescription = card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card) }
            .combinedClickable(interactionSource = null, indication = null, role = Role.Button,
                onLongClickLabel = "Card actions", onLongClick = onLongClick, onClick = onClick),
    ) {
        if (picture != null) BoxWithConstraints(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            // The largest box of the picture's shape that fits, as the tile is drawn when it comes to rest on its page.
            val frame = if (maxWidth / maxHeight < ratio) Modifier.size(maxWidth, maxWidth / ratio) else Modifier.size(maxHeight * ratio, maxHeight)
            Box(frame.clearAndSetSemantics {}) {
                val still = @Composable {
                    AsyncImage(model = image, contentDescription = null, placeholder = preview, error = preview,
                        contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
                }
                if (previews != null && video != null && moving) VideoPreview(video, previews, {}, still) else still()
            }
        } else {
            Text(
                info.words ?: info.title ?: galleryKind(card),
                Modifier.align(Alignment.Center).fillMaxWidth().padding(horizontal = 28.dp)
                    .windowInsetsPadding(WindowInsets.statusBars).padding(bottom = 140.dp),
                color = Color.White, maxLines = 12, overflow = TextOverflow.Ellipsis,
                style = LibraryStyle.body.copy(fontSize = 24.sp, lineHeight = 33.sp, hyphens = Hyphens.Auto, lineBreak = LineBreak.Paragraph),
            )
        }
        // Without a picture the words, or else the title, are the page's middle, so the foot leaves them out.
        ReelInfoBlock(info.source, info.title.takeIf { picture != null || info.words != null }, info.words.takeIf { picture != null }, info.tags,
            Modifier.align(Alignment.BottomStart))
    }
}

/**
 * The foot of a page in reels, white over a shade that darkens toward the bottom: the card's [source], its [title], its
 * [words] in up to four lines and its [tags] as chips, leaving out what is null or empty.
 */
@Composable
private fun ReelInfoBlock(source: String, title: String?, words: String?, tags: List<String>, modifier: Modifier) {
    val shade = Brush.verticalGradient(0f to Color.Transparent, 0.35f to Color.Black.copy(alpha = 0.55f), 1f to Color.Black.copy(alpha = 0.8f))
    Column(
        modifier.fillMaxWidth().background(shade).windowInsetsPadding(WindowInsets.navigationBars)
            .padding(start = 20.dp, end = 20.dp, top = 48.dp, bottom = 24.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Text(source, color = Color.White.copy(alpha = 0.72f), style = LibraryStyle.label, maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (title != null) Text(title, color = Color.White, maxLines = 2, overflow = TextOverflow.Ellipsis,
            style = LibraryStyle.title.copy(fontSize = 22.sp, lineHeight = 28.sp))
        if (words != null) Text(words, color = Color.White.copy(alpha = 0.88f), maxLines = 4, overflow = TextOverflow.Ellipsis,
            style = LibraryStyle.body)
        if (tags.isNotEmpty()) FlowRow(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp), maxLines = 1) {
            tags.forEach { tag ->
                Text(tag, Modifier.clip(RoundedCornerShape(50)).background(Color.White.copy(alpha = 0.16f))
                    .padding(horizontal = 12.dp, vertical = 6.dp), color = Color.White,
                    style = LibraryStyle.label.copy(fontWeight = FontWeight.Medium), maxLines = 1)
            }
        }
    }
}

package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.text.HtmlCompat
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.shell.FootPressTheme
import com.octaviantocan.mymind.ui.theme.LibraryStyle

private val VISUAL_CARD_TYPES = setOf("Image", "Screenshot", "Video", "YouTubeVideo", "InstagramReel")

/** A gallery brick keeps both the words and picture of a mixed card visible at every density. */
@Composable
internal fun GalleryTile(
    card: Card,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onLongClick: (() -> Unit)? = null,
    enabled: Boolean = true,
    highlight: Boolean = false,
) {
    val colors = LibraryStyle.colors
    val accent = cardAccent(card) ?: colors.selected
    val kind = tileKind(card)
    val picture = galleryPicture(card)
    val text = remember(card) { galleryText(card) }
    val mixed = picture != null && text.isNotBlank() && galleryKeepsText(card)
    val paper = lerp(colors.raised, accent, 0.13f)
    val background = if (picture == null || mixed) paper else parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor) ?: paper
    val heading = card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card)
    FootPressTheme {
        Box(
            modifier.fillMaxSize().clip(RoundedCornerShape(if (highlight) 28.dp else 3.dp)).background(background)
                .semantics(mergeDescendants = true) { contentDescription = heading }
                .combinedClickable(
                    interactionSource = null, indication = CardSheetIndication, enabled = enabled, role = Role.Button,
                    onLongClickLabel = "Card actions", onLongClick = onLongClick, onClick = onClick,
                ),
        ) {
            if (mixed) {
                Column(Modifier.fillMaxSize()) {
                    GalleryTextPreview(card, text, highlight, Modifier.weight(0.56f).fillMaxWidth())
                    AsyncImage(
                        model = resolveMediaUrl(BuildConfig.SERVER_URL, picture), contentDescription = "Image preview",
                        contentScale = ContentScale.Crop, modifier = Modifier.weight(0.44f).fillMaxWidth(),
                    )
                }
                Box(Modifier.fillMaxWidth().height(3.dp).background(accent))
            } else if (picture != null) {
                AsyncImage(
                    model = resolveMediaUrl(BuildConfig.SERVER_URL, picture), contentDescription = null,
                    contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize(),
                )
                if (highlight || kind !in setOf(TileKind.Media, TileKind.Video)) {
                    Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.35f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.78f))))
                    Text(
                        card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card),
                        Modifier.align(Alignment.BottomStart).padding(if (highlight) 18.dp else 12.dp).clearAndSetSemantics {},
                        color = Color.White, style = if (highlight) LibraryStyle.title else LibraryStyle.body,
                        maxLines = if (highlight) 3 else 2, overflow = TextOverflow.Ellipsis,
                    )
                }
            } else {
                GalleryTextPreview(card, text, highlight, Modifier.fillMaxSize())
                Box(Modifier.fillMaxWidth().height(3.dp).background(accent))
            }
            if (picture != null && kind == TileKind.Video) {
                Icon(
                    painterResource(R.drawable.ic_video), "Video", tint = Color.White,
                    modifier = Modifier.align(if (mixed) Alignment.BottomStart else Alignment.TopEnd).padding(10.dp).height(20.dp),
                )
            }
            val count = card.post?.gallery?.size ?: 0
            if (count > 1) Text(
                "$count", Modifier.align(if (mixed) Alignment.BottomEnd else Alignment.TopEnd).padding(8.dp).clip(RoundedCornerShape(5.dp))
                    .background(Color.Black.copy(alpha = 0.55f)).padding(horizontal = 6.dp, vertical = 2.dp).clearAndSetSemantics {},
                style = LibraryStyle.label, color = Color.White,
            )
        }
    }
}

/** Text gets its own paper area; compact tiles spend that space on the excerpt before the type label. */
@Composable
private fun GalleryTextPreview(card: Card, text: String, highlight: Boolean, modifier: Modifier) {
    val colors = LibraryStyle.colors
    BoxWithConstraints(modifier) {
        val inset = if (highlight) 18.dp else if (maxWidth < 110.dp) 8.dp else 14.dp
        val textSize = if (highlight) 22.sp else if (maxWidth < 110.dp) 15.sp else 17.sp
        val lineHeight = if (highlight) 29.sp else if (maxWidth < 110.dp) 21.sp else 24.sp
        val showKind = maxWidth >= 110.dp && maxHeight >= 112.dp
        val lineDp = with(LocalDensity.current) { lineHeight.toDp() }
        val availableLines = ((maxHeight - inset * 2 - if (showKind) 31.dp else 0.dp) / lineDp).toInt().coerceIn(1, 12)
        Column(Modifier.fillMaxSize().padding(inset)) {
            if (showKind) {
                Text(galleryKind(card), Modifier.clearAndSetSemantics {}, color = colors.muted,
                    style = LibraryStyle.label, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Spacer(Modifier.height(10.dp))
            }
            Text(
                text.ifBlank { cardHeading(card) }, Modifier.weight(1f),
                color = colors.text, style = LibraryStyle.body.copy(fontSize = textSize, lineHeight = lineHeight),
                fontWeight = if (tileKind(card) in setOf(TileKind.Repository, TileKind.Article, TileKind.Plain)) FontWeight.SemiBold else FontWeight.Normal,
                maxLines = availableLines, overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

/** Only image files are sent to Coil; video cards use their cover rather than the video blob. */
internal fun galleryPicture(card: Card): String? = card.cover?.src?.takeIf { it.isNotBlank() }
    ?: card.post?.gallery?.firstOrNull()?.src?.takeIf { it.isNotBlank() }
    ?: card.blob?.takeIf { it.type?.startsWith("image/") == true || (it.type == null && card.type in setOf("Image", "Screenshot")) }
        ?.path?.takeIf { it.isNotBlank() }

/** Saved links keep their headline; visual cards also keep any note the user wrote alongside the image. */
internal fun galleryKeepsText(card: Card): Boolean = card.type !in VISUAL_CARD_TYPES
    || !card.note.isNullOrBlank() || card.prose?.let(::proseText)?.isNotBlank() == true

private fun galleryText(card: Card): String = when (tileKind(card)) {
    TileKind.Note -> noteText(card)
    TileKind.Post -> card.post?.bodyHtml?.let { HtmlCompat.fromHtml(it, HtmlCompat.FROM_HTML_MODE_COMPACT).toString().trim() }
        ?: card.description.orEmpty()
    TileKind.Media, TileKind.Video -> card.prose?.let(::proseText)?.takeIf { it.isNotBlank() }
        ?: card.note?.takeIf { it.isNotBlank() }
        ?: card.title?.takeIf { it.isNotBlank() } ?: card.description.orEmpty()
    else -> card.title?.takeIf { it.isNotBlank() } ?: card.description.orEmpty()
}

private fun galleryKind(card: Card): String = when (tileKind(card)) {
    TileKind.Note -> if (card.type == "Quote" || card.type == "Quotation") "Quote" else "Note"
    TileKind.Post -> when (card.type) { "XPost" -> "X"; "SubstackNote" -> "Substack"; "ThreadsPost" -> "Threads"; else -> card.siteName ?: "Post" }
    TileKind.Repository -> "Repository"
    TileKind.Article -> card.siteName?.takeIf { it.isNotBlank() } ?: "Article"
    TileKind.Video -> "Video"
    else -> card.domain?.takeIf { it.isNotBlank() } ?: when (card.type) {
        "Image" -> "Image"
        "WebPage" -> "Web page"
        "Product" -> "Product"
        "SoftwareApplication" -> "App"
        else -> "Saved"
    }
}

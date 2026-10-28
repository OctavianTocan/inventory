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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.Hyphens
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.size.Scale
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.shell.FootPressTheme
import com.octaviantocan.mymind.ui.theme.LibraryStyle

private val GITHUB_REPOSITORY = Regex("""https://(?:www\.)?github\.com/([\w.-]+)/([\w.-]+?)(?:\.git)?/?(?:[?#].*)?""")
private val VISUAL_CARD_TYPES = setOf("Image", "Screenshot", "Video", "YouTubeVideo", "InstagramReel")
private val TILE_TEXT = 13.sp
private val HIGHLIGHT_TEXT = 17.sp

/** A gallery brick keeps both the words and picture of a mixed card visible at every density. */
@Composable
internal fun GalleryTile(
    card: Card,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onLongClick: (() -> Unit)? = null,
    enabled: Boolean = true,
    highlight: Boolean = false,
    imageRequestSizePx: Int? = null,
    playVideo: Boolean = true,
) {
    val colors = LibraryStyle.colors
    val accent = cardAccent(card) ?: colors.selected
    val kind = tileKind(card)
    val picture = galleryPicture(card)
    val imageModel = rememberGalleryImage(picture, imageRequestSizePx)
    val preview = rememberPicturePreview(card)
    val text = remember(card) { galleryText(card) }
    val mixed = picture != null && text.isNotBlank() && galleryKeepsText(card)
    val paper = lerp(colors.raised, accent, 0.13f)
    val background = if (picture == null || mixed || kind == TileKind.Post) paper else parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor) ?: paper
    val heading = card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card)
    val previews = LocalVideoPreviews.current
    val video = remember(card) { previewVideo(card) }
    var length by remember(video, previews) { mutableStateOf(video?.let { previews?.length(it) }) }
    val moving = rememberSheetMotionEnabled()
    val tilePicture = @Composable { description: String?, pictureModifier: Modifier ->
        if (previews != null && video != null && playVideo && moving) {
            Box(pictureModifier) {
                VideoPreview(video, previews, { length = it }) {
                    AsyncImage(model = imageModel, contentDescription = description, placeholder = preview, error = preview,
                        contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
                }
            }
        } else {
            AsyncImage(model = imageModel, contentDescription = description, placeholder = preview, error = preview,
                contentScale = ContentScale.Crop, modifier = pictureModifier)
        }
    }
    FootPressTheme {
        Box(
            modifier.fillMaxSize().clip(RoundedCornerShape(if (highlight) 28.dp else 3.dp)).background(background)
                .semantics(mergeDescendants = true) { contentDescription = heading }
                .then(if (enabled) Modifier.combinedClickable(
                    interactionSource = null, indication = CardSheetIndication, role = Role.Button,
                    onLongClickLabel = "Card actions", onLongClick = onLongClick, onClick = onClick,
                ) else Modifier),
        ) {
            if (kind == TileKind.Post) {
                PostTile(card, text, highlight, if (picture == null) null else ({ pictureModifier -> tilePicture("Image preview", pictureModifier) }))
                Box(Modifier.fillMaxWidth().height(3.dp).background(accent))
            } else if (mixed) {
                Column(Modifier.fillMaxSize()) {
                    GalleryTextPreview(card, text, highlight, Modifier.weight(0.56f).fillMaxWidth())
                    tilePicture("Image preview", Modifier.weight(0.44f).fillMaxWidth())
                }
                Box(Modifier.fillMaxWidth().height(3.dp).background(accent))
            } else if (picture != null) {
                tilePicture(null, Modifier.fillMaxSize())
                if (highlight || kind !in setOf(TileKind.Media, TileKind.Video)) {
                    Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.35f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.78f))))
                    Text(
                        card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card),
                        Modifier.align(Alignment.BottomStart).padding(if (highlight) 18.dp else 12.dp).clearAndSetSemantics {},
                        color = Color.White, style = LibraryStyle.body.copy(
                            fontSize = if (highlight) HIGHLIGHT_TEXT else TILE_TEXT,
                            lineHeight = if (highlight) 23.sp else 18.sp,
                        ),
                        maxLines = if (highlight) 3 else 2, overflow = TextOverflow.Ellipsis,
                    )
                }
            } else {
                GalleryTextPreview(card, text, highlight, Modifier.fillMaxSize())
                Box(Modifier.fillMaxWidth().height(3.dp).background(accent))
            }
            if (picture != null && kind == TileKind.Video) BoxWithConstraints(Modifier.matchParentSize()) {
                VideoBadge(card.type == "YouTubeVideo", length?.takeIf { maxWidth >= 96.dp },
                    Modifier.align(if (mixed) Alignment.BottomStart else Alignment.TopEnd).padding(8.dp))
            }
            if (kind != TileKind.Post) GalleryCount(card, Modifier.align(if (mixed) Alignment.BottomEnd else Alignment.TopEnd))
        }
    }
}

/** What a tile loads for [picture]: its address, or a request at [imageRequestSizePx] cached under that address. */
@Composable
private fun rememberGalleryImage(picture: String?, imageRequestSizePx: Int?): Any? {
    val context = LocalContext.current
    return remember(picture, imageRequestSizePx, context) {
        picture?.let {
            val url = resolveMediaUrl(BuildConfig.SERVER_URL, it)
            if (imageRequestSizePx == null) url else ImageRequest.Builder(context).data(url)
                .size(imageRequestSizePx.coerceAtLeast(1)).scale(Scale.FIT)
                .memoryCacheKey(url).placeholderMemoryCacheKey(url).build()
        }
    }
}

/**
 * A card's paper or picture without its words, labels or touches: what the blurred fill behind a pinch is drawn from,
 * so it stays cheap. It loads the same image request as the card's [GalleryTile].
 */
@Composable
internal fun GalleryTileBackdrop(card: Card, imageRequestSizePx: Int) {
    val colors = LibraryStyle.colors
    val picture = galleryPicture(card)
    val paper = lerp(colors.raised, cardAccent(card) ?: colors.selected, 0.13f)
    val onlyPicture = picture != null && tileKind(card) != TileKind.Post && !(galleryKeepsText(card) && galleryText(card).isNotBlank())
    val imageModel = rememberGalleryImage(picture.takeIf { onlyPicture }, imageRequestSizePx)
    val background = if (onlyPicture) parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor) ?: paper else paper
    Box(Modifier.fillMaxSize().clip(RoundedCornerShape(3.dp)).background(background).clearAndSetSemantics {}) {
        if (imageModel != null) AsyncImage(model = imageModel, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
    }
}

/** How many pictures a post holds, when it holds more than one. */
@Composable
private fun GalleryCount(card: Card, modifier: Modifier) {
    val count = card.post?.gallery?.size ?: 0
    if (count > 1) Text(
        "$count", modifier.padding(8.dp).clip(RoundedCornerShape(5.dp))
            .background(Color.Black.copy(alpha = 0.55f)).padding(horizontal = 6.dp, vertical = 2.dp).clearAndSetSemantics {},
        style = LibraryStyle.label, color = Color.White,
    )
}

/**
 * A saved post drawn as the post itself: where it was posted, its words, its own picture, then who wrote it. The words
 * take what they need, up to a little over half the tile, and the picture fills the rest.
 */
@Composable
private fun PostTile(card: Card, text: String, highlight: Boolean, picture: (@Composable (Modifier) -> Unit)?) {
    val colors = LibraryStyle.colors
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val compact = maxWidth < 110.dp
        val inset = if (highlight) 16.dp else if (compact) 8.dp else 12.dp
        val author =(card.post?.author ?: card.author)?.takeIf { it.isNotBlank() && !compact && maxHeight >= 160.dp }
        val words = text.ifBlank { if (picture == null) cardHeading(card) else "" }
        val showKind = !compact && maxHeight >= 112.dp
        val wordsHeight = maxHeight * 0.55f
        Column(Modifier.fillMaxSize().padding(inset)) {
            if (showKind) {
                KindLabel(galleryKind(card))
                Spacer(Modifier.height(6.dp))
            }
            if (words.isNotBlank()) {
                TileText(words, highlight, if (picture == null) Modifier.weight(1f) else Modifier.heightIn(max = wordsHeight))
            }
            if (picture != null) Box(Modifier.padding(top = if (words.isBlank()) 0.dp else 8.dp).fillMaxWidth().weight(1f)
                .clip(RoundedCornerShape(if (compact) 4.dp else 8.dp))) {
                picture(Modifier.fillMaxSize())
                GalleryCount(card, Modifier.align(Alignment.TopEnd))
            }
            if (author != null) Text(author, Modifier.padding(top = 6.dp).clearAndSetSemantics {}, color = colors.muted,
                style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp), maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

/** The muted line naming a tile's kind or site. */
@Composable
private fun KindLabel(kind: String) {
    Text(kind, Modifier.clearAndSetSemantics {}, color = LibraryStyle.colors.muted,
        style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp), maxLines = 1, overflow = TextOverflow.Ellipsis)
}

/**
 * A tile's words at the gallery's one text size and weight, so every tile reads alike and its text keeps its size
 * while a pinch changes the tile. A word too long for the line breaks with a hyphen.
 */
@Composable
private fun TileText(text: String, highlight: Boolean, modifier: Modifier) {
    Text(
        text, modifier, color = LibraryStyle.colors.text,
        style = LibraryStyle.body.copy(
            fontSize = if (highlight) HIGHLIGHT_TEXT else TILE_TEXT, lineHeight = (if (highlight) 1.34f else 1.42f).em,
            fontWeight = FontWeight.Normal, hyphens = Hyphens.Auto, lineBreak = LineBreak.Paragraph,
        ),
        overflow = TextOverflow.Ellipsis,
    )
}

/** Text gets its own paper area; compact tiles spend that space on the excerpt before the type label. */
@Composable
private fun GalleryTextPreview(card: Card, text: String, highlight: Boolean, modifier: Modifier) {
    BoxWithConstraints(modifier) {
        val compact = maxWidth < 110.dp
        val inset = if (highlight) 16.dp else if (compact) 8.dp else 12.dp
        val words = text.ifBlank { cardHeading(card) }
        val kind = galleryKind(card)
        val showKind = !compact && maxHeight >= 112.dp && !sameWords(kind, words)
        Column(Modifier.fillMaxSize().padding(inset)) {
            if (showKind) {
                KindLabel(kind)
                Spacer(Modifier.height(6.dp))
            }
            TileText(words, highlight, Modifier.weight(1f))
        }
    }
}

/**
 * @param label A tile's site or kind line.
 * @param words The tile's text.
 * @return True when the text only repeats the label, such as a page titled with its own address.
 */
internal fun sameWords(label: String, words: String): Boolean {
    fun plain(text: String) = text.trim().lowercase().removePrefix("https://").removePrefix("http://").removePrefix("www.").trimEnd('/')
    return plain(label).isNotEmpty() && plain(label) == plain(words)
}

/**
 * Only image files are sent to Coil: video cards use their cover rather than the video file, and a moving cover or a
 * video kept on the library server shows a still the server makes of it.
 */
internal fun galleryPicture(card: Card): String? = card.cover?.let { cover ->
    cover.src?.takeIf { it.isNotBlank() }?.let { if (cover.type == "video") videoPoster(it) else it }
}
    ?: card.post?.gallery?.firstOrNull()?.src?.takeIf { it.isNotBlank() }
    ?: card.blob?.takeIf { it.type?.startsWith("image/") == true || (it.type == null && card.type in setOf("Image", "Screenshot")) }
        ?.path?.takeIf { it.isNotBlank() }
    ?: serverVideo(card)?.let(::videoPoster)
    ?: repositoryCover(card)

/** GitHub's own preview picture of a saved public repository, for repositories saved before captures kept one. */
private fun repositoryCover(card: Card): String? = card.url?.takeIf { tileKind(card) == TileKind.Repository }
    ?.let(GITHUB_REPOSITORY::matchEntire)?.let { "https://opengraph.githubassets.com/1/${it.groupValues[1]}/${it.groupValues[2]}" }

/** Whether [card]'s gallery tile is only its picture, with no words, so a pinch may cut it to any cell. */
internal fun galleryTileIsPicture(card: Card): Boolean =
    galleryPicture(card) != null && !galleryKeepsText(card) && tileKind(card).let { it == TileKind.Media || it == TileKind.Video }

/** Saved links keep their headline; visual cards also keep any note the user wrote alongside the image. */
internal fun galleryKeepsText(card: Card): Boolean = card.type !in VISUAL_CARD_TYPES
    || !card.note.isNullOrBlank() || card.prose?.let(::proseText)?.isNotBlank() == true

private fun galleryText(card: Card): String = when (tileKind(card)) {
    TileKind.Note -> noteText(card)
    TileKind.Post -> postText(card)
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

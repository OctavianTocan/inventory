package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.Hyphens
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
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
import com.octaviantocan.mymind.ui.theme.LibraryColors
import com.octaviantocan.mymind.ui.theme.LibraryStyle

private val GITHUB_REPOSITORY = Regex("""https://(?:www\.)?github\.com/([\w.-]+)/([\w.-]+?)(?:\.git)?/?(?:[?#].*)?""")
private val VISUAL_CARD_TYPES = setOf("Image", "Screenshot", "Video", "YouTubeVideo", "InstagramReel")
private val TILE_TEXT = 13.sp
private val HIGHLIGHT_TEXT = 17.sp

/** The corner radius of a gallery tile, the web showcase's `rounded-lg` (Tavi, 29 October 2026). */
internal val TILE_CORNER = 12.dp

/** The corner radius of a Top of Mind card. */
private val HIGHLIGHT_CORNER = 28.dp

/**
 * A gallery brick keeps both the words and picture of a mixed card visible at every density. It rounds its own corners
 * to [corner]; the mosaic passes 0 and rounds them itself, so a tile drawn scaled keeps [TILE_CORNER] on screen.
 */
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
    corner: Dp = if (highlight) HIGHLIGHT_CORNER else TILE_CORNER,
) {
    val colors = LibraryStyle.colors
    val accent = cardAccent(card) ?: colors.selected
    val kind = tileKind(card)
    val picture = galleryPicture(card)
    val imageModel = rememberGalleryImage(picture, imageRequestSizePx)
    val preview = rememberPicturePreview(card)
    val text = remember(card) { galleryText(card) }
    val pin = picture != null && productStandsTall(card)
    val app = picture != null && card.type == "SoftwareApplication"
    val page = picture != null && pdfStandsTall(card)
    val caption = remember(card) { videoCaption(card) }
    val mixed = picture != null && text.isNotBlank() && galleryKeepsText(card) && !pin && !app && !page
    val paper = lerp(colors.raised, accent, 0.13f)
    val background = when {
        kind == TileKind.Post -> postBrand(card.type).paper
        pin || page -> colors.background
        caption != null -> colors.raised
        picture == null || mixed || app -> paper
        else -> parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor) ?: paper
    }
    val heading = card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card)
    val previews = LocalVideoPreviews.current
    val video = remember(card) { previewVideo(card) }
    var length by remember(video, previews, card.videoLength) {
        mutableStateOf(video?.let { previews?.length(it) } ?: card.videoLength?.takeIf { it > 0 }?.let { it * 1000L })
    }
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
            modifier.fillMaxSize().then(if (corner > 0.dp) Modifier.clip(RoundedCornerShape(corner)) else Modifier).background(background)
                .semantics(mergeDescendants = true) { contentDescription = heading }
                .then(if (enabled) Modifier.combinedClickable(
                    interactionSource = null, indication = CardSheetIndication, role = Role.Button,
                    onLongClickLabel = "Card actions", onLongClick = onLongClick, onClick = onClick,
                ) else Modifier),
        ) {
            if (kind == TileKind.Post) {
                PostTile(card, text, highlight, if (picture == null) null else ({ pictureModifier -> tilePicture("Image preview", pictureModifier) }))
            } else if (pin) {
                PinTile(card, highlight) { pictureModifier -> tilePicture("Image preview", pictureModifier) }
            } else if (page) {
                PdfTile(card, highlight) { pictureModifier -> tilePicture("First page", pictureModifier) }
            } else if (app) {
                AppTile(card, highlight) { pictureModifier -> tilePicture("App icon", pictureModifier) }
                Box(Modifier.fillMaxWidth().height(3.dp).background(accent))
            } else if (caption != null) {
                VideoCaptionTile(caption, highlight) { pictureModifier -> tilePicture(null, pictureModifier) }
            } else if (mixed) {
                BoxWithConstraints(Modifier.fillMaxSize()) {
                    // The words take what they need, up to a little over half, so a big tile's picture fills the rest.
                    val most = maxHeight * 0.56f
                    Column(Modifier.fillMaxSize()) {
                        GalleryTextPreview(card, text, highlight,
                            if (highlight) Modifier.weight(0.56f).fillMaxWidth() else Modifier.heightIn(max = most).fillMaxWidth(),
                            fit = !highlight)
                        tilePicture("Image preview", Modifier.weight(if (highlight) 0.44f else 1f).fillMaxWidth())
                    }
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
internal fun rememberGalleryImage(picture: String?, imageRequestSizePx: Int?): Any? {
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
 * so it stays cheap. Its picture is the address the card's [GalleryTile] loads, which the fill finds in memory.
 */
internal fun galleryBackdrop(card: Card, colors: LibraryColors): MosaicBackdrop {
    val picture = galleryPicture(card)
    val paper = when {
        tileKind(card) == TileKind.Post -> postBrand(card.type).paper
        picture != null && (productStandsTall(card) || pdfStandsTall(card)) -> colors.background
        else -> lerp(colors.raised, cardAccent(card) ?: colors.selected, 0.13f)
    }
    val onlyPicture = picture != null && tileKind(card) != TileKind.Post && !(galleryKeepsText(card) && galleryText(card).isNotBlank())
    if (!onlyPicture) return MosaicBackdrop(paper, null)
    return MosaicBackdrop(parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor) ?: paper, resolveMediaUrl(BuildConfig.SERVER_URL, picture))
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
 * A saved post drawn as the post itself, in its network's own colours ([postBrand]): the network's mark top left, the
 * author's name with the handle under it, the post's words, then its own picture with rounded corners. With a picture
 * the words take what they need, up to a little under half of the tile under the name, and the picture fills the rest.
 * An X post that carries an article shows the article as X does, in a box under the words: its cover, then its title
 * with Article under it. It shows no likes, reposts or other counts.
 */
@Composable
private fun PostTile(card: Card, text: String, highlight: Boolean, picture: (@Composable (Modifier) -> Unit)?) {
    val brand = postBrand(card.type)
    val byline = remember(card) { postByline(card) ?: PostByline(galleryKind(card), null) }
    val article = postArticle(card)
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val compact = maxWidth < 110.dp
        val inset = if (highlight) 16.dp else if (compact) 8.dp else 12.dp
        val words = text.ifBlank { if (picture == null && article == null) cardHeading(card) else "" }
        val head = !compact && maxHeight >= 96.dp
        val wordsHeight = ((maxHeight - inset * 2 - if (head) POST_HEAD else 0.dp) * 0.45f).coerceAtLeast(19.dp)
        Column(Modifier.fillMaxSize().padding(start = inset, end = inset, top = inset - 2.dp, bottom = inset - 3.dp)) {
            if (head) {
                PostHead(brand, byline)
                Spacer(Modifier.height(6.dp))
            }
            if (words.isNotBlank()) {
                TileText(words, highlight, if (picture == null) Modifier.weight(1f) else Modifier.heightIn(max = wordsHeight), brand.text)
            }
            val gap = Modifier.padding(top = if (words.isBlank()) 0.dp else 7.dp).fillMaxWidth()
            if (article != null) PostArticleBox(article.title.orEmpty(), brand, compact, picture, if (picture != null) gap.weight(1f) else gap)
            else if (picture != null) Box(gap.weight(1f).clip(RoundedCornerShape(if (compact) 4.dp else 8.dp))) {
                picture(Modifier.fillMaxSize())
                if (card.post?.hasVideo == true) VideoBadge(false, null, Modifier.align(Alignment.TopEnd).padding(6.dp))
                else GalleryCount(card, Modifier.align(Alignment.TopEnd))
            }
        }
    }
}

/**
 * A product as a pin, as Pinterest shows one: its photo with round corners standing on the gallery's paper, the price
 * on the photo when the shop gives one, and the name in up to two lines with the brand or shop under it.
 */
@Composable
private fun PinTile(card: Card, highlight: Boolean, picture: @Composable (Modifier) -> Unit) {
    val price = remember(card) { productPrice(card) }
    val shop = productShop(card)
    Column(Modifier.fillMaxSize().padding(start = 2.dp, top = 2.dp, end = 2.dp)) {
        Box(Modifier.fillMaxWidth().weight(1f).clip(RoundedCornerShape(if (highlight) 22.dp else TILE_CORNER))) {
            picture(Modifier.fillMaxSize())
            if (price != null) PriceChip(price, Modifier.align(Alignment.BottomStart).padding(7.dp))
        }
        Column(Modifier.fillMaxWidth().padding(start = 4.dp, end = 4.dp, top = 5.dp, bottom = 6.dp)) {
            TileText(cardHeading(card), highlight, Modifier, maxLines = 2)
            if (shop != null) KindLabel(shop)
        }
    }
}

/**
 * A PDF as its first page, whole at its own shape with a soft shadow on the gallery's paper, and its name in up to two
 * lines with PDF under it.
 */
@Composable
private fun PdfTile(card: Card, highlight: Boolean, picture: @Composable (Modifier) -> Unit) {
    val width = card.cover?.width ?: card.blob?.width
    val height = card.cover?.height ?: card.blob?.height
    val ratio = if (width != null && height != null && width > 0 && height > 0) width.toFloat() / height else PAGE_RATIO
    val shape = RoundedCornerShape(if (highlight) 8.dp else 4.dp)
    Column(Modifier.fillMaxSize().padding(start = 8.dp, top = 8.dp, end = 8.dp)) {
        Box(Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.TopCenter) {
            Box(Modifier.aspectRatio(ratio.coerceIn(0.3f, 3f)).shadow(3.dp, shape).clip(shape).background(Color.White)) {
                picture(Modifier.fillMaxSize())
            }
        }
        Column(Modifier.fillMaxWidth().padding(start = 2.dp, end = 2.dp, top = 7.dp, bottom = 6.dp)) {
            TileText(cardHeading(card), highlight, Modifier, maxLines = 2)
            KindLabel("PDF")
        }
    }
}

/** Width over height of an A4 page, for a PDF whose first page's size is not known. */
private const val PAGE_RATIO = 0.707f

/** A product's price on its photo: a small white pill with a soft shadow. */
@Composable
private fun PriceChip(price: String, modifier: Modifier) {
    Text(price, modifier.shadow(2.dp, CircleShape).clip(CircleShape).background(Color.White)
        .padding(horizontal = 7.dp, vertical = 2.dp), color = Color(0xFF262520),
        style = LibraryStyle.label.copy(fontSize = 11.sp, lineHeight = 15.sp, fontWeight = FontWeight.Medium), maxLines = 1)
}

/**
 * An app as its icon, whole with rounded corners, in the middle of paper of its colour, with its name and the store it
 * is on, with the store's mark, under it.
 */
@Composable
private fun AppTile(card: Card, highlight: Boolean, picture: @Composable (Modifier) -> Unit) {
    val store = appStore(card)
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val icon = minOf(maxWidth, maxHeight) * 0.44f
        Column(Modifier.fillMaxSize().padding(start = 12.dp, end = 12.dp, top = 12.dp, bottom = 10.dp)) {
            Box(Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.Center) {
                Box(Modifier.size(icon).clip(RoundedCornerShape(icon * 0.24f))) { picture(Modifier.fillMaxSize()) }
            }
            TileText(cardHeading(card), highlight, Modifier, maxLines = 1)
            if (store != null) Row(Modifier.padding(top = 3.dp).clearAndSetSemantics {}, horizontalArrangement = Arrangement.spacedBy(5.dp),
                verticalAlignment = Alignment.CenterVertically) {
                if (store.icon != null) Image(painterResource(store.icon), null, Modifier.size(10.dp),
                    colorFilter = ColorFilter.tint(LibraryStyle.colors.text))
                KindLabel(store.name)
            }
        }
    }
}

/**
 * A video from another site: its still, with the corner badge the gallery draws over it, then its title and channel
 * under it, on white. A tall tile gives the title three lines; others one.
 */
@Composable
private fun VideoCaptionTile(caption: VideoCaption, highlight: Boolean, picture: @Composable (Modifier) -> Unit) {
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val lines = if (maxHeight > maxWidth * 1.5f) 3 else 1
        Column(Modifier.fillMaxSize()) {
            Box(Modifier.fillMaxWidth().weight(1f)) { picture(Modifier.fillMaxSize()) }
            Column(Modifier.fillMaxWidth().padding(start = 8.dp, end = 8.dp, top = 6.dp, bottom = 8.dp)) {
                TileText(caption.title, highlight, Modifier, maxLines = lines)
                if (caption.channel != null) KindLabel(caption.channel)
            }
        }
    }
}

/** Height of a post tile's head: the name and the handle under it, as [PostHead] draws them. */
private val POST_HEAD = 38.dp

/** A post tile's head: its network's mark, then the author's name with the handle under it, on the network's colour. */
@Composable
private fun PostHead(brand: PostBrand, byline: PostByline) {
    val style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp)
    Row(Modifier.fillMaxWidth().clearAndSetSemantics {}, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
        if (brand.icon != null) Image(painterResource(brand.icon), null, Modifier.padding(top = 2.dp).size(13.dp),
            colorFilter = ColorFilter.tint(brand.text))
        Column(Modifier.weight(1f)) {
            Text(byline.name, color = brand.text, style = style, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (byline.handle != null) Text(byline.handle, color = brand.muted, style = style, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

/**
 * The article an X post carries, as X draws its card in a post: a box ruled in the network's line colour holding the
 * article's cover, when there is one, over its title in up to two lines with Article under it.
 */
@Composable
private fun PostArticleBox(title: String, brand: PostBrand, compact: Boolean, picture: (@Composable (Modifier) -> Unit)?, modifier: Modifier) {
    val shape = RoundedCornerShape(if (compact) 4.dp else 8.dp)
    val style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp)
    Column(modifier.clip(shape).border(1.dp, brand.line, shape)) {
        if (picture != null) Box(Modifier.fillMaxWidth().weight(1f)) { picture(Modifier.fillMaxSize()) }
        Column(Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 6.dp).clearAndSetSemantics {}) {
            Text(title, color = brand.text, style = style.copy(fontWeight = FontWeight.Medium), maxLines = 2, overflow = TextOverflow.Ellipsis)
            Text("Article", color = brand.muted, style = style.copy(fontSize = 11.sp, lineHeight = 14.sp), maxLines = 1)
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
private fun TileText(text: String, highlight: Boolean, modifier: Modifier, color: Color = LibraryStyle.colors.text, maxLines: Int = Int.MAX_VALUE) {
    Text(
        text, modifier, color = color, maxLines = maxLines,
        style = LibraryStyle.body.copy(
            fontSize = if (highlight) HIGHLIGHT_TEXT else TILE_TEXT, lineHeight = (if (highlight) 1.34f else 1.42f).em,
            fontWeight = FontWeight.Normal, hyphens = Hyphens.Auto, lineBreak = LineBreak.Paragraph,
        ),
        overflow = TextOverflow.Ellipsis,
    )
}

/**
 * Text gets its own paper area; compact tiles spend that space on the excerpt before the type label. With [fit] the
 * area is only as tall as its words, up to the height it is given; otherwise it fills that height.
 */
@Composable
private fun GalleryTextPreview(card: Card, text: String, highlight: Boolean, modifier: Modifier, fit: Boolean = false) {
    BoxWithConstraints(modifier) {
        val compact = maxWidth < 110.dp
        val inset = if (highlight) 16.dp else if (compact) 8.dp else 12.dp
        val words = text.ifBlank { cardHeading(card) }
        val kind = galleryKind(card)
        val showKind = !compact && maxHeight >= 112.dp && !sameWords(kind, words)
        Column((if (fit) Modifier.fillMaxWidth() else Modifier.fillMaxSize()).padding(inset)) {
            if (showKind) {
                KindLabel(kind)
                Spacer(Modifier.height(6.dp))
            }
            TileText(words, highlight, if (fit) Modifier else Modifier.weight(1f))
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
    galleryPicture(card) != null && !galleryKeepsText(card) && videoCaption(card) == null && tileKind(card).let { it == TileKind.Media || it == TileKind.Video }

/** Saved links keep their headline; visual cards also keep any note the user wrote alongside the image. */
internal fun galleryKeepsText(card: Card): Boolean = card.type !in VISUAL_CARD_TYPES
    || !card.note.isNullOrBlank() || card.prose?.let(::proseText)?.isNotBlank() == true

internal fun galleryText(card: Card): String = when (tileKind(card)) {
    TileKind.Note -> noteText(card)
    TileKind.Post -> if (card.type != "RedditPost") postWords(card)
        else listOfNotNull(card.reddit?.title, postWords(card)).filter { it.isNotBlank() }.distinct().joinToString("\n")
    TileKind.Media, TileKind.Video -> card.prose?.let(::proseText)?.takeIf { it.isNotBlank() }
        ?: card.note?.takeIf { it.isNotBlank() }
        ?: card.title?.takeIf { it.isNotBlank() } ?: card.description.orEmpty()
    else -> card.title?.takeIf { it.isNotBlank() } ?: card.description.orEmpty()
}

internal fun galleryKind(card: Card): String = when (tileKind(card)) {
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

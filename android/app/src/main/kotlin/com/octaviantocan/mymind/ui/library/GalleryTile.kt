package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Density
import kotlin.math.roundToInt
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.Hyphens
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.ResolvedTextDirection
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
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

/** The line height of a tile's words, as [TileText] sets them, and of its source line, as [KindLabel] does. */
private val TILE_LINE = 18.5.sp
private val SOURCE_LINE = 16.sp

/** The corner radius of a gallery tile, the web showcase's `rounded-lg` (Tavi, 29 October 2026). */
internal val TILE_CORNER = 12.dp

/** The corner radius of a Pinned card. */
private val HIGHLIGHT_CORNER = 28.dp

/** How high a gallery tile stands off the paper: the soft shadow every print casts, as look 07 Prints draws them. */
internal val TILE_ELEVATION = 2.dp

/** A photo print's white border, on screen at every density: a picture tile is measured to its cell. */
private val PRINT_BORDER = 4.dp

/** The border round the picture of a print with words under it; it is drawn scaled with the words. */
private val PRINT_WORDS_BORDER = 5.dp

/** The border round a Pinned print's picture. */
private val HIGHLIGHT_BORDER = 6.dp

/**
 * How big a gallery tile's words show at [columns] across, in sp: 11 at four columns, 13 at three and 16 at two, the
 * Gentle rule of look 07 Prints (Tavi's pick on the card looks canvas, 1 November 2026). A tile lays its words out at
 * [TILE_TEXT] in a frame drawn scaled to its cell ([MosaicLayouts.wordsFrame]), so this is the frame's scale times 13.
 *
 * @param columns Columns across, two to four; fewer count as two and more as four.
 * @return The size the words show at.
 */
internal fun galleryTextSize(columns: Int): Float = when {
    columns >= 4 -> 11f
    columns == 3 -> 13f
    else -> 16f
}

/** How much larger than they are laid out a gallery tile's words show at [columns] across ([galleryTextSize]). */
internal fun galleryWordsScale(columns: Int): Float = galleryTextSize(columns) / TILE_TEXT.value

/** True when [card]'s tile casts the soft shadow of a print; a PDF's page casts its own on the paper instead. */
internal fun galleryTileCastsShadow(card: Card): Boolean = !pdfStandsTall(card)

/**
 * A gallery brick keeps both the words and picture of a mixed card visible at every density. Every tile is drawn as a
 * print, the look Tavi picked on 1 November 2026 (07 Prints): a picture stands in a thin white border, a card with words
 * and a picture carries the words on the print's white foot, and a card of words is a plain white card. Posts keep their
 * network's colour, products are pins in the same border, and a PDF's page stands on the paper. It rounds its own corners
 * to [corner]; the mosaic passes 0 and rounds them itself, with the print's shadow, so a tile drawn scaled keeps
 * [TILE_CORNER] on screen.
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
    val background = when {
        kind == TileKind.Post -> postBrand(card.type).paper
        page -> colors.background
        else -> colors.raised
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
    // The video badge and the picture count sit on the print's picture, inside its border.
    val marks = @Composable { scope: BoxScope ->
        with(scope) {
            if (kind == TileKind.Video) VideoBadgeFor(card.type == "YouTubeVideo", length, Modifier.matchParentSize())
            GalleryCount(card, Modifier.align(Alignment.TopEnd))
        }
    }
    val shape = RoundedCornerShape(corner)
    FootPressTheme {
        Box(
            modifier.fillMaxSize()
                .then(if (corner > 0.dp) Modifier.shadow(if (highlight && galleryTileCastsShadow(card)) TILE_ELEVATION else 0.dp, shape,
                    clip = true, ambientColor = colors.shadow, spotColor = colors.shadow) else Modifier)
                .background(background)
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
            } else if (caption != null) {
                PrintTile(TileWords(null, caption.title), caption.channel, highlight, marks) { pictureModifier -> tilePicture(null, pictureModifier) }
            } else if (mixed) {
                PrintTile(remember(card) { tileWords(card) }, remember(card) { gallerySource(card) }, highlight, marks) { pictureModifier ->
                    tilePicture("Image preview", pictureModifier)
                }
            } else if (picture != null) {
                val border = if (highlight) HIGHLIGHT_BORDER else PRINT_BORDER
                Box(Modifier.fillMaxSize().padding(border).clip(RoundedCornerShape(corner.coerceAtLeast(TILE_CORNER) - border))) {
                    tilePicture(null, Modifier.fillMaxSize())
                    if (highlight || kind !in setOf(TileKind.Media, TileKind.Video)) {
                        Box(Modifier.fillMaxSize().background(Brush.verticalGradient(0.35f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.78f))))
                        Text(
                            card.title?.takeIf { it.isNotBlank() } ?: galleryKind(card),
                            Modifier.align(Alignment.BottomStart).padding(if (highlight) 14.dp else 8.dp).clearAndSetSemantics {},
                            color = Color.White, style = LibraryStyle.body.copy(
                                fontSize = if (highlight) HIGHLIGHT_TEXT else TILE_TEXT,
                                lineHeight = if (highlight) 23.sp else 18.sp,
                            ),
                            maxLines = if (highlight) 3 else 2, overflow = TextOverflow.Ellipsis,
                        )
                    }
                    marks(this)
                }
            } else if (kind == TileKind.Audio) {
                AudioTile(card, highlight)
            } else {
                WordsTile(card, highlight)
            }
        }
    }
}

/**
 * An audio file on the paper like a card of words: a small play mark beside its length, the title, then the first lines
 * of what is said, or its summary, at the gallery's one text size.
 */
@Composable
private fun AudioTile(card: Card, highlight: Boolean) {
    val colors = LibraryStyle.colors
    val length = remember(card.blob?.duration) { audioLength(card.blob?.duration) }
    val words = remember(card) { audioWords(card) }
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val inset = if (highlight) 16.dp else if (maxWidth < 110.dp) 8.dp else 12.dp
        Column(Modifier.fillMaxSize().padding(inset), verticalArrangement = Arrangement.spacedBy(inset * 0.5f)) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                AudioPlayMark(if (highlight) 22.dp else 18.dp, colors.text, colors.raised)
                if (length != null) Text(length, color = colors.muted, style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp), maxLines = 1)
            }
            TileText(cardHeading(card), highlight, Modifier, maxLines = 2)
            if (words.isNotEmpty()) TileText(words, highlight, Modifier, color = colors.muted, maxLines = AUDIO_WORDS_LINES)
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
        picture != null && pdfStandsTall(card) -> colors.background
        else -> colors.raised
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
    val posts = postThread(card).size
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val compact = maxWidth < 110.dp
        val inset = if (highlight) 16.dp else if (compact) 8.dp else 12.dp
        val words = text.ifBlank { if (picture == null && article == null) cardHeading(card) else "" }
        val head = !compact && maxHeight >= 96.dp
        val wordsHeight = ((maxHeight - inset * 2 - if (head) POST_HEAD else 0.dp) * 0.45f).coerceAtLeast(19.dp)
        Column(Modifier.fillMaxSize().padding(start = inset, end = inset, top = inset - 2.dp, bottom = inset - 3.dp)) {
            if (head) {
                if (posts > 0) ThreadHead(brand, byline.name, posts) else PostHead(brand, byline)
                Spacer(Modifier.height(6.dp))
            }
            if (words.isNotBlank()) {
                TileText(words, highlight, if (picture == null) Modifier.weight(1f) else Modifier.heightIn(max = wordsHeight), brand.text, paper = brand.paper)
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
 * A product as a pin, as Pinterest shows one, in a print's white border: its photo with round corners, the price on the
 * photo when the shop gives one, and the name in up to two lines with the brand or shop under it on the print's foot.
 */
@Composable
private fun PinTile(card: Card, highlight: Boolean, picture: @Composable (Modifier) -> Unit) {
    val price = remember(card) { productPrice(card) }
    val shop = productShop(card)
    val border = if (highlight) HIGHLIGHT_BORDER else PRINT_WORDS_BORDER
    Column(Modifier.fillMaxSize().padding(start = border, top = border, end = border)) {
        Box(Modifier.fillMaxWidth().weight(1f).clip(RoundedCornerShape((if (highlight) HIGHLIGHT_CORNER else TILE_CORNER) - border))) {
            picture(Modifier.fillMaxSize())
            if (price != null) PriceChip(price, Modifier.align(Alignment.BottomStart).padding(7.dp))
        }
        Column(Modifier.fillMaxWidth().padding(start = 7.dp, end = 7.dp, top = 7.dp, bottom = 8.dp)) {
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
 * A print with words under its picture: the picture in a thin white border with round corners, with [marks] on it, then
 * [words] on the print's white foot, with [source] under them once the foot holds two lines of words besides it. The
 * words take what they need, up to a little over half the tile (two fifths of a tile standing tall), the last line fading when they
 * run out of room, and the picture fills the rest (Tavi's pick of 07 Prints, 1 November 2026).
 */
@Composable
private fun PrintTile(words: TileWords, source: String?, highlight: Boolean, marks: @Composable (BoxScope) -> Unit,
    picture: @Composable (Modifier) -> Unit) {
    val border = if (highlight) HIGHLIGHT_BORDER else PRINT_WORDS_BORDER
    val corner = (if (highlight) HIGHLIGHT_CORNER else TILE_CORNER) - border - 1.dp
    // Laid out, not composed, by its size: a pinch that changes the tile's frame lays it out again but composes nothing.
    Layout(content = {
        Box(Modifier.clip(RoundedCornerShape(corner))) {
            picture(Modifier.fillMaxSize())
            marks(this)
        }
        TileText(words.text(LibraryStyle.colors), highlight, Modifier)
        if (source != null) KindLabel(source)
    }, modifier = Modifier.fillMaxSize()) { measurables, constraints ->
        val width = constraints.maxWidth
        val height = constraints.maxHeight
        val inset = tileInset(width, highlight)
        val edge = border.toPx()
        val foot = height * if (height > width * 1.6f) 0.4f else 0.56f
        val line = (if (highlight) HIGHLIGHT_LINE else TILE_LINE).toPx()
        val room = foot - inset * 1.55f >= line * 2 + SOURCE_LINE.toPx()
        val wordsWidth = (width - 2 * inset).roundToInt().coerceAtLeast(0)
        val top = (inset * 0.7f).roundToInt()
        val bottom = (inset * 0.85f).roundToInt()
        val gap = (inset * 0.35f).roundToInt()
        val label = measurables.getOrNull(2)?.takeIf { room }?.measure(Constraints(maxWidth = wordsWidth))
        val labelHeight = label?.let { it.height + gap } ?: 0
        val text = measurables[1].measure(Constraints(minWidth = wordsWidth, maxWidth = wordsWidth,
            maxHeight = (foot.roundToInt() - top - bottom - labelHeight).coerceAtLeast(0)))
        val footHeight = (top + text.height + labelHeight + bottom).coerceAtMost(foot.roundToInt())
        val pictureWidth = (width - 2 * edge).roundToInt().coerceAtLeast(0)
        val pictureHeight = (height - edge.roundToInt() - footHeight).coerceAtLeast(0)
        val shown = measurables[0].measure(Constraints.fixed(pictureWidth, pictureHeight))
        layout(width, height) {
            shown.place(edge.roundToInt(), edge.roundToInt())
            val wordsTop = edge.roundToInt() + pictureHeight + top
            text.place(inset.roundToInt(), wordsTop)
            label?.place(inset.roundToInt(), wordsTop + text.height + gap)
        }
    }
}

/** A tile's inset, in pixels, at [width] pixels across: tighter on a narrow tile, wider on a Pinned card. */
private fun Density.tileInset(width: Int, highlight: Boolean): Float = when {
    highlight -> 16.dp.toPx()
    width < 110.dp.toPx() -> 8.dp.toPx()
    else -> 12.dp.toPx()
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
 * An X thread's tile head: X's mark, then the author's name and how many posts the thread has, in one muted line, as
 * the thread's name leads the tile under it (Tavi's pick of 08 Summary first, 4 November 2026).
 */
@Composable
private fun ThreadHead(brand: PostBrand, name: String, posts: Int) {
    val style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp)
    Row(Modifier.fillMaxWidth().clearAndSetSemantics {}, horizontalArrangement = Arrangement.spacedBy(7.dp), verticalAlignment = Alignment.CenterVertically) {
        if (brand.icon != null) Image(painterResource(brand.icon), null, Modifier.size(13.dp), colorFilter = ColorFilter.tint(brand.text))
        Text("$name · $posts posts", Modifier.weight(1f), color = brand.muted, style = style, maxLines = 1, overflow = TextOverflow.Ellipsis)
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

/** The muted line naming a tile's kind or where its words come from. */
@Composable
private fun KindLabel(kind: String, modifier: Modifier = Modifier) {
    Text(kind, modifier.clearAndSetSemantics {}, color = LibraryStyle.colors.muted,
        style = LibraryStyle.label.copy(fontSize = 12.sp, lineHeight = 16.sp), maxLines = 1, overflow = TextOverflow.Ellipsis)
}

/**
 * A tile's words at the gallery's one text size and weight, so every tile reads alike and its text keeps its size
 * while a pinch changes the tile. A word too long for the line breaks with a hyphen. Words with no [maxLines] that run
 * out of room stop at the last whole line that fits, which fades out towards its end, as look 07 Prints draws them;
 * with [maxLines] the last line ends in an ellipsis. The fade is drawn in [paper], the colour the words sit on.
 */
@Composable
private fun TileText(text: AnnotatedString, highlight: Boolean, modifier: Modifier, color: Color = LibraryStyle.colors.text,
    maxLines: Int = Int.MAX_VALUE, paper: Color = LibraryStyle.colors.raised) {
    val fade = maxLines == Int.MAX_VALUE
    val layout = remember { WordsLayout() }
    Text(
        text, if (fade) modifier.fadeLastLine(layout, paper) else modifier, color = color, maxLines = maxLines,
        style = LibraryStyle.body.copy(
            fontSize = if (highlight) HIGHLIGHT_TEXT else TILE_TEXT, lineHeight = if (highlight) HIGHLIGHT_LINE else TILE_LINE,
            fontWeight = FontWeight.Normal, hyphens = Hyphens.Auto, lineBreak = LineBreak.Paragraph,
        ),
        overflow = if (fade) TextOverflow.Clip else TextOverflow.Ellipsis,
        onTextLayout = { layout.result = it },
    )
}

@Composable
private fun TileText(text: String, highlight: Boolean, modifier: Modifier, color: Color = LibraryStyle.colors.text, maxLines: Int = Int.MAX_VALUE,
    paper: Color = LibraryStyle.colors.raised) = TileText(AnnotatedString(text), highlight, modifier, color, maxLines, paper)

/** The line height of a Pinned card's words. */
private val HIGHLIGHT_LINE = 22.8.sp

/** The last layout of a tile's words, which [fadeLastLine] reads as it draws; drawing follows every layout. */
private class WordsLayout {
    var result: TextLayoutResult? = null
}

/**
 * Words that overflow their box stop at the last whole line that fits, and that line fades out over the last two
 * thirds of its width, towards its end in the words' own direction. The fade is [paper] laid over the line, which on
 * opaque paper looks the same as fading the words themselves and needs no offscreen layer, so it costs nothing per frame.
 */
private fun Modifier.fadeLastLine(layout: WordsLayout, paper: Color): Modifier = drawWithContent {
    val result = layout.result
    if (result == null || !result.didOverflowHeight || result.lineCount == 0) {
        drawContent()
        return@drawWithContent
    }
    var last = -1
    while (last + 1 < result.lineCount && result.getLineBottom(last + 1) <= size.height + 0.5f) last++
    if (last < 0) return@drawWithContent
    val top = result.getLineTop(last)
    val bottom = result.getLineBottom(last)
    val rtl = result.getParagraphDirection(result.getLineStart(last)) == ResolvedTextDirection.Rtl
    val clear = paper.copy(alpha = 0f)
    val fade = if (rtl) Brush.horizontalGradient(0.06f to paper, 0.7f to clear, endX = size.width)
        else Brush.horizontalGradient(0.3f to clear, 0.94f to paper, endX = size.width)
    clipRect(bottom = bottom) { this@drawWithContent.drawContent() }
    drawRect(fade, Offset(0f, top), Size(size.width, bottom - top))
}

/**
 * A card of words as a plain white card: its words fill it, the last line fading when they run out of room, with where
 * they come from under them, a site, a quote's author, once the card holds three lines of words besides it. No kind
 * label shows (Tavi's pick of 07 Prints, 1 November 2026).
 */
@Composable
private fun WordsTile(card: Card, highlight: Boolean) {
    val words = remember(card) { tileWords(card) }
    val source = remember(card) { gallerySource(card) }
    // Laid out, not composed, by its size, as a print is ([PrintTile]).
    Layout(content = {
        TileText(words.text(LibraryStyle.colors), highlight, Modifier)
        if (source != null) KindLabel(source)
    }, modifier = Modifier.fillMaxSize()) { measurables, constraints ->
        val width = constraints.maxWidth
        val height = constraints.maxHeight
        val inset = tileInset(width, highlight)
        val line = (if (highlight) HIGHLIGHT_LINE else TILE_LINE).toPx()
        val room = height - inset * 2 >= line * 3 + SOURCE_LINE.toPx()
        val side = inset.roundToInt()
        val innerWidth = (width - 2 * side).coerceAtLeast(0)
        val innerHeight = (height - 2 * side).coerceAtLeast(0)
        val gap = (inset * 0.4f).roundToInt()
        val label = measurables.getOrNull(1)?.takeIf { room }?.measure(Constraints(maxWidth = innerWidth))
        val labelHeight = label?.let { it.height + gap } ?: 0
        val text = measurables[0].measure(Constraints.fixed(innerWidth, (innerHeight - labelHeight).coerceAtLeast(0)))
        layout(width, height) {
            text.place(side, side)
            label?.place(side, side + text.height + gap)
        }
    }
}

/**
 * What a tile's words say: a [lead], such as an article's title, in the words' own colour, then the [body] in the
 * quieter one; without a lead the body is in the words' colour.
 */
internal class TileWords(val lead: String?, val body: String) {
    /** The words as the tile sets them, in [colors]. */
    fun text(colors: LibraryColors): AnnotatedString = buildAnnotatedString {
        if (lead == null) append(body)
        else {
            append(lead)
            append("\n")
            withStyle(SpanStyle(color = colors.muted)) { append(body) }
        }
    }
}

/**
 * @param card A card drawn with words, other than a post.
 * @return Its words: a note's text; a link's title with its description under it when it has both; else its title or
 *   description, else its heading.
 */
internal fun tileWords(card: Card): TileWords {
    val text = galleryText(card).ifBlank { cardHeading(card) }
    val title = card.title?.trim()?.takeIf { it.isNotEmpty() }
    val description = card.description?.trim()?.takeIf { it.isNotEmpty() }
    return if (tileKind(card) != TileKind.Note && title != null && description != null && !sameWords(title, description) && text.trim() == title) TileWords(title, description)
    else TileWords(null, text)
}

/** True when a card's words are its title or description, not a note of its own. */
private fun galleryKeepsTitle(card: Card): Boolean = card.note.isNullOrBlank() && card.prose?.let(::proseText).isNullOrBlank()

/**
 * @param card A card drawn with words, other than a post.
 * @return Where its words come from, shown under them: a quote's author, a video's channel, a link's site or address;
 *   null for a note, for a picture with a note of its own, and when it would only repeat the words.
 */
internal fun gallerySource(card: Card): String? {
    val kind = tileKind(card)
    val source = when {
        kind == TileKind.Post -> null
        card.type == "Quote" || card.type == "Quotation" -> card.author
        kind == TileKind.Note -> null
        (kind == TileKind.Media || kind == TileKind.Video) && !galleryKeepsTitle(card) -> null
        kind == TileKind.Video -> videoCaption(card)?.channel ?: card.siteName
        else -> card.siteName ?: card.domain
    }?.trim()?.removePrefix("by ")?.trim()?.takeIf { it.isNotEmpty() } ?: return null
    return source.takeUnless { sameWords(it, galleryText(card)) }
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

/** True when [card]'s [GalleryTile] shows words: a card of words, or a picture with words on the print's foot. */
internal fun galleryTileShowsWords(card: Card): Boolean {
    if (galleryPicture(card) == null) return true
    return galleryText(card).isNotBlank() && galleryKeepsText(card) && !productStandsTall(card) &&
        card.type != "SoftwareApplication" && !pdfStandsTall(card)
}

internal fun galleryText(card: Card): String = when (tileKind(card)) {
    TileKind.Note -> noteText(card)
    TileKind.Audio -> audioWords(card).ifBlank { card.title.orEmpty() }
    TileKind.Post -> if (card.type != "RedditPost") threadName(card).takeIf { postThread(card).isNotEmpty() } ?: postWords(card)
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
    TileKind.Audio -> "Audio"
    else -> card.domain?.takeIf { it.isNotBlank() } ?: when (card.type) {
        "Image" -> "Image"
        "WebPage" -> "Web page"
        "Product" -> "Product"
        "SoftwareApplication" -> "App"
        else -> "Saved"
    }
}

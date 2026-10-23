package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Image
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import com.octaviantocan.mymind.R
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.fromHtml
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.min
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Corner radius of grid tiles (`mymind-card --border-radius`). */
val TILE_RADIUS = 6.dp

private val IMAGE_RADIUS = 5.dp
private val POST_EDGE = 5.dp
private val REPOSITORY_EDGE = 4.dp
private val REPOSITORY_NARROW = 194.dp
private val REPOSITORY_LABEL = Color(0xFF909CAE)
private const val REPOSITORY_RATIO = 164f / 154f
private const val ARTICLE_COVER_RATIO = 162f / 64f
private const val DEFAULT_RATIO = 1.5f

// The reader shows any picture whole; beyond these proportions it gains bands rather than a cut.
private val WHOLE_RATIOS = 0.5f..3f
private const val POST_MAX_LINES = 25
private const val NOTE_MAX_LINES = 40
private const val NUNITO_NORMAL_LINE = 1.364f
private val POST_TEXT = 12.sp
private val POST_BASE_TEXT = 7.68.sp
private val DECK_POST_TEXT = 16.sp
private val NOTE_TEXT = 13.2.sp

/**
 * One card in the grid, drawn the way the web front-end draws its type, with the title underneath where the web
 * shows one.
 *
 * @param card Card to draw.
 * @param onClick Called when the tile is tapped.
 * @param modifier Modifier for the tile column.
 * @param caption False leaves the title out, as in space stacks.
 * @param deck True draws the tile the way Serendipity shows it: post text at the page's base size, no coloured edge
 *   and no title.
 * @param onLongClick Called after a long press, with the long-press haptic; null leaves long presses alone.
 */
@Composable
fun CardTile(
    card: Card,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    caption: Boolean = true,
    deck: Boolean = false,
    onLongClick: (() -> Unit)? = null,
) {
    val kind = tileKind(card)
    Column(modifier.fillMaxWidth().combinedClickable(interactionSource = null, indication = null, onLongClick = onLongClick, onClick = onClick)) {
        when (kind) {
            TileKind.Note -> NoteTile(card)
            TileKind.Post -> PostTile(card, deck)
            TileKind.Repository -> RepositoryTile(card)
            TileKind.Article -> ArticleTile(card)
            TileKind.Media -> CardPicture(card, play = false)
            TileKind.Video -> CardPicture(card, play = true)
            TileKind.Plain -> PlainTile(card)
        }
        val title = card.title?.takeIf { it.isNotBlank() }
        if (caption && !deck && title != null && kind in CAPTIONED) TileCaption(title)
    }
}

private val CAPTIONED = setOf(TileKind.Post, TileKind.Repository, TileKind.Media, TileKind.Video)

/**
 * The single-line title under a tile (`mymind-card .title`).
 *
 * @param text Title to show.
 * @param modifier Modifier for the 20 dp tall line.
 */
@Composable
fun TileCaption(text: String, modifier: Modifier = Modifier) {
    Text(
        text,
        modifier.fillMaxWidth().padding(top = 3.dp).height(20.dp),
        style = TextStyle(color = MymindTheme.page.muted, fontSize = 14.sp, fontFamily = MymindTheme.fonts.body, textAlign = TextAlign.Center),
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

/**
 * The card's cover picture at its own proportions, over its dominant colour until it loads.
 *
 * @param card Card whose cover is drawn.
 * @param play True draws the play mark of video tiles over it.
 * @param modifier Modifier for the picture box.
 * @param whole True shows the whole picture at the proportions it loads with, as the reader does, never cropping it;
 *   false fills the box at the stored proportions, as grid tiles do.
 */
@Composable
fun CardPicture(card: Card, play: Boolean, modifier: Modifier = Modifier, whole: Boolean = false) {
    val src = galleryPicture(card)
    var loaded by remember(src) { mutableStateOf<Float?>(null) }
    val ratio = if (whole) (loaded ?: storedRatio(card) ?: DEFAULT_RATIO).coerceIn(WHOLE_RATIOS) else pictureRatio(card)
    Box(
        modifier
            .fillMaxWidth()
            .aspectRatio(ratio)
            .clip(RoundedCornerShape(IMAGE_RADIUS))
            .background(pictureBackground(card)),
    ) {
        if (src != null) AsyncImage(
            model = resolveMediaUrl(BuildConfig.SERVER_URL, src),
            contentDescription = null,
            contentScale = if (whole) ContentScale.Fit else ContentScale.Crop,
            modifier = Modifier.fillMaxSize(),
            onSuccess = { state ->
                val image = state.result.image
                if (whole && image.width > 0 && image.height > 0) loaded = image.width.toFloat() / image.height
            },
        )
        if (play) PlayMark(Modifier.align(Alignment.Center))
    }
}

/** Draws the card's picture (cover, post image or image file) over the whole box, cropped. */
@Composable
private fun PictureImage(card: Card, modifier: Modifier = Modifier) {
    val src = galleryPicture(card) ?: return
    AsyncImage(
        model = resolveMediaUrl(BuildConfig.SERVER_URL, src),
        contentDescription = null,
        contentScale = ContentScale.Crop,
        modifier = modifier.fillMaxSize(),
    )
}

/** Width over height of the card's cover, kept between tall portrait and wide banner. */
private fun pictureRatio(card: Card): Float = storedRatio(card)?.coerceIn(0.4f, 3f) ?: DEFAULT_RATIO

/** Width over height the card's cover or file was saved with, or null when it was not. */
private fun storedRatio(card: Card): Float? {
    val w = card.cover?.width ?: card.blob?.width
    val h = card.cover?.height ?: card.blob?.height
    return if (w != null && h != null && w > 0 && h > 0) w.toFloat() / h else null
}

/** Colour shown behind a cover while it loads. */
@Composable
private fun pictureBackground(card: Card): Color =
    parseHexColor(card.cover?.dominantColor ?: card.blob?.dominantColor) ?: MymindTheme.page.surface

/** Phosphor play-circle over video tiles. */
@Composable
private fun PlayMark(modifier: Modifier = Modifier) {
    BoxWithConstraints(modifier) {
        val side: Dp = min(min(maxWidth * 0.225f, maxHeight * 0.4f), 74.dp)
        Image(painterResource(R.drawable.ic_play_circle), null,
            Modifier.size(side).background(Color.Black.copy(alpha = 0.15f), androidx.compose.foundation.shape.CircleShape),
            colorFilter = ColorFilter.tint(Color.White))
    }
}

/** A note's text on a plain tile (`mymind-card.content`, 12 px prose with 1.1 em paragraphs). */
@Composable
private fun NoteTile(card: Card) {
    val style = TextStyle(color = MymindTheme.page.text2, fontSize = NOTE_TEXT, lineHeight = 1.5.em, fontFamily = MymindTheme.fonts.body)
    val gap = with(LocalDensity.current) { NOTE_TEXT.toDp() }
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(TILE_RADIUS))
            .background(MymindTheme.page.surface)
            .padding(horizontal = 12.dp, vertical = 17.dp)
            .heightIn(max = with(LocalDensity.current) { (NOTE_TEXT * 1.5f * NOTE_MAX_LINES).toDp() })
            .clipToBounds(),
        verticalArrangement = Arrangement.spacedBy(gap),
    ) {
        noteText(card).split(Regex("\\n\\s*\\n")).filter { it.isNotBlank() }.forEach { Text(it.trim(), style = style) }
    }
}

/** A social post: coloured top edge, platform mark, paragraphs, first pictures and author. */
@Composable
private fun PostTile(card: Card, deck: Boolean) {
    val page = MymindTheme.page
    val size = if (deck) DECK_POST_TEXT else POST_TEXT
    val body = TextStyle(color = page.text2, fontSize = size, lineHeight = 1.3.em, fontFamily = MymindTheme.fonts.body)
    val gap = with(LocalDensity.current) { (size * 1.3f).toDp() }
    val links = TextLinkStyles(SpanStyle(color = page.link))
    val paragraphs = remember(card.id, card.post?.bodyHtml) { postParagraphs(card) }
    // On the web the edge sits above the tile's box (`.body` margin-top -5px), and Serendipity clips it away.
    Column(
        Modifier
            .then(if (deck) Modifier else Modifier.overhangTop(POST_EDGE))
            .fillMaxWidth()
            .clip(RoundedCornerShape(TILE_RADIUS))
            .background(page.surface),
    ) {
        if (!deck) Box(Modifier.fillMaxWidth().height(POST_EDGE).background(postColor(card.type)))
        val base = if (deck) DECK_POST_TEXT else POST_BASE_TEXT
        val maxBody = with(LocalDensity.current) { (base * NUNITO_NORMAL_LINE * POST_MAX_LINES).toDp() }
        Column(Modifier.padding(16.dp).heightIn(max = maxBody).clipToBounds()) {
            PlatformMark(card.type)
            paragraphs.forEach { html ->
                Spacer(Modifier.height(gap))
                Text(AnnotatedString.fromHtml(html, links), style = body)
            }
            val gallery = card.post?.gallery.orEmpty().take(2)
            if (gallery.isNotEmpty()) {
                Spacer(Modifier.height(gap))
                PostGallery(card, gallery.size)
            }
            card.post?.author?.takeIf { it.isNotBlank() }?.let {
                Spacer(Modifier.height(gap))
                Text(it, style = body.copy(color = page.faint, fontFamily = MymindTheme.fonts.body, fontWeight = FontWeight.SemiBold), maxLines = 1)
            }
        }
    }
}

/** Splits a post's HTML into paragraphs, each still HTML. */
private fun postParagraphs(card: Card): List<String> {
    val html = card.post?.bodyHtml ?: return listOfNotNull(card.description?.takeIf { it.isNotBlank() })
    return html.split(Regex("</p>|<br\\s*/?>\\s*<br\\s*/?>", RegexOption.IGNORE_CASE))
        .map { it.replace(Regex("<p[^>]*>", RegexOption.IGNORE_CASE), "").trim() }
        .filter { it.isNotEmpty() }
}

/** Lays the content out [amount] higher than its slot and shortens the slot by as much. */
private fun Modifier.overhangTop(amount: Dp) = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    val shift = amount.roundToPx()
    layout(placeable.width, (placeable.height - shift).coerceAtLeast(0)) { placeable.place(0, -shift) }
}

/** The 20 dp tall platform mark at the top of a post tile. */
@Composable
private fun PlatformMark(type: String) {
    Box(Modifier.height(20.dp)) {
        if (type == "XPost") {
            Text("𝕏", style = TextStyle(color = MymindTheme.page.icon, fontSize = 18.sp, lineHeight = 20.sp))
        }
    }
}

/** Up to two square pictures of a post side by side. */
@Composable
private fun PostGallery(card: Card, count: Int) {
    Row(horizontalArrangement = Arrangement.spacedBy(5.dp)) {
        repeat(count) { index ->
            val src = card.post?.gallery?.get(index)?.src
            Box(
                Modifier
                    .weight(1f)
                    .aspectRatio(1f)
                    .clip(RoundedCornerShape(2.dp))
                    .background(MymindTheme.page.border),
            ) {
                if (src != null) {
                    AsyncImage(
                        model = resolveMediaUrl(BuildConfig.SERVER_URL, src),
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize(),
                    )
                }
            }
        }
        if (count == 1) Spacer(Modifier.weight(1f))
    }
}

/** A repository: green top edge, name in the serif face sized to the tile and the host underneath. */
@Composable
private fun RepositoryTile(card: Card) {
    val page = MymindTheme.page
    val shape = RoundedCornerShape(TILE_RADIUS)
    BoxWithConstraints(Modifier.fillMaxWidth()) {
        val narrow = maxWidth <= REPOSITORY_NARROW
        val nameSize = (maxWidth.value * if (narrow) 0.10f else 0.12f).coerceIn(14f, 26f).sp
        // The web ratio sizes the box inside the green edge, so the edge adds to the height.
        Column(
            Modifier
                .fillMaxWidth()
                .height(maxWidth / REPOSITORY_RATIO + REPOSITORY_EDGE)
                .clip(shape)
                .background(REPOSITORY_ACCENT)
                .padding(top = REPOSITORY_EDGE)
                .clip(shape)
                .background(page.surface)
                .padding(12.dp),
            verticalArrangement = Arrangement.Bottom,
        ) {
            Text(
                card.title?.substringAfterLast('/')?.takeIf { it.isNotBlank() } ?: cardHeading(card),
                style = TextStyle(color = page.heading, fontSize = nameSize, lineHeight = 1.2.em, fontFamily = MymindTheme.fonts.heading, letterSpacing = (-0.05).em),
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(if (narrow) 4.dp else 18.dp))
            Text(
                card.siteName ?: card.domain.orEmpty(),
                style = TextStyle(
                    color = REPOSITORY_LABEL,
                    fontSize = 14.sp,
                    lineHeight = 19.sp,
                    fontFamily = MymindTheme.fonts.body, fontWeight = FontWeight.Medium,
                    letterSpacing = (-0.03).em,
                ),
                maxLines = 1,
            )
        }
    }
}

/** An article: square tile with the headline above the cover. */
@Composable
private fun ArticleTile(card: Card) {
    val page = MymindTheme.page
    Column(
        Modifier
            .fillMaxWidth()
            .aspectRatio(1f)
            .clip(RoundedCornerShape(TILE_RADIUS))
            .background(page.surface),
    ) {
        Column(Modifier.weight(1f).fillMaxWidth().padding(start = 12.dp, end = 12.dp, top = 12.dp, bottom = 1.dp)) {
            Spacer(Modifier.width(16.dp).height(14.dp))
            Spacer(Modifier.weight(1f))
            Text(
                cardHeading(card),
                Modifier.padding(bottom = 8.dp),
                style = TextStyle(
                    color = page.heading,
                    fontSize = 17.sp,
                    lineHeight = 1.06.em,
                    fontFamily = MymindTheme.fonts.heading,
                    letterSpacing = (-0.03).em,
                    lineHeightStyle = LineHeightStyle(LineHeightStyle.Alignment.Center, LineHeightStyle.Trim.None),
                ),
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (card.cover != null) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(ARTICLE_COVER_RATIO)
                    .background(pictureBackground(card)),
            ) { PictureImage(card) }
        }
    }
}

/** A card without a picture or special layout: its title on a plain tile. */
@Composable
private fun PlainTile(card: Card) {
    Box(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(TILE_RADIUS))
            .background(MymindTheme.page.surface)
            .padding(16.dp),
    ) {
        Text(cardHeading(card), style = TextStyle(color = MymindTheme.page.text2, fontSize = 16.sp, fontFamily = MymindTheme.fonts.body))
    }
}

/**
 * A card in the Top of Mind row: a 124 x 100 dp crop of the card.
 *
 * @param card Pinned card.
 * @param onClick Called when the tile is tapped.
 * @param modifier Modifier for the tile.
 */
@Composable
fun TopOfMindTile(card: Card, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val page = MymindTheme.page
    val kind = tileKind(card)
    Column(
        modifier
            .size(width = 124.dp, height = 100.dp)
            .clip(RoundedCornerShape(TILE_RADIUS))
            .background(if (kind == TileKind.Media || kind == TileKind.Video) pictureBackground(card) else page.surface)
            .clickable(interactionSource = null, indication = null, onClick = onClick),
    ) {
        when (kind) {
            TileKind.Media, TileKind.Video -> PictureImage(card)
            else -> {
                Text(
                    if (kind == TileKind.Note) noteText(card) else cardHeading(card),
                    Modifier.padding(start = 10.dp, end = 8.dp, top = 8.dp),
                    style = TextStyle(color = page.heading, fontSize = 17.sp, lineHeight = 1.06.em, fontFamily = MymindTheme.fonts.heading, letterSpacing = (-0.03).em),
                    maxLines = 3,
                    overflow = TextOverflow.Clip,
                )
                Spacer(Modifier.weight(1f))
                if (card.cover?.src != null) {
                    Box(Modifier.fillMaxWidth().height(34.dp).background(pictureBackground(card))) { PictureImage(card) }
                }
            }
        }
    }
}

package com.octaviantocan.mymind.ui.moment

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.SubcomposeLayout
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.library.CardThumbnail
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.theme.AppFontFamily
import java.text.NumberFormat
import java.util.Random
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.sin

/**
 * What the card of a find shows: what was saved, by its name once ThyMind knows it, its kind, its picture once there
 * is one, ThyMind's tags as they arrive, and its number in the library.
 *
 * @property title What was saved.
 * @property kind Where it is from or what it is, such as The Atlantic or Photo.
 * @property icon A mark for the kind, shown in the window while there is no picture.
 * @property card The saved card as the library has it, once it does.
 * @property shared A shared picture's address, shown until the card has its own.
 * @property tags The tags on the find.
 * @property number The find's number in the library, or null when the library is not counted.
 */
@Immutable
data class MomentFind(
    val title: String,
    val kind: String,
    @param:DrawableRes val icon: Int,
    val card: Card? = null,
    val shared: String? = null,
    val tags: List<String> = emptyList(),
    val number: Int? = null,
)

/**
 * Sizes on the card in the prototype's units: the card is 200 by 284. Text grows with the phone's text size up to
 * [TEXT_BOOST_MAX], so a phone that draws text bigger still fits it on the card.
 */
@Immutable
internal class CardUnits(private val px: Float, private val density: Float, private val fontScale: Float) {
    fun dp(v: Float): Dp = (v * px / density).dp
    fun px(v: Float): Float = v * px
    fun sp(v: Float): TextUnit = (v * px * min(fontScale, TEXT_BOOST_MAX) / (density * fontScale)).sp
}

/** How much bigger than the card's own size its text may get with a bigger system text size. */
internal const val TEXT_BOOST_MAX = 1.15f

private val Ink = Color(0xFFF4F2EF)
private val Gold = Color(0xFFF3E2B0)
private val GoldLine = Color(0xFFE9D4A0)
private val Bezel = Color(0xFFD9B76A)
private val Sub = Color(0xFFC9D6EA)

/** The gem in five facet tones, for each rarity; the card's back always shows the gold one. */
internal fun gemTones(r: Rarity?): List<Color> = when (r) {
    Rarity.Common -> listOf(0xFFF4F2EF, 0xFFCFE7E2, 0xFFA7D3CC, 0xFF6FA7A0, 0xFF3F7A82)
    Rarity.Rare -> listOf(0xFFEAF4FA, 0xFF9CCBE3, 0xFF3E8DB5, 0xFF2A6A8C, 0xFF1D4A63)
    else -> listOf(0xFFFFF3D1, 0xFFF2C46A, 0xFFE2A23B, 0xFFB07A22, 0xFF8A5A14)
}.map { Color(it) }

/**
 * The find's card face, the prototype's star chart look: night blue with a gold-ringed round window for the picture,
 * the rarity and its stars along the top, the name, three facts and the tags.
 *
 * @param find What the card shows.
 * @param rarity How rare the find is.
 * @param stars How many of its stars are stamped so far.
 * @param units The card's units.
 * @param light Where the light falls as the card tilts, from -1 to 1 each way, and the sheen's place and strength.
 */
@Composable
internal fun FindFront(find: MomentFind, rarity: Rarity, stars: Int, units: CardUnits, light: () -> FloatArray, modifier: Modifier = Modifier) {
    val u = units
    val field = remember { starField(7, 140) }
    Box(
        modifier.size(u.dp(200f), u.dp(284f)).clip(RoundedCornerShape(u.dp(16f)))
            .drawBehind { face(this, u, field, rarity, stars) }
            .drawWithContent {
                drawContent()
                bezel(this, u)
                cardLight(this, light())
            },
    ) {
        Text(rarity.label, Modifier.offset(u.dp(16f), u.dp(12.5f)), style = style(u, 12f, Gold, FontWeight.Bold), maxLines = 1)
        Box(Modifier.offset(u.dp(37f), u.dp(35f)).size(u.dp(126f)).clip(CircleShape).background(Color(0xFF0B1A36))) {
            FindPicture(find, u)
        }
        Column(
            Modifier.fillMaxWidth().offset(y = u.dp(176f)).padding(horizontal = u.dp(12f)),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(find.title, style = style(u, 15f, Ink, FontWeight.Medium, lineHeight = 18f, center = true), maxLines = 2, overflow = TextOverflow.Ellipsis)
            Spacer(Modifier.height(u.dp(7f)))
            Row(horizontalArrangement = Arrangement.spacedBy(u.dp(4f)), verticalAlignment = Alignment.CenterVertically) {
                Chip(u, find.kind, Sub, Modifier.weight(1f, fill = false)) {
                    Image(painterResource(find.icon), null, Modifier.size(u.dp(10f)), colorFilter = ColorFilter.tint(Sub))
                }
                find.number?.let { Chip(u, "No. ${NumberFormat.getIntegerInstance().format(it)}", Sub) }
            }
            if (find.tags.isNotEmpty()) {
                Spacer(Modifier.height(u.dp(5f)))
                TagRow(u, find.tags)
            }
        }
    }
}

/** The find's card back: a star chart round the gold gem. */
@Composable
internal fun FindBack(units: CardUnits, modifier: Modifier = Modifier) {
    val field = remember { starField(11, 220) }
    Box(modifier.size(units.dp(200f), units.dp(284f)).clip(RoundedCornerShape(units.dp(16f))).drawBehind { back(this, units, field) })
}

@Composable
private fun FindPicture(find: MomentFind, u: CardUnits) {
    val card = find.card
    when {
        card != null && galleryPicture(card) != null -> CardThumbnail(card, Modifier.fillMaxSize())
        find.shared != null -> AsyncImage(model = find.shared, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        else -> Box(Modifier.fillMaxSize().background(Brush.radialGradient(listOf(Color(0xFF1B3A6A), Color(0xFF0B1A36)))), contentAlignment = Alignment.Center) {
            Image(painterResource(find.icon), null, Modifier.size(u.dp(40f)), colorFilter = ColorFilter.tint(Gold.copy(alpha = 0.9f)))
        }
    }
}

@Composable
private fun Chip(u: CardUnits, text: String, ink: Color, modifier: Modifier = Modifier, icon: (@Composable () -> Unit)? = null) {
    Row(
        modifier.height(u.dp(19f)).clip(CircleShape).background(Sub.copy(alpha = 0.08f))
            .border(1.dp, GoldLine.copy(alpha = 0.3f), CircleShape).padding(horizontal = u.dp(6f)),
        horizontalArrangement = Arrangement.spacedBy(u.dp(3f)),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        icon?.invoke()
        Text(text, style = style(u, 11f, ink, FontWeight.Medium), maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/** ThyMind's tags in one line; those that do not fit become a +N chip at its end. */
@Composable
private fun TagRow(u: CardUnits, tags: List<String>) {
    val gap = u.px(4f)
    SubcomposeLayout(Modifier.fillMaxWidth()) { constraints ->
        val loose = Constraints(maxWidth = constraints.maxWidth)
        val chips = subcompose("tags") { tags.forEach { TagChip(u, "#$it") } }.map { it.measure(loose) }
        var shown = chips.size
        var width = chips.sumOf { it.width } + gap * (chips.size - 1).coerceAtLeast(0)
        var more = emptyList<androidx.compose.ui.layout.Placeable>()
        if (width > constraints.maxWidth) {
            for (k in chips.size - 1 downTo 0) {
                val plus = subcompose("more$k") { TagChip(u, "+${chips.size - k}") }.map { it.measure(loose) }
                val used = chips.take(k).sumOf { it.width } + gap * k + plus.sumOf { it.width }
                if (used <= constraints.maxWidth || k == 0) { shown = k; more = plus; width = used; break }
            }
        }
        val height = chips.maxOfOrNull { it.height } ?: 0
        layout(constraints.maxWidth, height) {
            var x = ((constraints.maxWidth - width) / 2f).toInt().coerceAtLeast(0)
            (chips.take(shown) + more).forEach { it.placeRelative(x, 0); x += it.width + gap.toInt() }
        }
    }
}

@Composable
private fun TagChip(u: CardUnits, text: String) {
    Box(Modifier.height(u.dp(18f)).clip(CircleShape).background(Sub.copy(alpha = 0.12f)).padding(horizontal = u.dp(6f)), contentAlignment = Alignment.Center) {
        Text(text, style = style(u, 10.5f, Color(0xFFE8F0FF), FontWeight.Medium), maxLines = 1)
    }
}

/** The Inventory gem in flat facets, [size] units across. */
@Composable
internal fun Gem(u: CardUnits, size: Float, tones: List<Color>, modifier: Modifier = Modifier) {
    Box(modifier.size(u.dp(size)).drawBehind { gem(this, Offset(this.size.width / 2f, this.size.height / 2f), this.size.width, tones) })
}

/** Draws the gem centred on [center], [size] pixels across. */
internal fun gem(scope: DrawScope, center: Offset, size: Float, tones: List<Color>) = with(scope) {
    val k = size / 48f
    translate(center.x - size / 2f, center.y - size / 2f) {
        fun facet(color: Color, vararg pts: Float) {
            val p = Path().apply { moveTo(pts[0] * k, pts[1] * k); for (i in 2 until pts.size step 2) lineTo(pts[i] * k, pts[i + 1] * k); close() }
            drawPath(p, color)
        }
        facet(tones[3], 24f, 3f, 42f, 18f, 24f, 45f, 6f, 18f)
        facet(tones[1], 6f, 18f, 15f, 9f, 24f, 3f, 33f, 9f, 42f, 18f)
        facet(tones[0], 15f, 9f, 24f, 18f, 33f, 9f, 24f, 3f)
        facet(tones[2], 6f, 18f, 24f, 18f, 24f, 45f)
        facet(tones[4], 24f, 18f, 42f, 18f, 24f, 45f)
        facet(tones[2], 15f, 9f, 24f, 18f, 6f, 18f)
        facet(tones[3], 33f, 9f, 42f, 18f, 24f, 18f)
        val outline = Path().apply { moveTo(24f * k, 3f * k); lineTo(42f * k, 18f * k); lineTo(24f * k, 45f * k); lineTo(6f * k, 18f * k); close() }
        drawPath(outline, Color.White.copy(alpha = 0.5f), style = Stroke(0.8f * k))
    }
}

private fun style(u: CardUnits, size: Float, color: Color, weight: FontWeight, lineHeight: Float? = null, center: Boolean = false) = TextStyle(
    color = color, fontSize = u.sp(size), lineHeight = u.sp(lineHeight ?: (size * 1.25f)), fontWeight = weight, fontFamily = AppFontFamily,
    textAlign = if (center) TextAlign.Center else TextAlign.Start,
)

/** Seeded stars for a card side: x and y in units, radius and opacity. */
private fun starField(seed: Long, n: Int): FloatArray {
    val r = Random(seed)
    return FloatArray(n * 4).also { a ->
        for (i in 0 until n) { a[i * 4] = r.nextFloat() * 200f; a[i * 4 + 1] = r.nextFloat() * 284f; a[i * 4 + 2] = r.nextFloat() * 0.9f + 0.2f; a[i * 4 + 3] = 0.2f + r.nextFloat() * 0.7f }
    }
}

private fun face(scope: DrawScope, u: CardUnits, field: FloatArray, rarity: Rarity, stars: Int) = with(scope) {
    val k = u.px(1f)
    drawRect(Brush.radialGradient(0f to Color(0xFF16305A), 0.55f to Color(0xFF0B1A36), 1f to Color(0xFF050C1C), center = Offset(100f * k, 92f * k), radius = 220f * k))
    for (i in field.indices step 4) drawCircle(Color(0xFFE8F0FF).copy(alpha = field[i + 3]), field[i + 2] * k, Offset(field[i] * k, field[i + 1] * k))
    // Gold rings and ticks round the window, kept off the top row.
    clipRect(top = 36f * k) {
        val c = Offset(100f * k, 98f * k)
        drawCircle(GoldLine.copy(alpha = 0.35f), 72f * k, c, style = Stroke(0.5f * k))
        drawCircle(GoldLine.copy(alpha = 0.35f), 78f * k, c, style = Stroke(0.5f * k))
        for (i in 0 until 72) {
            val a = i / 72f * 2f * PI.toFloat()
            val len = if (i % 6 == 0) 6f else 3f
            drawLine(GoldLine.copy(alpha = 0.35f), c + Offset(cos(a), sin(a)) * (78f * k), c + Offset(cos(a), sin(a)) * ((78f + len) * k), 0.5f * k)
        }
    }
    val cons = listOf(150f to 160f, 166f to 150f, 176f to 164f, 188f to 150f)
    for (i in 1 until cons.size) drawLine(GoldLine.copy(alpha = 0.55f), Offset(cons[i - 1].first * k, cons[i - 1].second * k), Offset(cons[i].first * k, cons[i].second * k), 0.6f * k)
    cons.forEach { (x, y) -> starShape(this, Offset(x * k, y * k), 2.2f * k, Gold) }
    for (i in 0 until rarity.stars) {
        val x = 184f - (rarity.stars - 1 - i) * 13f
        starShape(this, Offset(x * k, 21f * k), 5.6f * k, if (i < stars) rarity.color else GoldLine.copy(alpha = 0.22f))
    }
}

private fun bezel(scope: DrawScope, u: CardUnits) = with(scope) {
    val k = u.px(1f)
    val c = Offset(100f * k, 98f * k)
    drawCircle(Color.Black.copy(alpha = 0.35f), 63f * k, c, style = Stroke(1.5f * k))
    drawCircle(Brush.sweepGradient(listOf(Bezel, Color(0xFFFFF3D1), Bezel, Color(0xFF8A6A2A), Bezel), c), 64.5f * k, c, style = Stroke(3f * k))
}

/** Light on the face: a soft highlight that follows the tilt, and the sheen that crosses it once. */
private fun cardLight(scope: DrawScope, light: FloatArray) = with(scope) {
    val (tx, ty, sheen, sheenA) = light
    val w = size.width; val h = size.height
    drawRect(
        Brush.radialGradient(listOf(Color.White.copy(alpha = 0.10f + 0.08f * (kotlin.math.abs(tx) + kotlin.math.abs(ty))), Color.Transparent),
            center = Offset(w * (0.5f - 0.7f * tx), h * (0.35f + 0.6f * ty)), radius = w * 0.9f),
        blendMode = BlendMode.Plus,
    )
    if (sheenA > 0f) {
        rotate(25f) {
            val x = lerp(-w * 0.9f, w * 1.9f, (sheen + 0.4f) / 2.3f)
            val band = w * 0.28f
            drawRect(
                Brush.horizontalGradient(listOf(Color.Transparent, Color.White.copy(alpha = sheenA), Color.Transparent), startX = x - band, endX = x + band),
                topLeft = Offset(x - band, -h), size = Size(band * 2f, h * 3f), blendMode = BlendMode.Plus,
            )
        }
    }
}

private fun back(scope: DrawScope, u: CardUnits, field: FloatArray) = with(scope) {
    val k = u.px(1f)
    val c = Offset(100f * k, 142f * k)
    drawRect(Brush.radialGradient(listOf(Color(0xFF132A50), Color(0xFF050C1C)), center = c, radius = 200f * k))
    for (i in field.indices step 4) drawCircle(Color(0xFFE8F0FF).copy(alpha = field[i + 3]), field[i + 2] * k, Offset(field[i] * k, field[i + 1] * k))
    val line = GoldLine.copy(alpha = 0.5f)
    drawCircle(line, 58f * k, c, style = Stroke(0.7f * k))
    drawCircle(line, 64f * k, c, style = Stroke(0.7f * k))
    for (i in 0 until 8) {
        val a = (i + 1) / 8f * 2f * PI.toFloat() - PI.toFloat() / 2f
        drawLine(line, c + Offset(cos(a), sin(a)) * (18f * k), c + Offset(cos(a), sin(a)) * (54f * k), 0.7f * k)
    }
    gem(this, c, 30f * k, gemTones(null))
    drawRoundRect(GoldLine.copy(alpha = 0.8f), Offset(7f * k, 7f * k), Size(186f * k, 270f * k), androidx.compose.ui.geometry.CornerRadius(10f * k), style = Stroke(0.7f * k))
}

private fun starShape(scope: DrawScope, at: Offset, r: Float, color: Color) {
    val p = Path()
    for (i in 0 until 10) {
        val a = -PI.toFloat() / 2f + i * PI.toFloat() / 5f
        val rr = if (i % 2 == 1) r * 0.45f else r
        val x = at.x + cos(a) * rr; val y = at.y + sin(a) * rr
        if (i == 0) p.moveTo(x, y) else p.lineTo(x, y)
    }
    p.close()
    scope.drawPath(p, color)
}

private operator fun FloatArray.component1() = this[0]
private operator fun FloatArray.component2() = this[1]
private operator fun FloatArray.component3() = this[2]
private operator fun FloatArray.component4() = this[3]


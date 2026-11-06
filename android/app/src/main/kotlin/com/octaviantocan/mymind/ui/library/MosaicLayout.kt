package com.octaviantocan.mymind.ui.library

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.Offset
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Track
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.exp
import kotlin.math.floor
import kotlin.math.ln
import kotlin.math.roundToInt
import kotlin.math.sqrt

/** One band of the gallery, in pixels, in the server's original card order: a lazy-list slot of one or two rows. */
internal data class MosaicRow(val cards: List<Card>, val widths: List<Float>, val height: Float) {
    val key: String get() = cards.first().id
}

/** How many grid squares a card covers across and down at two columns or more. */
internal data class MosaicSpan(val across: Int, val down: Int)

/** Pictures this tall or taller (width over height), such as phone screenshots, stand two squares high. */
private const val TALL_RATIO = 0.62f

/** Pictures this wide or wider may lie two squares across. */
private const val WIDE_RATIO = 1.6f

/**
 * The card's shape in the grid, as Google Photos breaks up its squares and the web showcase spreads its pieces: a card
 * whose picture is tall, such as a screenshot, stands two squares high, about half the wide pictures shown alone lie two
 * across, and about one card with a picture in seven is shown big, two by two, its words included. A social post with a
 * picture or more words than a square holds stands two squares high ([postStandsTall]), and so does a product with a
 * photo, drawn as a pin ([productStandsTall]), and a PDF, its first page whole ([pdfStandsTall]); a card of words alone
 * stays one square. The choice is fixed by the card's id, so a card keeps its shape at every density and across reloads.
 */
internal fun mosaicSpan(card: Card): MosaicSpan {
    if (postStandsTall(card) || productStandsTall(card) || pdfStandsTall(card)) return MosaicSpan(1, 2)
    if (galleryPicture(card) == null) return MosaicSpan(1, 1)
    val ratio = storedRatio(card) ?: return MosaicSpan(1, 1)
    val pick = card.id.hashCode().let { (it xor (it ushr 16)) and Int.MAX_VALUE }
    return when {
        ratio <= TALL_RATIO -> MosaicSpan(1, 2)
        // Words over a picture two squares wide would leave both a thin strip, so only a picture alone lies across.
        ratio >= WIDE_RATIO && pick % 2 == 0 && !galleryKeepsText(card) -> MosaicSpan(2, 1)
        pick % 7 == 3 -> MosaicSpan(2, 2)
        else -> MosaicSpan(1, 1)
    }
}

/** Width over height of the card's stored picture as the person turned it, or null when it has no usable size. */
internal fun storedRatio(card: Card): Float? {
    val width = card.cover?.width ?: card.blob?.width
    val height = card.cover?.height ?: card.blob?.height
    if (width == null || height == null || width <= 0 || height <= 0) return null
    return if (card.turnedSideways) height.toFloat() / width else width.toFloat() / height
}

/** The bands of [mosaicLevel], for callers that only need which cards share a slot. */
internal fun mosaicRows(cards: List<Card>, width: Float, columns: Int, gap: Float, tiles: MosaicTiles = GalleryMosaicTiles): List<MosaicRow> =
    mosaicLevel(cards, width, columns, gap, tiles).rows

/**
 * Lays the cards out at one density without reordering or dropping any. One column shows each card across the whole
 * width at its own proportions. Two to four columns share square cells in bands of one or two rows, filled row by row
 * in card order: a card two squares high starts a two-row band, and a card that does not fit where it falls takes one
 * square there, so a band is never left with a hole before the last card. Card tops never decrease with the order.
 * [tiles] gives each card its shape.
 */
internal fun mosaicLevel(cards: List<Card>, width: Float, columns: Int, gap: Float, tiles: MosaicTiles = GalleryMosaicTiles): MosaicLayouts.Level {
    if (cards.isEmpty() || !width.isFinite() || width <= 0f) return MosaicLayouts.Level(emptyList(), emptyList(), emptyList(), IntArray(0), 0f, 0f, 0f, 0f)
    val across = columns.coerceIn(1, 4)
    val space = mosaicGap(width, gap)
    val cells = ArrayList<Rect>(cards.size)
    val rows = ArrayList<MosaicRow>()
    val starts = ArrayList<Int>()
    val rowOf = IntArray(cards.size)
    var top = 0f
    val unit = if (across == 1) width else (width - space * (across - 1)) / across
    if (across == 1) {
        cards.forEachIndexed { index, card ->
            val height = width / tiles.aspectRatio(card)
            starts += index
            rowOf[index] = index
            cells += Rect(0f, top, width, top + height)
            rows += MosaicRow(listOf(card), listOf(width), height)
            top += height + space
        }
    } else {
        val step = unit + space
        val spans = cards.map { tiles.span(it) }
        var index = 0
        while (index < cards.size) {
            val start = index
            val taken = Array(2) { BooleanArray(across) }
            var down = 1
            var row = 0
            var column = 0
            while (index < cards.size) {
                while (row < down && taken[row][column]) {
                    column++
                    if (column == across) { column = 0; row++ }
                }
                if (row >= down) break
                val span = spans[index]
                var tall = if (row == 0) span.down else 1
                var wide = span.across
                fun free(w: Int, h: Int) = column + w <= across && (row until row + h).all { r -> (column until column + w).none { taken[r][it] } }
                if (!free(wide, tall)) {
                    wide = 1
                    if (!free(1, tall)) tall = 1
                }
                for (r in row until row + tall) for (c in column until column + wide) taken[r][c] = true
                if (tall == 2) down = 2
                rowOf[index] = rows.size
                cells += Rect(column * step, top + row * step, column * step + wide * unit + (wide - 1) * space,
                    top + row * step + tall * unit + (tall - 1) * space)
                index++
                column += wide
                if (column >= across) { column = 0; row++ }
            }
            val height = down * unit + (down - 1) * space
            starts += start
            rows += MosaicRow(cards.subList(start, index), cells.subList(start, index).map { it.width }, height)
            top += height + space
        }
    }
    return MosaicLayouts.Level(rows, cells, starts, rowOf, (top - space).coerceAtLeast(0f), rows.maxOf { it.height }, unit, space)
}

/**
 * Reels, a gallery's one column: every card gets a page of the gallery's [page] height, stacked with no space between,
 * reaching [bleed] past each side of the grid to the screen's edges. The card is drawn inside its page at its own shape
 * ([MosaicLayouts.reelContent]).
 */
internal fun reelLevel(cards: List<Card>, width: Float, page: Float, bleed: Float): MosaicLayouts.Level {
    if (cards.isEmpty() || !width.isFinite() || width <= 0f || !page.isFinite() || page <= 0f) {
        return MosaicLayouts.Level(emptyList(), emptyList(), emptyList(), IntArray(0), 0f, 0f, 0f, 0f)
    }
    val across = width + 2 * bleed
    val cells = cards.indices.map { Rect(-bleed, it * page, width + bleed, (it + 1) * page) }
    val rows = cards.map { MosaicRow(listOf(it), listOf(across), page) }
    return MosaicLayouts.Level(rows, cells, cards.indices.toList(), IntArray(cards.size) { it }, cards.size * page, page, across, 0f)
}

/** Readable proportions for the one-column gallery: the picture's own, else one that suits the card's text. */
internal fun mosaicAspectRatio(card: Card): Float {
    if (galleryKeepsText(card) && galleryPicture(card) != null) return 0.9f
    storedRatio(card)?.let { return it.coerceIn(0.65f, 1.8f) }
    return when (tileKind(card)) {
        TileKind.Note, TileKind.Audio -> 0.95f
        TileKind.Post -> 1.05f
        TileKind.Repository -> 1.3f
        else -> 1.15f
    }
}

/** The gap [mosaicLevel] leaves between cards: [gap], kept to a sixteenth of the [width]. */
private fun mosaicGap(width: Float, gap: Float): Float = gap.takeIf { it.isFinite() }?.coerceIn(0f, width / 16f) ?: 0f

/**
 * The space between tiles at [columns] across a gallery [width] wide, both in dp: about a twentieth of a column, from 6
 * to 24 dp, so a denser grid sits tighter, as the web showcase spaces its wall (`wallGap` in octa-ui's
 * `apps/showcase`). On a phone that is 16 dp at one column, 8 at two and 6 at three and four.
 */
internal fun mosaicGapDp(columns: Int, width: Float): Float =
    (width / columns.coerceAtLeast(1) * 0.045f).roundToInt().coerceIn(6, 24).toFloat()

/**
 * Cached canonical geometry, with the cards shaped and drawn by [tiles]. Only the visible window is interpolated during a
 * pointer frame. [gaps] holds the space between cards at one to four columns. With a [page] height, one column is the
 * reels ([reelLevel]), each page reaching [bleed] past the grid's sides.
 */
internal class MosaicLayouts(
    val cards: List<Card>, val width: Float, gaps: List<Float>, val tiles: MosaicTiles = GalleryMosaicTiles,
    val page: Float = 0f, val bleed: Float = 0f,
) {
    /** The same [gap] at every density. */
    constructor(cards: List<Card>, width: Float, gap: Float, tiles: MosaicTiles = GalleryMosaicTiles) : this(cards, width, List(4) { gap }, tiles)

    /**
     * One density's grid: [unit] is one square's width (the whole width at one column) and [gap] the space between
     * cards.
     */
    internal class Level(
        val rows: List<MosaicRow>, val cells: List<Rect>, val starts: List<Int>, val rowOf: IntArray, val height: Float,
        val tallest: Float, val unit: Float, val gap: Float,
    )
    val byId = cards.mapIndexed { index, card -> card.id to index }.toMap()
    private val grids = (1..4).map { columns -> mosaicLevel(cards, width, columns, gaps.getOrElse(columns - 1) { 0f }, tiles) }
    private val reels = if (page > 0f) reelLevel(cards, width, page, bleed) else grids[0]
    private val pictures = BooleanArray(cards.size) { tiles.isPicture(cards[it]) }

    /** Width over height of each card drawn in its page: a picture at its own shape, words at their one-column shape. */
    private val reelRatios = FloatArray(cards.size) { index ->
        val cell = grids[0].cells[index]
        if (pictures[index]) tiles.reelRatio(cards[index]) else cell.width / cell.height
    }

    /** Whether card [index]'s tile is measured to its cell at every size ([MosaicTiles.isPicture]). */
    fun isPicture(index: Int): Boolean = pictures[index]

    /** The grid at [columns] across, or the reels at one column when the layouts have a [page]. */
    fun level(columns: Int): Level = if (isReels(columns)) reels else grids[columns.coerceIn(1, 4) - 1]

    /** True when [columns] across is the reels: one column, with a [page] height. */
    fun isReels(columns: Int): Boolean = page > 0f && columns <= 1

    /**
     * Where card [index] is drawn inside [cell], a rectangle on its way from the card's one-column cell to its page
     * [reel] of the way into reels: the largest rectangle of a shape between its one-column shape and its own that
     * fits, centred, so a picture shows whole and words keep their lines.
     */
    fun reelContent(index: Int, cell: Rect, reel: Float): Rect {
        if (reel <= 0f || cell.width <= 0f || cell.height <= 0f) return cell
        val one = grids[0].cells[index]
        val ratio = one.width / one.height + (reelRatios[index] - one.width / one.height) * reel.coerceAtMost(1f)
        val width = minOf(cell.width, cell.height * ratio)
        val height = width / ratio
        return Rect(cell.center.x - width / 2f, cell.center.y - height / 2f, cell.center.x + width / 2f, cell.center.y + height / 2f)
    }

    /** The space between cards at three columns. */
    private val wordsGap = mosaicGap(width, gaps.getOrElse(2) { 0f })

    /** One square's width at three columns: a tile with words at one column is laid out this wide. */
    private val wordsWidth = (width - 2 * wordsGap) / 3

    /**
     * The size a tile with words is laid out at while the grid of [columns] is shown: its cell there, divided by how much
     * larger the tiles show their words at that size ([MosaicTiles.wordsScale]), or [wordsWidth] wide in the cell's shape
     * at one column. The tile is drawn scaled to its cell, so every tile of one grid shows its words at the same size, a
     * big tile's the size of its neighbours', and a pinch between two grids cross-fades the tile's layouts in both rather
     * than rewrapping it under the fingers.
     */
    fun wordsFrame(index: Int, columns: Int): Size {
        // A page draws a tile with words at its one-column shape ([reelContent]), so it keeps its one-column frame.
        val level = grids[columns.coerceIn(1, 4) - 1]
        val cell = level.cells[index]
        if (level.unit >= width || level.unit <= 0f) return Size(wordsWidth, wordsWidth * cell.height / cell.width)
        val scale = tiles.wordsScale(columns).takeIf { it > 0f && it.isFinite() } ?: 1f
        return Size(cell.width / scale, cell.height / scale)
    }

/** Projects a whole canonical grid around one shared focal rectangle; row neighbours never cross. */
    fun projection(columns: Int, position: Float, focalIndex: Int): MosaicProjection {
        val base = level(columns).cells[focalIndex]
        val focal = cell(focalIndex, position)
        val scale = focal.width / base.width
        return MosaicProjection(this, columns, scale, Offset(focal.left - base.left * scale, focal.top - base.top * scale))
    }

    fun cell(index: Int, position: Float): Rect {
        val p = position.coerceIn(1f, 4f)
        val a = level(floor(p).toInt()).cells[index]
        val b = level(ceil(p).toInt()).cells[index]
        val t = p - floor(p)
        return Rect(a.left + (b.left - a.left) * t, a.top + (b.top - a.top) * t,
            a.right + (b.right - a.right) * t, a.bottom + (b.bottom - a.bottom) * t)
    }

    fun height(position: Float): Float {
        val p = position.coerceIn(1f, 4f)
        val a = level(floor(p).toInt()).height
        return a + (level(ceil(p).toInt()).height - a) * (p - floor(p))
    }

    fun slotHeight(row: Int, columns: Int, position: Float): Float {
        val starts = level(columns).starts
        val top = cell(starts[row], position).top
        return ((starts.getOrNull(row + 1)?.let { cell(it, position).top } ?: height(position)) - top).coerceAtLeast(0f)
    }

    /** Binary search keeps lookup O(log n); overscan covers a whole neighbouring brick at either density. */
    fun window(position: Float, origin: Float, viewport: Float): IntRange {
        if (level(1).cells.isEmpty()) return IntRange.EMPTY
        val p = position.coerceIn(1f, 4f)
        val margin = maxOf(level(floor(p).toInt()).tallest, level(ceil(p).toInt()).tallest)
        fun lowerBound(y: Float): Int {
            var low = 0
            var high = cards.size
            while (low < high) {
                val middle = (low + high) ushr 1
                if (cell(middle, position).top < y) low = middle + 1 else high = middle
            }
            return low
        }
        return lowerBound(origin - margin)..(lowerBound(origin + viewport + margin) - 1).coerceAtMost(cards.lastIndex)
    }
}

/** A rigid grid projection. Its inverse transform allows a bounded canonical viewport lookup. */
internal class MosaicProjection(val layouts: MosaicLayouts, val columns: Int, val scale: Float, val offset: Offset) {
    fun cell(index: Int): Rect {
        val base = layouts.level(columns).cells[index]
        return Rect(base.left * scale + offset.x, base.top * scale + offset.y,
            base.right * scale + offset.x, base.bottom * scale + offset.y)
    }
    fun window(origin: Float, viewport: Float): IntRange =
        layouts.window(columns.toFloat(), (origin - offset.y) / scale, viewport / scale)
}

/**
 * Saveable density with continuous, reversible finger tracking between canonical brick layouts. With [reels], one
 * column is full-screen reels: one card per page.
 */
@Stable
class MosaicState(initialColumns: Int = 2, val reels: Boolean = false) {
    var columns by mutableIntStateOf(initialColumns.coerceIn(1, 4))
        private set
    internal var isPinching by mutableStateOf(false)
        private set
    internal var position by mutableFloatStateOf(initialColumns.coerceIn(1, 4).toFloat())
    internal var referenceColumns by mutableIntStateOf(initialColumns.coerceIn(1, 4))

    /**
     * The density the tiles decode their pictures for. It follows [columns] only once the gallery has come to rest, so
     * letting go never asks every picture again while the cards are still moving.
     */
    internal var pictureColumns by mutableIntStateOf(initialColumns.coerceIn(1, 4))

    /** The stop the last pinch or step started from: the grid the cards move away from (see [between]). */
    internal var home by mutableIntStateOf(initialColumns.coerceIn(1, 4))
        private set

    /** How far the gallery has gone into reels: 0 at two columns or more, 1 at one column when it has [reels]. */
    internal val reel: Float get() = if (reels) (2f - position).coerceIn(0f, 1f) else 0f

    /** True while the gallery rests on, or is landing on, its reels. */
    internal val inReels: Boolean get() = reels && columns == 1

    /** Counts the hops a pinch has started, so the gallery eases each one with [advanceHop]. */
    internal var hops by mutableIntStateOf(0)
        private set

    /** A hop is still easing the cards to where the fingers are. */
    internal val hopping: Boolean get() = hopGap != 0f
    private var startFinger = position
    private var finger = position
    private var stop = columns
    private var hopGap = 0f
    private var hopDone = 0f
    private var direction = 0
    private var gestureScale = 1f

    /** Makes cards larger; returns false at the largest size. */
    fun larger(): Boolean = changeColumns(columns - 1)

    /** Fits more cards across; returns false at the smallest size. */
    fun smaller(): Boolean = changeColumns(columns + 1)

    private fun changeColumns(value: Int): Boolean {
        val next = value.coerceIn(1, 4)
        if (next == columns) return false
        home = columns
        columns = next
        position = next.toFloat()
        return true
    }

    internal fun beginPinch() {
        isPinching = true
        if (abs(position - position.roundToInt()) < 1e-3f) home = position.roundToInt()
        finger = fingerPosition(position)
        startFinger = finger
        stop = finger.roundToInt()
        // A pinch that interrupts a landing between the follow ranges eases from the geometry on screen.
        startHop(position - steppedPosition(finger))
        direction = 0
        gestureScale = 1f
    }

    internal fun zoomPinch(factor: Float) {
        if (!factor.isFinite() || factor <= 0f) return
        // Discard overshoot at a limit so the very next reversed pointer delta changes the geometry.
        gestureScale = (gestureScale * factor).coerceIn(exp((startFinger - 4f) * PINCH_STEP), exp((startFinger - 1f) * PINCH_STEP))
        val next = (startFinger - ln(gestureScale) / PINCH_STEP).coerceIn(1f, 4f)
        if (abs(next - finger) > 1e-4f) direction = if (next > finger) 1 else -1
        finger = next
        if (finger.roundToInt() != stop) {
            stop = finger.roundToInt()
            startHop(position - steppedPosition(finger))
        }
        position = steppedPosition(finger) + hopGap * (1f - hopEase(hopDone))
    }

    /** Eases the current hop to [fraction] of its length; the cards land where the fingers are at 1. */
    internal fun advanceHop(fraction: Float) {
        if (!isPinching || hopGap == 0f) return
        hopDone = fraction.coerceIn(0f, 1f)
        position = steppedPosition(finger) + hopGap * (1f - hopEase(hopDone))
        if (hopDone >= 1f) hopGap = 0f
    }

    private fun startHop(gap: Float) {
        hopGap = if (abs(gap) < 1e-4f) 0f else gap
        hopDone = 0f
        if (hopGap != 0f) hops++
    }

    internal fun finishPinch() {
        val before = columns
        columns = releaseColumns(finger, direction)
        if (columns != before) Track.count("pinch", detail = "$columns")
        isPinching = false
        hopGap = 0f
    }

    /** Ends a pinch the gallery lost, on the stop nearest where it is; a landing already under way keeps its stop. */
    internal fun cancelPinch() {
        if (isPinching) columns = position.roundToInt().coerceIn(1, 4)
        isPinching = false
        hopGap = 0f
        gestureScale = 1f
    }

    internal companion object {
        val Saver: Saver<MosaicState, Int> = saver(reels = false)

        /** Keeps the nearest stop; the state comes back with or without [reels]. */
        fun saver(reels: Boolean): Saver<MosaicState, Int> =
            Saver(save = { it.position.roundToInt().coerceIn(1, 4) }, restore = { MosaicState(it, reels) })
    }
}

/**
 * The two grids a [MosaicState.position] lies between: [from] on the side of the gesture's home stop, [to] the other,
 * and [progress] from 0 on [from] to 1 on [to].
 */
internal class MosaicBetween(val from: Int, val to: Int, val progress: Float)

/** Where the gallery is between its two nearest stops, seen from the stop the last gesture started at. */
internal fun MosaicState.between(): MosaicBetween {
    val p = position.coerceIn(1f, 4f)
    val low = floor(p).toInt()
    val high = minOf(4, low + 1)
    return if (home <= low) MosaicBetween(low, high, p - low) else MosaicBetween(high, low, 1f - (p - low))
}

/** Pinch scale, as a natural log, that moves the gallery one stop: about 30% at every size (Tavi's canvas, 28 Oct 2026). */
private val PINCH_STEP = ln(1.3f)

/** Share of the finger travel the cards follow; the rest is a quick hop when the fingers pass halfway between stops. */
private const val FOLLOW = 0.6f
private const val FILL_EDGE = 0.2f

/**
 * The density drawn for a finger position: the nearest stop plus [FOLLOW] of the way the fingers have moved from it, so
 * the cards move with the fingers from the first moment and hop the rest once the fingers pass halfway.
 */
internal fun steppedPosition(finger: Float): Float {
    val f = finger.coerceIn(1f, 4f)
    val stop = f.roundToInt()
    return stop + (f - stop) * FOLLOW
}

/**
 * How strongly the blurred fill behind a pinch shows at [progress] of a step between two stops: fully while the cards
 * are on their way, fading in from the stop the step starts at and out toward the one it ends at, so the fill never
 * appears or disappears in one frame.
 */
internal fun mosaicFillShown(progress: Float): Float {
    val edge = (minOf(progress, 1f - progress) / FILL_EDGE).coerceIn(0f, 1f)
    return edge * edge * (3f - 2f * edge)
}

/** The finger position whose [steppedPosition] is nearest [position], so a pinch that interrupts a landing starts there. */
internal fun fingerPosition(position: Float): Float {
    val p = position.coerceIn(1f, 4f)
    val stop = p.roundToInt()
    return (stop + (p - stop) / FOLLOW).coerceIn(stop - 0.5f, stop + 0.5f).coerceIn(1f, 4f)
}

/** Where a released pinch lands: the way the fingers were moving, once they are a quarter of the way to the next stop. */
internal fun releaseColumns(finger: Float, direction: Int): Int = when {
    direction < 0 -> floor(finger + 0.25f).toInt()
    direction > 0 -> ceil(finger - 0.25f).toInt()
    else -> finger.roundToInt()
}.coerceIn(1, 4)

/**
 * How long a landing of [way] stops takes: 250 ms for a whole stop, the pace of one Larger or Smaller step, and longer
 * than its share for a short way, so the last fifth of a stop after a hop eases in about 100 ms instead of snapping.
 */
internal fun settleMillis(way: Float): Float = 250f * sqrt(abs(way))

/**
 * How far a landing has come at [fraction] of its time: a cubic that starts at [lead] times its average speed and comes
 * to rest, so the cards keep the speed they had when the fingers let go instead of jumping to the landing's own. A
 * [lead] of 0 starts from still; 3, the most it takes, is the plain ease-out. It never overshoots.
 */
internal fun settleProgress(fraction: Float, lead: Float): Float {
    val t = fraction.coerceIn(0f, 1f)
    val m = lead.coerceIn(0f, 3f)
    return m * t * (1f - t) * (1f - t) + t * t * (3f - 2f * t)
}

/**
 * The [settleProgress] lead of a landing [way] stops long that takes [millis] and starts while the density moves at
 * [speed] stops per millisecond: 0 when the cards were still or moving away from where they land.
 */
internal fun settleLead(speed: Float, way: Float, millis: Float): Float =
    if (way == 0f || millis <= 0f) 0f else (speed * millis / way).coerceIn(0f, 3f)

/** The density's speed as a pinch ends, in stops per millisecond, measured over about its last [SPEED_WINDOW_MILLIS]. */
internal class MosaicSpeed {
    private val times = ArrayDeque<Long>()
    private val positions = ArrayDeque<Float>()

    fun clear() {
        times.clear()
        positions.clear()
    }

    /** Records that the density was at [position] at [time], in milliseconds. */
    fun add(time: Long, position: Float) {
        times.addLast(time)
        positions.addLast(position)
        while (times.size > 1 && time - times.first() > 4 * SPEED_WINDOW_MILLIS) {
            times.removeFirst()
            positions.removeFirst()
        }
    }

    /**
     * The speed that brought the density to [position] at [time]: from the first sample in the window before it, or the
     * last sample when none is that recent, so fingers held still before letting go give 0.
     */
    fun at(time: Long, position: Float): Float {
        val index = times.indexOfFirst { time - it <= SPEED_WINDOW_MILLIS }.takeIf { it >= 0 } ?: times.lastIndex
        if (index < 0 || time <= times[index]) return 0f
        return (position - positions[index]) / (time - times[index])
    }
}

private const val SPEED_WINDOW_MILLIS = 50L

/** The ease-out curve of a hop: fast away, gentle landing. */
private fun hopEase(fraction: Float): Float {
    val rest = 1f - fraction.coerceIn(0f, 1f)
    return 1f - rest * rest * rest
}

/**
 * Keeps the selected gallery density across tab changes and activity recreation.
 *
 * @param reels Whether one column is full-screen reels.
 * @return The gallery's density.
 */
@Composable
fun rememberMosaicState(reels: Boolean = false): MosaicState =
    rememberSaveable(saver = MosaicState.saver(reels)) { MosaicState(reels = reels) }

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
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.exp
import kotlin.math.floor
import kotlin.math.ln
import kotlin.math.roundToInt

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
 * The card's shape in the grid, as Google Photos breaks up its squares: screenshots and other tall pictures stand two
 * squares high, about half the wide pictures lie two across, and about one picture in seven is shown big, two by two.
 * Text, posts and pictures with words stay one square. The choice is fixed by the card's id, so a card keeps its shape
 * at every density and across reloads.
 */
internal fun mosaicSpan(card: Card): MosaicSpan {
    if (galleryKeepsText(card) || galleryPicture(card) == null) return MosaicSpan(1, 1)
    val ratio = storedRatio(card) ?: return MosaicSpan(1, 1)
    val pick = card.id.hashCode().let { (it xor (it ushr 16)) and Int.MAX_VALUE }
    return when {
        ratio <= TALL_RATIO -> MosaicSpan(1, 2)
        ratio >= WIDE_RATIO && pick % 2 == 0 -> MosaicSpan(2, 1)
        pick % 7 == 3 -> MosaicSpan(2, 2)
        else -> MosaicSpan(1, 1)
    }
}

/** Width over height of the card's stored picture, or null when it has no usable size. */
private fun storedRatio(card: Card): Float? {
    val width = card.cover?.width ?: card.blob?.width
    val height = card.cover?.height ?: card.blob?.height
    return if (width != null && height != null && width > 0 && height > 0) width.toFloat() / height else null
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
    if (cards.isEmpty() || !width.isFinite() || width <= 0f) return MosaicLayouts.Level(emptyList(), emptyList(), emptyList(), IntArray(0), 0f, 0f)
    val across = columns.coerceIn(1, 4)
    val space = mosaicGap(width, gap)
    val cells = ArrayList<Rect>(cards.size)
    val rows = ArrayList<MosaicRow>()
    val starts = ArrayList<Int>()
    val rowOf = IntArray(cards.size)
    var top = 0f
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
        val unit = (width - space * (across - 1)) / across
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
    return MosaicLayouts.Level(rows, cells, starts, rowOf, (top - space).coerceAtLeast(0f), rows.maxOf { it.height })
}

/** Readable proportions for the one-column gallery: the picture's own, else one that suits the card's text. */
internal fun mosaicAspectRatio(card: Card): Float {
    if (galleryKeepsText(card) && galleryPicture(card) != null) return 0.9f
    storedRatio(card)?.let { return it.coerceIn(0.65f, 1.8f) }
    return when (tileKind(card)) {
        TileKind.Note -> 0.95f
        TileKind.Post -> 1.05f
        TileKind.Repository -> 1.3f
        else -> 1.15f
    }
}

/** The gap [mosaicLevel] leaves between cards: [gap], kept to a sixteenth of the [width]. */
private fun mosaicGap(width: Float, gap: Float): Float = gap.takeIf { it.isFinite() }?.coerceIn(0f, width / 16f) ?: 0f

/**
 * Cached canonical geometry, with the cards shaped and drawn by [tiles]. Only the visible window is interpolated during a
 * pointer frame.
 */
internal class MosaicLayouts(val cards: List<Card>, val width: Float, gap: Float, val tiles: MosaicTiles = GalleryMosaicTiles) {
    internal class Level(val rows: List<MosaicRow>, val cells: List<Rect>, val starts: List<Int>, val rowOf: IntArray, val height: Float, val tallest: Float)
    val byId = cards.mapIndexed { index, card -> card.id to index }.toMap()
    val levels = (1..4).map { columns -> mosaicLevel(cards, width, columns, gap, tiles) }
    private val pictures = BooleanArray(cards.size) { tiles.isPicture(cards[it]) }

    /** Whether card [index]'s tile is measured to its cell at every size ([MosaicTiles.isPicture]). */
    fun isPicture(index: Int): Boolean = pictures[index]

    fun level(columns: Int): Level = levels[columns.coerceIn(1, 4) - 1]

    /** One square's width at three columns: every tile with words is laid out this wide, at every density. */
    private val wordsWidth = (width - 2 * mosaicGap(width, gap)) / 3

    /**
     * The size a tile with words is laid out at while the grid of [columns] is shown: [wordsWidth] wide, in the shape
     * of its cell there. The tile is then drawn scaled to its cell, a picture of itself, so its words keep the same lines
     * at every density and between them.
     */
    fun wordsFrame(index: Int, columns: Int): Size {
        val cell = level(columns).cells[index]
        return Size(wordsWidth, wordsWidth * cell.height / cell.width)
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
        val margin = maxOf(level(floor(position).toInt()).tallest, level(ceil(position).toInt()).tallest)
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

/** Saveable density with continuous, reversible finger tracking between canonical brick layouts. */
@Stable
class MosaicState(initialColumns: Int = 2) {
    var columns by mutableIntStateOf(initialColumns.coerceIn(1, 4))
        private set
    internal var isPinching by mutableStateOf(false)
        private set
    internal var position by mutableFloatStateOf(initialColumns.coerceIn(1, 4).toFloat())
    internal var referenceColumns by mutableIntStateOf(initialColumns.coerceIn(1, 4))

    /** The stop the last pinch or step started from: the grid the cards move away from (see [between]). */
    internal var home by mutableIntStateOf(initialColumns.coerceIn(1, 4))
        private set

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
        columns = releaseColumns(finger, direction)
        isPinching = false
        hopGap = 0f
    }

    internal fun cancelPinch() {
        isPinching = false
        columns = position.roundToInt().coerceIn(1, 4)
        hopGap = 0f
        gestureScale = 1f
    }

    internal companion object {
        val Saver: Saver<MosaicState, Int> = Saver(save = { it.position.roundToInt().coerceIn(1, 4) }, restore = { MosaicState(it) })
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

/** The ease-out curve of a hop: fast away, gentle landing. */
private fun hopEase(fraction: Float): Float {
    val rest = 1f - fraction.coerceIn(0f, 1f)
    return 1f - rest * rest * rest
}

/** Keeps the selected gallery density across tab changes and activity recreation. */
@Composable
fun rememberMosaicState(): MosaicState = rememberSaveable(saver = MosaicState.Saver) { MosaicState() }

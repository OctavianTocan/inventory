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
import kotlin.math.ceil
import kotlin.math.floor
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
internal fun mosaicRows(cards: List<Card>, width: Float, columns: Int, gap: Float): List<MosaicRow> =
    mosaicLevel(cards, width, columns, gap).rows

/**
 * Lays the cards out at one density without reordering or dropping any. One column shows each card across the whole
 * width at its own proportions. Two to four columns share square cells in bands of one or two rows, filled row by row
 * in card order: a card two squares high starts a two-row band, and a card that does not fit where it falls takes one
 * square there, so a band is never left with a hole before the last card. Card tops never decrease with the order.
 */
internal fun mosaicLevel(cards: List<Card>, width: Float, columns: Int, gap: Float): MosaicLayouts.Level {
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
            val height = width / mosaicAspectRatio(card)
            starts += index
            rowOf[index] = index
            cells += Rect(0f, top, width, top + height)
            rows += MosaicRow(listOf(card), listOf(width), height)
            top += height + space
        }
    } else {
        val unit = (width - space * (across - 1)) / across
        val step = unit + space
        val spans = cards.map(::mosaicSpan)
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
private fun mosaicAspectRatio(card: Card): Float {
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

/** Cached canonical geometry. Only the visible window is interpolated during a pointer frame. */
internal class MosaicLayouts(val cards: List<Card>, val width: Float, gap: Float) {
    internal class Level(val rows: List<MosaicRow>, val cells: List<Rect>, val starts: List<Int>, val rowOf: IntArray, val height: Float, val tallest: Float)
    val byId = cards.mapIndexed { index, card -> card.id to index }.toMap()
    val levels = (1..4).map { columns -> mosaicLevel(cards, width, columns, gap) }

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
    private var startPosition = position
    private var gestureScale = 1f

    /** Makes cards larger; returns false at the largest size. */
    fun larger(): Boolean = changeColumns(columns - 1)

    /** Fits more cards across; returns false at the smallest size. */
    fun smaller(): Boolean = changeColumns(columns + 1)

    private fun changeColumns(value: Int): Boolean {
        val next = value.coerceIn(1, 4)
        if (next == columns) return false
        columns = next
        position = next.toFloat()
        return true
    }

    internal fun beginPinch() {
        isPinching = true
        startPosition = fingerPosition(position)
        gestureScale = 1f
    }

    internal fun zoomPinch(factor: Float) {
        if (!factor.isFinite() || factor <= 0f) return
        // Discard overshoot at a limit so the very next reversed pointer delta changes the geometry.
        gestureScale = (gestureScale * factor).coerceIn(startPosition / 4f, startPosition)
        position = heldPosition(startPosition / gestureScale)
    }

    internal fun finishPinch() {
        columns = position.roundToInt().coerceIn(1, 4)
        isPinching = false
    }

    internal fun cancelPinch() {
        isPinching = false
        columns = position.roundToInt().coerceIn(1, 4)
        gestureScale = 1f
    }

    internal companion object {
        val Saver: Saver<MosaicState, Int> = Saver(save = { it.position.roundToInt().coerceIn(1, 4) }, restore = { MosaicState(it) })
    }
}

/** Share of each step between stops that the fingers travel before the next denser grid takes over. */
private const val STOP_HOLD = 0.1f

/** Maps finger travel to a density, holding each stop briefly so the sparser grid is never shown below its own size. */
internal fun heldPosition(finger: Float): Float {
    val raw = finger.coerceIn(1f, 4f)
    val stop = floor(raw)
    return stop + ((raw - stop - STOP_HOLD) / (1f - STOP_HOLD)).coerceAtLeast(0f)
}

/** The inverse of [heldPosition], so a pinch that interrupts a settle starts from the geometry on screen. */
internal fun fingerPosition(position: Float): Float {
    val held = position.coerceIn(1f, 4f)
    val stop = floor(held)
    return if (held == stop) stop else stop + STOP_HOLD + (held - stop) * (1f - STOP_HOLD)
}

/** The grid drawn at [position]: the denser neighbour between stops, which is enlarged and so always fills the width. */
internal fun pinchScene(position: Float): Int = ceil(position.coerceIn(1f, 4f)).toInt().coerceIn(1, 4)

/** Keeps the selected gallery density across tab changes and activity recreation. */
@Composable
fun rememberMosaicState(): MosaicState = rememberSaveable(saver = MosaicState.Saver) { MosaicState() }

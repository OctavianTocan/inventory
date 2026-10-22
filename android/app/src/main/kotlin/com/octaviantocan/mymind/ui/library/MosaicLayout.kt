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
import androidx.compose.ui.geometry.Offset
import com.octaviantocan.mymind.data.Card
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.min
import kotlin.math.roundToInt

/** One justified row, in pixels, in the server's original card order. */
internal data class MosaicRow(val cards: List<Card>, val widths: List<Float>, val height: Float) {
    val key: String get() = cards.first().id
}

/** Packs variable-width bricks around the selected tile size without reordering or dropping cards. */
internal fun mosaicRows(cards: List<Card>, width: Float, columns: Int, gap: Float): List<MosaicRow> {
    if (cards.isEmpty() || !width.isFinite() || width <= 0f) return emptyList()
    val across = columns.coerceIn(1, 4)
    val space = gap.takeIf { it.isFinite() }?.coerceIn(0f, width / 16f) ?: 0f
    val ratios = cards.map(::mosaicAspectRatio)
    val rows = ArrayList<MosaicRow>()
    var start = 0
    while (start < cards.size) {
        val count = min(across, cards.size - start)
        val sum = (start until start + count).sumOf { ratios[it].toDouble() }.toFloat()
        val justified = (width - space * (count - 1)) / sum
        val height = if (count < across) min(justified, (width - space * (across - 1)) / (across * sum / count)) else justified
        rows += MosaicRow(cards.subList(start, start + count), (start until start + count).map { ratios[it] * height }, height)
        start += count
    }
    return rows
}

/** Uses cover proportions where possible, and readable proportions for text cards. */
private fun mosaicAspectRatio(card: Card): Float {
    if (galleryKeepsText(card) && galleryPicture(card) != null) return 0.9f
    val width = card.cover?.width ?: card.blob?.width
    val height = card.cover?.height ?: card.blob?.height
    if (width != null && height != null && width > 0 && height > 0) return (width.toFloat() / height).coerceIn(0.65f, 1.8f)
    return when (tileKind(card)) {
        TileKind.Note -> 0.95f
        TileKind.Post -> 1.05f
        TileKind.Repository -> 1.3f
        else -> 1.15f
    }
}

/** Cached canonical geometry. Only the visible window is interpolated during a pointer frame. */
internal class MosaicLayouts(val cards: List<Card>, val width: Float, gap: Float) {
    internal class Level(val rows: List<MosaicRow>, val cells: List<Rect>, val starts: List<Int>, val rowOf: IntArray, val height: Float, val tallest: Float)
    val byId = cards.mapIndexed { index, card -> card.id to index }.toMap()
    val levels = (1..4).map { columns ->
        val rows = mosaicRows(cards, width, columns, gap)
        val cells = ArrayList<Rect>(cards.size)
        val starts = ArrayList<Int>(rows.size)
        val rowOf = IntArray(cards.size)
        var y = 0f
        rows.forEachIndexed { rowIndex, row ->
            starts += cells.size
            var x = 0f
            row.widths.forEach { w ->
                rowOf[cells.size] = rowIndex
                cells += Rect(x, y, x + w, y + row.height)
                x += w + gap
            }
            y += row.height + gap
        }
        Level(rows, cells, starts, rowOf, (y - gap).coerceAtLeast(0f), rows.maxOfOrNull { it.height } ?: 0f)
    }

    fun level(columns: Int): Level = levels[columns.coerceIn(1, 4) - 1]

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

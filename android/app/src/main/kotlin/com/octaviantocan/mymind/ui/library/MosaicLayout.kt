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
import com.octaviantocan.mymind.data.Card
import kotlin.math.abs
import kotlin.math.min

/** One justified row, in pixels, in the server's original card order. */
internal data class MosaicRow(val cards: List<Card>, val widths: List<Float>, val height: Float) {
    val key: String get() = cards.first().id
}

/** Packs variable-width bricks around the selected tile size without reordering or dropping cards. */
internal fun mosaicRows(cards: List<Card>, width: Float, columns: Int, gap: Float): List<MosaicRow> {
    if (cards.isEmpty() || !width.isFinite() || width <= 0f) return emptyList()
    val across = columns.coerceIn(1, 4)
    val space = gap.takeIf { it.isFinite() }?.coerceIn(0f, width / 16f) ?: 0f
    val target = width / across
    val ratios = cards.map(::mosaicAspectRatio)
    val rows = ArrayList<MosaicRow>()
    var start = 0
    while (start < cards.size) {
        var count = 1
        var sum = ratios[start]
        if (across > 1) {
            while (start + count < cards.size && count < across + 2) {
                val height = (width - space * (count - 1)) / sum
                val nextSum = sum + ratios[start + count]
                val nextHeight = (width - space * count) / nextSum
                if (abs(nextHeight - target) >= abs(height - target)) break
                count++
                sum = nextSum
            }
        }
        val justified = (width - space * (count - 1)) / sum
        val height = if (start + count == cards.size && count < across) min(justified, target) else justified
        rows += MosaicRow(cards.subList(start, start + count), (start until start + count).map { ratios[it] * height }, height)
        start += count
    }
    return rows
}

/** Uses cover proportions where possible, and readable proportions for text cards. */
private fun mosaicAspectRatio(card: Card): Float {
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

/** Saveable density and the short-lived visual response to a two-finger pinch. */
@Stable
class MosaicState(initialColumns: Int = 2) {
    var columns by mutableIntStateOf(initialColumns.coerceIn(1, 4))
        private set
    internal var isPinching by mutableStateOf(false)
        private set
    internal var previewScale by mutableFloatStateOf(1f)
        private set
    internal var focalX by mutableFloatStateOf(0.5f)
    private var gestureScale = 1f

    /** Makes cards larger; returns false at the largest size. */
    fun larger(): Boolean = changeColumns(columns - 1)

    /** Fits more cards across; returns false at the smallest size. */
    fun smaller(): Boolean = changeColumns(columns + 1)

    private fun changeColumns(value: Int): Boolean {
        val next = value.coerceIn(1, 4)
        if (next == columns) return false
        columns = next
        return true
    }

    internal fun beginPinch() {
        isPinching = true
        gestureScale = 1f
        previewScale = 1f
    }

    internal fun zoomPinch(factor: Float) {
        if (!factor.isFinite() || factor <= 0f) return
        gestureScale = (gestureScale * factor).coerceIn(0.25f, 4f)
        previewScale = gestureScale.coerceIn(0.82f, 1.2f)
    }

    internal fun finishPinch() {
        if (gestureScale < 0.88f) smaller() else if (gestureScale > 1.12f) larger()
        cancelPinch()
    }

    internal fun cancelPinch() {
        isPinching = false
        previewScale = 1f
        gestureScale = 1f
    }

    internal companion object {
        val Saver: Saver<MosaicState, Int> = Saver(save = { it.columns }, restore = { MosaicState(it) })
    }
}

/** Keeps the selected gallery density across tab changes and activity recreation. */
@Composable
fun rememberMosaicState(): MosaicState = rememberSaveable(saver = MosaicState.Saver) { MosaicState() }

package com.octaviantocan.mymind.ui.library

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card

/**
 * What a [LibraryMosaic] draws for each of its cards and the shape it gives them. [GalleryMosaicTiles] draws library
 * cards; another page, such as the Spaces list, draws its own items in the same pinchable grid.
 */
@Stable
internal interface MosaicTiles {
    /** The card's shape in the grid at two columns or more. */
    fun span(card: Card): MosaicSpan

    /** Width over height of the card's cell at one column. */
    fun aspectRatio(card: Card): Float

    /**
     * True when the tile is measured to its cell at every size; false when it is laid out once and drawn scaled, as a
     * tile with words is ([MosaicLayouts.wordsFrame]).
     */
    fun isPicture(card: Card): Boolean

    /**
     * The radius the mosaic rounds every tile's corners to, the same on screen at every density and while a tile with
     * words is drawn scaled; 0 when the tiles draw their own edges.
     */
    val corner: Dp

    /**
     * The card's own drawing, which takes its touches and semantics.
     *
     * @param card The card.
     * @param onClick Called when the tile is tapped.
     * @param onLongClick Called when the tile is held, or null when holding does nothing.
     * @param enabled False while a pinch moves the tile, so it takes no touches.
     * @param imageRequestSizePx Size, in pixels, to decode the tile's pictures at.
     * @param modifier Modifier for the tile, sized to its cell.
     */
    @Composable
    fun Tile(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, enabled: Boolean, imageRequestSizePx: Int, modifier: Modifier)

    /**
     * A drawing of the card that takes no touches and plays nothing: its copy while it changes line, or its layout in
     * the other grid. The mosaic clears its semantics.
     *
     * @param card The card.
     * @param imageRequestSizePx Size, in pixels, to decode the tile's pictures at.
     */
    @Composable
    fun Copy(card: Card, imageRequestSizePx: Int)

    /**
     * The card in the blurred fill behind a pinch, which only has to cover the paper between moving tiles.
     *
     * @param card The card.
     * @param imageRequestSizePx Size, in pixels, to decode the tile's pictures at.
     */
    @Composable
    fun Backdrop(card: Card, imageRequestSizePx: Int)
}

/** The library's own tiles ([GalleryTile]), shaped the way Google Photos breaks up its squares ([mosaicSpan]). */
internal object GalleryMosaicTiles : MosaicTiles {
    override fun span(card: Card): MosaicSpan = mosaicSpan(card)

    override fun aspectRatio(card: Card): Float = mosaicAspectRatio(card)

    override fun isPicture(card: Card): Boolean = galleryTileIsPicture(card)

    override val corner: Dp = TILE_CORNER

    @Composable
    override fun Tile(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, enabled: Boolean, imageRequestSizePx: Int, modifier: Modifier) =
        GalleryTile(card, onClick, modifier = modifier, enabled = enabled, onLongClick = onLongClick, imageRequestSizePx = imageRequestSizePx,
            corner = 0.dp)

    @Composable
    override fun Copy(card: Card, imageRequestSizePx: Int) =
        GalleryTile(card, {}, enabled = false, imageRequestSizePx = imageRequestSizePx, playVideo = false, corner = 0.dp)

    @Composable
    override fun Backdrop(card: Card, imageRequestSizePx: Int) = GalleryTileBackdrop(card, imageRequestSizePx)
}

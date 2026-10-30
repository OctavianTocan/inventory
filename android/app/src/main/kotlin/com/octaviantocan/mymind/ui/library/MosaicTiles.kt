package com.octaviantocan.mymind.ui.library

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.theme.LibraryColors

/**
 * What the blurred fill behind a pinch draws for a card: its [color], and its [picture], the address the tile loads,
 * once that picture is in memory.
 */
internal class MosaicBackdrop(val color: Color, val picture: String?)

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

    /** Width over height a picture tile ([isPicture]) is drawn at on its own page in reels; its one-column shape by default. */
    fun reelRatio(card: Card): Float = aspectRatio(card)

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
     * @param enabled False for a drawing that takes no touches; the mosaic passes true and keeps a pinch's touches itself.
     * @param imageRequestSizePx Size, in pixels, to decode the tile's pictures at.
     * @param modifier Modifier for the tile, sized to its cell.
     */
    @Composable
    fun Tile(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, enabled: Boolean, imageRequestSizePx: Int, modifier: Modifier)

    /**
     * The card on its own full-screen page in reels, fuller than its tile, which takes its touches and semantics there.
     *
     * @param card The card.
     * @param onClick Called when the page is tapped.
     * @param onLongClick Called when the page is held, or null when holding does nothing.
     * @param imageRequestSizePx Size, in pixels, to decode the card's pictures at.
     * @param modifier Modifier for the page, sized to it.
     */
    @Composable
    fun ReelPage(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, imageRequestSizePx: Int, modifier: Modifier)

    /**
     * A drawing of the card that takes no touches and plays nothing: its layout in the other grid while a tile with words
     * changes shape. The mosaic clears its semantics.
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
     * @param colors The page's colours.
     * @return What to draw, or null to leave the card's place empty.
     */
    fun backdrop(card: Card, colors: LibraryColors): MosaicBackdrop?
}

/** The library's own tiles ([GalleryTile]), shaped the way Google Photos breaks up its squares ([mosaicSpan]). */
internal object GalleryMosaicTiles : MosaicTiles {
    override fun span(card: Card): MosaicSpan = mosaicSpan(card)

    override fun aspectRatio(card: Card): Float = mosaicAspectRatio(card)

    override fun reelRatio(card: Card): Float = storedRatio(card) ?: mosaicAspectRatio(card)

    override fun isPicture(card: Card): Boolean = galleryTileIsPicture(card)

    override val corner: Dp = TILE_CORNER

    @Composable
    override fun Tile(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, enabled: Boolean, imageRequestSizePx: Int, modifier: Modifier) =
        GalleryTile(card, onClick, modifier = modifier, enabled = enabled, onLongClick = onLongClick, imageRequestSizePx = imageRequestSizePx,
            corner = 0.dp)

    @Composable
    override fun ReelPage(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, imageRequestSizePx: Int, modifier: Modifier) =
        GalleryReelPage(card, onClick, onLongClick, imageRequestSizePx, modifier)

    @Composable
    override fun Copy(card: Card, imageRequestSizePx: Int) =
        GalleryTile(card, {}, enabled = false, imageRequestSizePx = imageRequestSizePx, playVideo = false, corner = 0.dp)

    override fun backdrop(card: Card, colors: LibraryColors): MosaicBackdrop = galleryBackdrop(card, colors)
}

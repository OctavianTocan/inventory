package com.octaviantocan.mymind.ui.library

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Matrix
import coil3.request.ImageRequest
import coil3.request.transformations
import coil3.size.Size
import coil3.transform.Transformation
import com.octaviantocan.mymind.data.Card

/** How far the card's picture is turned clockwise, in degrees: 0, 90, 180 or 270. */
internal val Card.turn: Int get() = Math.floorMod(rotation ?: 0, 360).let { it - it % 90 }

/** Whether the card's turn swaps its picture's width and height. */
internal val Card.turnedSideways: Boolean get() = turn % 180 == 90

/**
 * Whether the card is a picture the person can turn: an image or screenshot they saved, not a video, PDF or link.
 *
 * @param card Card the menu is for.
 * @return True when Rotate is offered.
 */
fun canTurnPicture(card: Card): Boolean = galleryPicture(card) != null &&
    (card.type in setOf("Image", "Screenshot") || card.blob?.type?.startsWith("image/") == true)

/** Turns a decoded picture clockwise by [degrees], so it loads at its turned size and crops or fits as usual. */
private class TurnPicture(private val degrees: Int) : Transformation() {
    override val cacheKey: String = "turn-$degrees"

    override suspend fun transform(input: Bitmap, size: Size): Bitmap =
        Bitmap.createBitmap(input, 0, 0, input.width, input.height, Matrix().apply { postRotate(degrees.toFloat()) }, true)
}

/**
 * Adds [turn] to a picture request; an upright picture is left as it was.
 *
 * @param turn Degrees clockwise, from [Card.turn].
 * @return This builder.
 */
internal fun ImageRequest.Builder.turned(turn: Int): ImageRequest.Builder = if (turn == 0) this else transformations(TurnPicture(turn))

/**
 * What Coil loads for [url] turned by [turn]: the address itself when upright, so it shares the cache with every other
 * screen, otherwise a request that turns it.
 *
 * @param context Any context.
 * @param url Full address of the picture.
 * @param turn Degrees clockwise, from [Card.turn].
 * @return A Coil model.
 */
internal fun turnedPicture(context: Context, url: String, turn: Int): Any =
    if (turn == 0) url else ImageRequest.Builder(context).data(url).turned(turn).build()

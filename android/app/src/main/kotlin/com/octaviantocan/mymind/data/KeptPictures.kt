package com.octaviantocan.mymind.data

/** How many of the gallery's newest cards keep their pictures on the phone, besides Pinned. */
const val KEPT_PICTURE_CARDS = 1_000

/**
 * Fetches ahead the gallery pictures of the cards the phone keeps for use offline: Pinned and the gallery's
 * [KEPT_PICTURE_CARDS] newest cards, so they show without internet.
 *
 * @param copy The phone's copy, which lists the cards.
 * @param pictureOf The address of a card's gallery picture, or null when it has none.
 * @param kept True when a picture is on the phone already.
 * @param fetch Downloads one picture onto the phone; false when that failed.
 */
class KeptPictures(
    private val copy: LibraryCopy,
    private val pictureOf: (Card) -> String?,
    private val kept: (String) -> Boolean,
    private val fetch: suspend (String) -> Boolean,
) {
    /**
     * Fetches the kept cards' pictures the phone does not have yet, Pinned first, then newest first.
     *
     * @param stopped Asked before each picture; true ends the run, such as when Wi-Fi is gone.
     * @return How many pictures were fetched, or null while the copy does not hold the whole library.
     */
    suspend fun fetchMissing(stopped: () -> Boolean = { false }): Int? {
        if (!copy.complete()) return null
        val pictures = copy.pictureCards(KEPT_PICTURE_CARDS)?.mapNotNull(pictureOf)?.distinct() ?: return null
        var fetched = 0
        for (picture in pictures) {
            if (stopped()) break
            if (!kept(picture) && fetch(picture)) fetched++
        }
        return fetched
    }
}

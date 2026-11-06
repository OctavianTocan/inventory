package com.octaviantocan.mymind.ui.theme

import androidx.compose.runtime.Immutable

/**
 * A set of small line drawings for a doodle backdrop, each an SVG path on a 24 unit square drawn as a stroke. They are
 * ThyMind's own drawings of things one saves and keeps (books, notes, ideas, photos, plants, the night sky), made for
 * this app in the manner of chat wallpapers, not copied from any.
 *
 * @property name What the set shows, for tests and previews.
 * @property shapes The drawings.
 * @property seed Where the scatter starts, so a set always falls the same way.
 */
@Immutable
class DoodleSet(val name: String, val shapes: List<String>, val seed: Int)

/** ThyMind's own doodles. */
object Doodles {
    private const val STAR = "M12 3 L14.6 9 L21 9.5 L16 13.6 L17.6 20 L12 16.6 L6.4 20 L8 13.6 L3 9.5 L9.4 9 Z"
    private const val HEART = "M12 20 C5 15 3 11.5 3 8.5 C3 5.5 5.3 4 7.5 4 C9.5 4 11 5.2 12 7 C13 5.2 14.5 4 16.5 4 C18.7 4 21 5.5 21 8.5 C21 11.5 19 15 12 20 Z"
    private const val MOON = "M15 3 A9 9 0 1 0 21 15 A7 7 0 1 1 15 3 Z"
    private const val CLOUD = "M7 18 H17 A4 4 0 0 0 17 10 A5.5 5.5 0 0 0 6.5 11 A3.5 3.5 0 0 0 7 18 Z"
    private const val BOOK = "M3 6 C6 5 9 5 12 7 C15 5 18 5 21 6 V19 C18 18 15 18 12 20 C9 18 6 18 3 19 Z M12 7 V20"
    private const val BOOKMARK = "M7 3 H17 V21 L12 17 L7 21 Z"
    private const val PENCIL = "M4 20 L5 15.5 L16 4.5 L19.5 8 L8.5 19 Z M14 6.5 L17.5 10"
    private const val BULB = "M9 18 H15 M10 21 H14 M12 3 A6 6 0 0 0 8.5 14 V16 H15.5 V14 A6 6 0 0 0 12 3 Z"
    private const val PLANE = "M3 11 L21 3 L14 21 L11 13 Z M11 13 L21 3"
    private const val CUP = "M5 8 H16 V14 A5 5 0 0 1 11 19 H10 A5 5 0 0 1 5 14 Z M16 10 H17.5 A2 2 0 0 1 17.5 14 H16 M9 3 C8 4 10 5 9 6 M12.5 3 C11.5 4 13.5 5 12.5 6"
    private const val NOTE = "M5 3 H15 L19 7 V21 H5 Z M15 3 V7 H19 M8 11 H16 M8 14 H16 M8 17 H13"
    private const val LEAF = "M5 19 C5 10 10 5 20 4 C19 14 14 19 5 19 Z M5 19 L13 11"
    private const val SPARKLE = "M12 3 C12.8 8.5 15.5 11.2 21 12 C15.5 12.8 12.8 15.5 12 21 C11.2 15.5 8.5 12.8 3 12 C8.5 11.2 11.2 8.5 12 3 Z"
    private const val MUSIC = "M5 17 A3 3 0 1 0 11 17 A3 3 0 1 0 5 17 Z M11 17 V4 L19 6 V9 L11 7"
    private const val CAMERA = "M3 8 H7 L9 5 H15 L17 8 H21 V19 H3 Z M8.5 13 A3.5 3.5 0 1 0 15.5 13 A3.5 3.5 0 1 0 8.5 13 Z"
    private const val FLOWER = "M10 12 A2 2 0 1 0 14 12 A2 2 0 1 0 10 12 Z M9 6.5 A3 3 0 1 0 15 6.5 A3 3 0 1 0 9 6.5 Z M9 17.5 A3 3 0 1 0 15 17.5 A3 3 0 1 0 9 17.5 Z M3.5 12 A3 3 0 1 0 9.5 12 A3 3 0 1 0 3.5 12 Z M14.5 12 A3 3 0 1 0 20.5 12 A3 3 0 1 0 14.5 12 Z"
    private const val SUN = "M8 12 A4 4 0 1 0 16 12 A4 4 0 1 0 8 12 Z M12 2 V5 M12 19 V22 M2 12 H5 M19 12 H22 M5 5 L7 7 M17 17 L19 19 M5 19 L7 17 M17 7 L19 5"
    private const val PLANET = "M7 12 A5 5 0 1 0 17 12 A5 5 0 1 0 7 12 Z M5 14.6 C1.5 17 3 18.2 7 17 C11 15.8 17 12.5 19.5 10 C21.5 8 20.5 6.8 18.5 7.2"
    private const val LENS = "M4 10 A6 6 0 1 0 16 10 A6 6 0 1 0 4 10 Z M14.5 14.5 L20 20"
    private const val POT = "M7 14 H17 L16 21 H8 Z M12 14 V9 M12 10 C9 10 7 8 7 5 C10 5 12 7 12 10 Z M12 9 C12 6 14 4 17 4 C17 7 15 9 12 9 Z"
    private const val ENVELOPE = "M3 6 H21 V18 H3 Z M3 6 L12 13 L21 6"
    private const val PIN = "M12 21 C8 16 6 13 6 10 A6 6 0 0 1 18 10 C18 13 16 16 12 21 Z M10 10 A2 2 0 1 0 14 10 A2 2 0 1 0 10 10 Z"

    /** Things a library holds: books, notes, ideas, pictures and songs. */
    val Notebook = DoodleSet("Notebook", listOf(BOOK, BOOKMARK, PENCIL, BULB, PLANE, NOTE, LENS, CUP, STAR, SPARKLE, HEART, MUSIC, CAMERA), seed = 11)

    /** Leaves, flowers, pots and weather. */
    val Garden = DoodleSet("Garden", listOf(LEAF, FLOWER, SUN, CLOUD, POT, SPARKLE, HEART, LEAF, FLOWER), seed = 23)

    /** Stars, moons, planets and the odd cloud. */
    val NightSky = DoodleSet("Night sky", listOf(STAR, MOON, SPARKLE, PLANET, CLOUD, STAR, SPARKLE), seed = 37)

    /** Things sent and kept from trips: letters, places, photos and songs. */
    val Postcard = DoodleSet("Postcard", listOf(ENVELOPE, PIN, CAMERA, PLANE, HEART, SUN, MUSIC, STAR, CUP), seed = 53)

    /** Every set. */
    val all = listOf(Notebook, Garden, NightSky, Postcard)
}

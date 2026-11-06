package com.octaviantocan.mymind.ui.sky

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.vector.PathParser

/** The side of the square a figure is drawn in; its stars and line art use these units, y down. */
internal const val FIGURE_BOX = 200f

/**
 * How a constellation is drawn: where its stars sit, the lines between them, and the picture that draws in once every
 * star is lit.
 *
 * @property id The server's figure, such as `ship`.
 * @property stars Where each of the constellation's finds sits, in order: the recent save first, then the old finds.
 * @property edges The lines between stars, by index into [stars].
 * @property art The line picture as SVG path data in a [FIGURE_BOX] square.
 */
internal class SkyFigure(val id: String, val stars: List<Offset>, val edges: List<Pair<Int, Int>>, val art: String) {
    /** [art] as a path, parsed once. */
    val path: Path by lazy { PathParser().parsePathString(art).toPath() }
}

/**
 * The figure the server names [id], or the ship for one this app does not know yet, so a newer server never leaves a
 * constellation undrawn.
 */
internal fun figureOf(id: String): SkyFigure = FIGURES.firstOrNull { it.id == id } ?: FIGURES.first()

// One for each of the server's figures (`rust/crates/core/src/sky.rs`, FIGURES), in its order.
internal val FIGURES = listOf(
    SkyFigure(
        "ship",
        listOf(Offset(26.4f, 120.8f), Offset(96f, 152.8f), Offset(176.8f, 116f), Offset(98.4f, 24.8f), Offset(149.6f, 109.6f), Offset(50.4f, 111.2f)),
        listOf(0 to 1, 1 to 2, 1 to 3, 3 to 4, 4 to 2, 3 to 5, 5 to 0),
        "M 21.6 119.2 C 34.4 144.8 61.6 156 96 156 C 135.2 156 165.6 144.8 180 114.4 M 21.6 119.2 L 180 114.4 M 98.4 117.6 L 98.4 18.4 M 98.4 18.4 Q 108 21.6 117.6 18.4 Q 109.6 24.8 98.4 26.4 M 103.2 28 Q 141.6 60 151.2 108 L 103.2 108 M 93.6 36 Q 60 69.6 48.8 109.6 L 93.6 109.6 M 8 168.8 Q 24 162.4 40 168.8 Q 56 175.2 72 168.8 Q 88 162.4 104 168.8 Q 120 175.2 136 168.8 Q 152 162.4 168 168.8 Q 180 173.6 192 168.8",
    ),
    SkyFigure(
        "cup",
        listOf(Offset(50.4f, 87.2f), Offset(141.6f, 87.2f), Offset(96f, 149.6f), Offset(172f, 111.2f), Offset(109.6f, 26.4f), Offset(165.6f, 156f)),
        listOf(0 to 1, 0 to 2, 2 to 1, 1 to 3, 4 to 0, 4 to 1, 2 to 5),
        "M 50.4 87.2 Q 96 78.4 141.6 87.2 Q 96 96 50.4 87.2 M 50.4 87.2 C 50.4 127.2 63.2 149.6 96 149.6 C 128.8 149.6 141.6 127.2 141.6 87.2 M 140 98.4 C 178.4 95.2 180 128.8 135.2 128.8 M 26.4 156 Q 96 170.4 165.6 156 M 77.6 76 C 63.2 60 93.6 52 80.8 32.8 M 106.4 74.4 C 93.6 58.4 124 48.8 109.6 26.4",
    ),
    SkyFigure(
        "book",
        listOf(Offset(28f, 66f), Offset(100f, 72.4f), Offset(172f, 66f), Offset(172f, 139.6f), Offset(100f, 146f), Offset(28f, 139.6f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 3 to 4, 4 to 5, 5 to 0, 1 to 4),
        "M 28 66 Q 64 54.8 100 72.4 L 100 146 Q 64 128.4 28 139.6 L 28 66 M 172 66 Q 136 54.8 100 72.4 M 100 146 Q 136 128.4 172 139.6 L 172 66 M 42.4 82 Q 68 75.6 88.8 86.8 M 42.4 98 Q 68 91.6 88.8 102.8 M 111.2 86.8 Q 132 75.6 157.6 82 M 111.2 102.8 Q 132 91.6 157.6 98",
    ),
    SkyFigure(
        "mountain",
        listOf(Offset(8f, 170f), Offset(70f, 62f), Offset(100f, 104f), Offset(138f, 22f), Offset(192f, 170f), Offset(100f, 170f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 3 to 4, 4 to 5, 5 to 0),
        "M 8 170 L 70 62 L 100 104 L 138 22 L 192 170 M 4 170 L 196 170 M 121 58 L 128 50 L 135 60 L 142 51 L 151 58 M 57 84 L 64 78 L 71 86 L 78 79 L 86 84 M 138 22 L 138 4 L 156 9 L 138 14 M 100 170 Q 122 152 110 136 Q 100 122 120 108",
    ),
    SkyFigure(
        "lantern",
        listOf(Offset(100f, 8f), Offset(50f, 58f), Offset(150f, 58f), Offset(60f, 172f), Offset(140f, 172f), Offset(100f, 134f)),
        listOf(0 to 1, 0 to 2, 1 to 3, 2 to 4, 3 to 4, 3 to 5, 5 to 4),
        "M 90 18 A 10 10 0 1 1 110 18 A 10 10 0 1 1 90 18 M 50 58 L 100 28 L 150 58 Z M 56 58 L 144 58 L 144 66 L 56 66 Z M 62 66 L 66 156 M 138 66 L 134 156 M 54 156 L 146 156 L 140 172 L 60 172 Z M 100 92 C 114 108 112 124 100 134 C 88 124 86 108 100 92 Z M 100 110 C 105 116 104 124 100 128 C 96 124 95 116 100 110",
    ),
    SkyFigure(
        "camera",
        listOf(Offset(14f, 64f), Offset(70f, 40f), Offset(186f, 64f), Offset(186f, 166f), Offset(14f, 166f), Offset(104f, 112f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 3 to 4, 4 to 0, 5 to 1),
        "M 14 64 L 58 64 L 70 40 L 122 40 L 134 64 L 186 64 L 186 166 L 14 166 Z M 66 112 A 38 38 0 1 1 142 112 A 38 38 0 1 1 66 112 M 80 112 A 24 24 0 1 1 128 112 A 24 24 0 1 1 80 112 M 92 100 Q 98 94 106 94 M 150 64 L 150 54 L 172 54 L 172 64 M 26 78 L 46 78 L 46 90 L 26 90 Z",
    ),
    SkyFigure(
        "harp",
        listOf(Offset(40f, 186f), Offset(40f, 30f), Offset(96f, 14f), Offset(138f, 38f), Offset(178f, 16f), Offset(100f, 114f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 3 to 4, 4 to 5, 5 to 0),
        "M 40 186 L 40 30 M 40 30 C 58 4 84 2 96 14 C 108 28 124 44 138 38 C 152 30 168 12 178 16 M 178 16 Q 188 20 182 28 Q 176 32 174 26 M 178 16 L 46 182 M 30 186 L 60 186 M 60 15 L 60 164 M 80 9 L 80 139 M 100 18 L 100 114 M 120 36 L 120 89 M 140 37 L 140 64",
    ),
    SkyFigure(
        "tree",
        listOf(Offset(100f, 186f), Offset(100f, 122f), Offset(40f, 76f), Offset(86f, 32f), Offset(142f, 36f), Offset(162f, 96f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 3 to 4, 4 to 5, 5 to 1),
        "M 92 186 C 94 160 96 140 94 122 M 108 186 C 106 160 104 140 106 122 M 40 186 Q 100 180 160 186 M 100 122 C 52 136 22 104 40 76 C 26 46 60 22 86 32 C 98 6 136 10 142 36 C 176 36 186 80 162 96 C 166 126 126 132 100 122 Z M 94 122 L 72 92 M 106 122 L 130 88 M 100 122 L 100 72 M 92 186 Q 84 190 74 190 M 108 186 Q 116 190 126 190",
    ),
    SkyFigure(
        "house",
        listOf(Offset(34f, 182f), Offset(34f, 88f), Offset(100f, 28f), Offset(166f, 88f), Offset(166f, 182f), Offset(143f, 36f)),
        listOf(0 to 1, 1 to 2, 2 to 5, 5 to 3, 3 to 4, 4 to 0),
        "M 34 182 L 34 88 M 166 88 L 166 182 M 26 182 L 174 182 M 16 104 L 100 28 L 184 104 M 134 58 L 134 36 L 152 36 L 152 74 M 86 182 L 86 134 L 114 134 L 114 182 M 48 116 L 74 116 L 74 142 L 48 142 Z M 61 116 L 61 142 M 48 129 L 74 129 M 126 116 L 152 116 L 152 142 L 126 142 Z M 139 116 L 139 142 M 126 129 L 152 129 M 143 30 C 136 22 150 16 142 8",
    ),
    SkyFigure(
        "compass",
        listOf(Offset(100f, 22f), Offset(166f, 100f), Offset(100f, 178f), Offset(34f, 100f), Offset(100f, 100f), Offset(159.4f, 40.6f)),
        listOf(0 to 4, 4 to 2, 3 to 4, 4 to 1, 0 to 5, 5 to 1),
        "M 16 100 A 84 84 0 1 1 184 100 A 84 84 0 1 1 16 100 M 28 100 A 72 72 0 1 1 172 100 A 72 72 0 1 1 28 100 M 100 22 L 114 100 L 100 178 L 86 100 Z M 86 100 L 114 100 M 34 100 L 100 92 L 166 100 L 100 108 Z M 49 49 L 57 57 M 151 49 L 143 57 M 49 151 L 57 143 M 151 151 L 143 143 M 95 100 A 5 5 0 1 1 105 100 A 5 5 0 1 1 95 100 M 92 14 L 100 2 L 108 14",
    ),
    SkyFigure(
        "anvil",
        listOf(Offset(10f, 78f), Offset(178f, 64f), Offset(140f, 116f), Offset(164f, 172f), Offset(56f, 172f), Offset(80f, 116f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 3 to 4, 4 to 5, 5 to 0),
        "M 44 64 L 178 64 L 178 88 L 150 88 Q 136 100 140 116 L 140 134 L 164 152 L 164 172 L 56 172 L 56 152 L 80 134 L 80 116 Q 84 100 70 92 Q 40 86 10 78 Q 28 64 44 64 Z M 44 72 L 178 72 M 110 52 L 104 30 M 126 50 L 140 28 M 96 54 L 80 38 M 142 54 L 160 44",
    ),
    SkyFigure(
        "bird",
        listOf(Offset(182f, 78f), Offset(146f, 90f), Offset(132f, 12f), Offset(38f, 34f), Offset(18f, 134f), Offset(117f, 116f)),
        listOf(0 to 1, 1 to 2, 1 to 5, 5 to 3, 5 to 4),
        "M 182 78 Q 160 74 146 90 Q 112 104 46 112 L 18 134 L 72 120 Q 124 124 150 96 Q 162 84 182 78 M 140 94 Q 128 50 132 12 Q 112 54 96 102 M 112 100 Q 78 62 38 34 Q 66 80 80 106 M 182 78 L 196 82 L 180 86 M 168 80 A 2 2 0 1 1 172 80 A 2 2 0 1 1 168 80",
    ),
    SkyFigure(
        "wave",
        listOf(Offset(8f, 166f), Offset(84f, 96f), Offset(162f, 30f), Offset(172f, 76f), Offset(96f, 184f), Offset(192f, 184f)),
        listOf(0 to 1, 1 to 2, 2 to 3, 0 to 4, 4 to 5, 5 to 3),
        "M 8 166 C 48 158 70 130 84 96 C 100 56 128 28 162 30 C 190 32 194 64 172 76 C 156 84 140 72 150 58 C 156 50 166 52 168 58 M 96 120 C 108 86 126 62 148 52 M 162 30 Q 172 18 184 24 M 140 34 Q 146 22 158 22 M 8 184 Q 30 176 52 184 Q 74 192 96 184 Q 118 176 140 184 Q 162 192 192 184 M 186 40 A 2 2 0 1 1 190 40 A 2 2 0 1 1 186 40",
    ),
    SkyFigure(
        "coin",
        listOf(Offset(90f, 42f), Offset(160f, 112f), Offset(90f, 182f), Offset(20f, 112f), Offset(90f, 82f), Offset(169f, 31f)),
        listOf(3 to 0, 0 to 5, 5 to 1, 1 to 2, 2 to 3, 0 to 4),
        "M 98.1 22.3 A 58 58 0 0 1 183.2 89.9 M 20 112 A 70 70 0 1 1 160 112 A 70 70 0 1 1 20 112 M 34 112 A 56 56 0 1 1 146 112 A 56 56 0 1 1 34 112 M 62 132 L 56 92 L 76 110 L 90 82 L 104 110 L 124 92 L 118 132 Z M 64 142 L 116 142",
    ),
)

package com.octaviantocan.mymind.ui

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.graphics.drawscope.DrawScope
import com.octaviantocan.mymind.ui.card.CardHeaderIcons
import com.octaviantocan.mymind.ui.errors.OfflinePictures
import com.octaviantocan.mymind.ui.splash.SplashPictures

/** Flat-colour stand-in for a picture, sized in pixels like the decoded bitmap it replaces. */
class BlockPainter(private val color: Color, private val pixels: Size) : Painter() {
    override val intrinsicSize: Size = pixels

    override fun DrawScope.onDraw() = drawRect(color)
}

/** Alternating horizontal bands in two colours, so a blur over the picture is visible. */
class BandPainter(private val first: Color, private val second: Color, private val pixels: Size, private val bands: Int = 16) : Painter() {
    override val intrinsicSize: Size = pixels

    override fun DrawScope.onDraw() {
        val band = size.height / bands
        for (i in 0 until bands) {
            drawRect(if (i % 2 == 0) first else second, Offset(0f, i * band), Size(size.width, band))
        }
    }
}

private const val XXXHDPI_TO_DEVICE = 450f / 640f
private const val MDPI_TO_DEVICE = 450f / 160f

private fun scaled(width: Int, height: Int, factor: Float) = Size(width * factor, height * factor)

/** Splash pictures with the pixel sizes of the shipped bitmaps and none of their artwork. */
fun flatSplashPictures() = SplashPictures(
    background = BandPainter(Color(0xFFD97A97), Color(0xFFF2C3D2), scaled(1500, 1696, XXXHDPI_TO_DEVICE)),
    icon = BlockPainter(Color.White, scaled(672, 1052, XXXHDPI_TO_DEVICE)),
)

/** Offline pictures with the pixel sizes of the shipped bitmaps and none of their artwork. */
fun flatOfflinePictures(dark: Boolean) = OfflinePictures(
    background = BandPainter(
        if (dark) Color(0xFF1B2A4A) else Color(0xFF8CA0C8),
        if (dark) Color(0xFF2B3C63) else Color(0xFFB4C3E0),
        if (dark) Size(750f, 1624f) else scaled(375, 812, MDPI_TO_DEVICE),
    ),
    notice = BlockPainter(Color(0xFFEFEFEF), Size(704f, 492f)),
)

/** Card header icons as 24 dp squares. */
fun flatCardHeaderIcons() = CardHeaderIcons(
    close = BlockPainter(Color.Gray, Size(67.5f, 67.5f)),
    menu = BlockPainter(Color.Gray, Size(67.5f, 67.5f)),
)

package com.octaviantocan.mymind.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.ui.card.CardHeader
import com.octaviantocan.mymind.ui.card.CardHeaderState
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class CardHeaderScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    private val state = CardHeaderState()

    private fun start(dark: Boolean) {
        compose.setPhoneContent(dark) {
            val card = Color(0xFF7FA3D6)
            Box(Modifier.fillMaxSize().background(card)) {
                Image(
                    remember { BandPainter(Color(0xFFD9534F), Color(0xFFF7C94B), Size(100f, 400f), bands = 8) },
                    contentDescription = null,
                    modifier = Modifier.fillMaxWidth().height(160.dp),
                    contentScale = ContentScale.FillBounds,
                )
                CardHeader(
                    title = "A card with a title",
                    color = card,
                    state = state,
                    onClose = {},
                    onMenu = {},
                    icons = flatCardHeaderIcons(),
                )
            }
        }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun scrollTo(vararg positions: Int) {
        compose.change { positions.forEach(state::onScroll) }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun capture(name: String) = compose.onRoot().captureRoboImage("screenshots/$name.png")

    private fun sequence(dark: Boolean, suffix: String) {
        start(dark)
        capture("card_header_rest_$suffix")
        scrollTo(300)
        capture("card_header_hidden_$suffix")
        scrollTo(240)
        capture("card_header_shown_$suffix")
    }

    @Test
    fun cardHeaderLight() = sequence(dark = false, suffix = "light")

    @Test
    fun cardHeaderDark() = sequence(dark = true, suffix = "dark")

    private companion object {
        const val SETTLE_MS = 600L
    }
}

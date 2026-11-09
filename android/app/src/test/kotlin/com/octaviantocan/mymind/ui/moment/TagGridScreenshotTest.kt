package com.octaviantocan.mymind.ui.moment

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.setPhoneContent
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareSheetState
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/**
 * share-tag-step-icon-grid: the tag step on the card's back at the size it has on the reference phone, with text 10%
 * bigger as on Octavian's phone, so its spacing can be checked by eye.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = 1.1f)
class TagGridScreenshotTest {
    @get:Rule
    val compose = createComposeRule()

    @Test
    fun tagGrid() {
        val cameras = Space("s1", "Cameras", "#4f8a5b", count = 41)
        val link = "https://www.leica-camera.com/en-int/photography/m6"
        val state = ShareSheetState(listOf(ShareItem.Text(link)), SavePhase.Saved).apply {
            spaces = listOf(cameras, Space("s2", "Film", "#3e8db5", count = 23), Space("s3", "To buy", "#b5705a", count = 12))
            recentTags = listOf("35mm", "analog", "vintage", "gift idea", "photography")
            suggestion = SpaceSuggestion(cameras, 12)
            spaceIds = listOf("s1")
            cards = listOf(testCard("c1").copy(type = "Article", title = "Leica M6", siteName = "Leica Camera", url = link, domain = "leica-camera.com",
                tags = listOf(Tag("t1", "film camera", 2), Tag("t2", "rangefinder", 2), Tag("t3", "1980s", 2))))
            tags = listOf("street photography")
            saved = 1
        }
        compose.setPhoneContent(dark = true) {
            Box(Modifier.fillMaxSize().background(Color(0xFF06181D)), contentAlignment = Alignment.Center) {
                // The size TagBack gives it on this phone: the width less 18 dp a side, at the card's shape.
                TagGrid(state, Rarity.Legendary.color, left = { 0.6f }, onTouch = {}, onDone = {}, modifier = Modifier.size(348.dp, (348 / CARD_RATIO).dp))
            }
        }
        compose.mainClock.advanceTimeBy(1_000)
        compose.onRoot().captureRoboImage("screenshots/M9_TagGrid.png")
    }
}

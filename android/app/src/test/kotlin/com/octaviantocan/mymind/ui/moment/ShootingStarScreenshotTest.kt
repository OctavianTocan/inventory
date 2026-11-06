package com.octaviantocan.mymind.ui.moment

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.PHONE_FONT_SCALE
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.setPhoneContent
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareSheetState
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The shooting star moment (M03) as a film of stills, one rarity per run, then the tag grid and the backpack. */
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class ShootingStarScreenshotTest(private val rarity: Rarity) {
    @get:Rule
    val compose = createComposeRule()

    private val link = "https://www.leica-camera.com/en-int/photography/m6"
    private val reading = Space("s1", "Cameras", "#4f8a5b", count = 41)

    private fun state() = ShareSheetState(listOf(ShareItem.Text(link)), SavePhase.Saved).apply {
        spaces = listOf(reading, Space("s2", "Film", "#3e8db5", count = 23), Space("s3", "To buy", "#b5705a", count = 12))
        recentTags = listOf("35mm", "analog", "vintage", "gift idea")
        allTags = listOf("film camera", "35mm", "analog", "vintage", "gift idea", "photography")
        suggestion = SpaceSuggestion(reading, 12)
        cards = listOf(testCard("c1").copy(type = "Article", title = "Leica M6, the rangefinder camera of the 1980s", siteName = "Leica Camera", url = link,
            domain = "leica-camera.com", tags = listOf(Tag("t1", "film camera", 2), Tag("t2", "rangefinder", 2), Tag("t3", "1980s", 2))))
        saved = 1
    }

    @Test
    fun film() {
        val moment = StarMoment(null, null, seed = 7L).apply { rarity = this@ShootingStarScreenshotTest.rarity }
        val state = state()
        var tagOpen by mutableStateOf(false)
        compose.setPhoneContent(dark = true) {
            ShootingStar(moment, momentFindOf(state, 1_486), backpack = 1_485, saved = 1, tagOpen = tagOpen, onOutside = {}, onSky = {}) { modifier ->
                TagGrid(state, rarity.color, left = { 0.6f }, onTouch = {}, onDone = {}, modifier = modifier)
            }
        }
        var at = 0L
        fun shot(ms: Long) {
            compose.mainClock.advanceTimeBy(ms - at)
            at = ms
            compose.onRoot().captureRoboImage("screenshots/M03_${rarity.name}_${ms.toString().padStart(5, '0')}.png")
        }
        listOf(250L, 800L, 1_300L, 1_900L, 2_500L, 3_000L, 3_250L, 3_600L, 4_000L, 4_500L, 5_200L, 6_000L, 7_000L).forEach(::shot)
        compose.runOnIdle { assertEquals(StarPhase.Settled, moment.phase); tagOpen = true }
        shot(7_350L); shot(8_000L)
        compose.runOnIdle { tagOpen = false; moment.closeTags() }
        shot(8_400L); shot(9_000L); shot(9_350L); shot(9_700L); shot(10_400L)
    }

    companion object {
        @JvmStatic
        @ParameterizedRobolectricTestRunner.Parameters(name = "{0}")
        fun rarities() = listOf(Rarity.Legendary, Rarity.Rare, Rarity.Common)
    }
}

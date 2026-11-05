package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.theme.MymindTheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** How an audio card is told apart, timed and drawn in the grid. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class AudioCardsTest {
    @get:Rule val compose = createComposeRule()

    private fun memo(duration: Double? = 83.4) = testCard("a1", type = "Audio", title = "Voice memo.m4a",
        blob = Blob(path = "a1/memo.m4a", type = "audio/mp4", duration = duration))
        .copy(transcript = "Remember to call the dentist and then pick up the bread.")

    @Test
    fun anAudioCardIsItsOwnKindOnItsOwnPaperWithNoPicture() {
        assertEquals(TileKind.Audio, tileKind(memo()))
        assertNull(galleryPicture(memo()))
        assertEquals("Audio", galleryKind(memo()))
    }

    @Test
    fun aLengthShowsAsMinutesOrAsHoursWhenAnHourOrMoreAndNotAtAllWhenMissing() {
        assertEquals("1:23", audioLength(83.4))
        assertEquals("0:05", audioLength(4.6))
        assertEquals("59:59", audioLength(3599.0))
        assertEquals("1:01:01", audioLength(3661.0))
        assertNull(audioLength(null))
        assertNull(audioLength(0.0))
        assertNull(audioLength(Double.NaN))
    }

    @Test
    fun theTilesWordsAreWhatIsSaidElseTheSummary() {
        assertEquals("Remember to call the dentist and then pick up the bread.", audioWords(memo()))
        assertEquals("A short reminder.", audioWords(memo().copy(transcript = null, description = "A short reminder.")))
        assertEquals("", audioWords(memo().copy(transcript = " ", description = null)))
    }

    private fun render(card: Card, highlight: Boolean = false) {
        compose.setContent {
            MymindTheme(dark = true) {
                Box(Modifier.width(180.dp).height(240.dp)) { GalleryTile(card, {}, highlight = highlight, playVideo = false) }
            }
        }
    }

    @Test
    fun anAudioTileShowsItsTitleItsLengthAndTheStartOfWhatIsSaid() {
        render(memo())
        compose.onNodeWithText("Voice memo.m4a", useUnmergedTree = true).assertIsDisplayed()
        compose.onNodeWithText("1:23", useUnmergedTree = true).assertIsDisplayed()
        compose.onNodeWithText("Remember to call the dentist", substring = true, useUnmergedTree = true).assertIsDisplayed()
    }

    @Test
    fun anAudioTileWithNoLengthLeavesItOut() {
        render(memo(duration = null))
        compose.onNodeWithText("Voice memo.m4a", useUnmergedTree = true).assertIsDisplayed()
        compose.onNodeWithText("1:23", useUnmergedTree = true).assertDoesNotExist()
    }

    @Test
    fun theListTileShowsTheSameThree() {
        compose.setContent {
            MymindTheme(dark = true) { Box(Modifier.width(180.dp)) { CardTile(memo(), {}) } }
        }
        compose.onNodeWithText("Voice memo.m4a").assertIsDisplayed()
        compose.onNodeWithText("1:23").assertIsDisplayed()
        compose.onNodeWithText("Remember to call the dentist", substring = true).assertIsDisplayed()
    }
}


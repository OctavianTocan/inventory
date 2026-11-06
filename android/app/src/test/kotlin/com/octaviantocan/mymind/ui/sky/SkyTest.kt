package com.octaviantocan.mymind.ui.sky

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Constellation
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.data.SkyStar
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.setPhoneContent
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

private fun ids(prefix: String) = (0 until 6).map { "$prefix$it" }

private val ACTIVE = Constellation(1, "ship", "The Voyager", "c0", ids("c"), lit = listOf("c0", "c1", "c2"))

private val DONE = listOf(
    Constellation(2, "lantern", "The Lantern", "d0", ids("d"), lit = ids("d"), completed = "2026-10-20T10:00:00Z"),
    Constellation(3, "bird", "The Dreamer", "e0", ids("e"), lit = ids("e"), completed = "2026-10-30T10:00:00Z"),
)

private val LOOSE = (1..6).map { "l$it" }

private val SKY = Sky(
    day = "2026-11-05",
    back = listOf("c3", "l1", "l2"),
    active = ACTIVE,
    done = DONE,
    stars = (listOf("c1", "c2") + ids("d") + ids("e") + LOOSE).map { SkyStar(it, "2026-11-01T10:00:00Z") },
)

private val FINDS = (ids("c") + ids("d") + ids("e") + LOOSE).associateWith { testCard(it, title = "Find $it") } +
    ("c3" to testCard("c3", title = "Harbour at dusk"))

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class SkyTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun everyServerFigureHasSixJoinedStarsAndArtInItsSquare() {
        // The server's FIGURES in rust/crates/core/src/sky.rs, in its order.
        val server = listOf("ship", "cup", "book", "mountain", "lantern", "camera", "harp", "tree", "house", "compass", "anvil", "bird", "wave", "coin")
        assertEquals(server, FIGURES.map { it.id })
        for (figure in FIGURES) {
            assertEquals(figure.id, 6, figure.stars.size)
            assertTrue(figure.id, figure.stars.all { it.x in 0f..FIGURE_BOX && it.y in 0f..FIGURE_BOX })
            assertTrue(figure.id, figure.edges.all { (a, b) -> a != b && a in figure.stars.indices && b in figure.stars.indices })
            assertEquals(figure.id, figure.stars.indices.toSet(), figure.edges.flatMap { listOf(it.first, it.second) }.toSet())
            val bounds = figure.path.getBounds()
            assertTrue(figure.id, bounds.width >= FIGURE_BOX / 2 && bounds.left >= -12f && bounds.right <= FIGURE_BOX + 12f)
        }
        assertEquals("ship", figureOf("a figure from a newer server").id)
    }

    @Test
    fun theActiveConstellationIsLargeAndLooseStarsKeepOutOfIt() {
        val layout = skyLayout(SKY, 1080f, 2520f, 250f, 150f)
        val active = layout.constellations.first()
        assertTrue(active.active)
        assertEquals(ids("c"), active.stars.map { it.id })
        assertEquals(listOf(true, true, true, false, false, false), active.stars.map { it.lit })
        assertFalse(active.complete)
        val done = layout.constellations.drop(1)
        assertEquals(listOf("bird", "lantern"), done.map { it.figure.id })
        assertTrue(done.all { it.complete && it.size < active.size / 2 })
        assertEquals(LOOSE.toSet(), layout.loose.map { it.id }.toSet())
        val half = active.size / 2f
        for (star in layout.stars) assertTrue(star.id, star.at.x in 0f..1080f && star.at.y in 250f..2370f)
        for (star in layout.loose) {
            val inside = kotlin.math.abs(star.at.x - active.center.x) < half && kotlin.math.abs(star.at.y - active.center.y) < half
            assertFalse(star.id, inside)
        }
        // The same find sits in the same place every time.
        assertEquals(layout.stars.map { it.at }, skyLayout(SKY, 1080f, 2520f, 250f, 150f).stars.map { it.at })
    }

    @Test
    fun theSkyNamesItsConstellationAndOpensAStarsFind() {
        runBlocking { Nebula.picture() }
        val opened = mutableListOf<String>()
        var closed = false
        compose.setPhoneContent(true) { SkyScreen(SKY, FINDS, { opened += it }, { closed = true }) }
        compose.onNodeWithText("Your sky").assertIsDisplayed()
        compose.onNodeWithText("The Voyager").assertIsDisplayed()
        compose.onRoot().captureRoboImage("screenshots/sky.png")
        compose.onAllNodesWithContentDescription("Find c0, a lit star")[0].assertIsDisplayed()
        compose.onNodeWithContentDescription("Harbour at dusk, a dark star").performClick()
        assertEquals(listOf("c3"), opened)
        compose.onNodeWithContentDescription("Close the sky").performClick()
        assertTrue(closed)
    }

    @Test
    fun theStarButtonShowsOnceTheSkyHasStarsAndOpensIt() {
        var state by mutableStateOf(BackTodayState())
        var opens = 0
        compose.setPhoneContent(false) {
            EverythingContent(
                feed = FeedState(endReached = true), searching = false, searchResults = null,
                typeChips = emptyList(), filter = null, suggestions = emptyList(),
                onChoose = {}, onRemoveFilter = {}, onLoadMore = {}, onLoadMoreResults = {},
                onRetry = {}, onOpenCard = {}, onOpenSettings = {}, onAdd = {},
                backToday = state, onSkyOpened = { opens++ },
            )
        }
        compose.mainClock.advanceTimeByFrame()
        compose.onAllNodesWithContentDescription("Your sky").assertCountEquals(0)
        state = BackTodayState(SKY.copy(active = null, done = emptyList(), stars = emptyList()))
        compose.mainClock.advanceTimeByFrame()
        compose.onAllNodesWithContentDescription("Your sky").assertCountEquals(0)
        state = BackTodayState(SKY, finds = FINDS)
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithContentDescription("Your sky").performClick()
        compose.mainClock.advanceTimeBy(SKY_OPEN_MS + 32L)
        assertEquals(1, opens)
        compose.onNodeWithTag("sky").assertIsDisplayed()
        compose.onNodeWithText("The Voyager").assertIsDisplayed()
        compose.onNodeWithContentDescription("Close the sky").performClick()
        compose.mainClock.advanceTimeBy(SKY_OPEN_MS + 32L)
        compose.onAllNodesWithTag("sky").assertCountEquals(0)
    }
}

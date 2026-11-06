package com.octaviantocan.mymind.ui.sky

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.click
import androidx.compose.ui.test.performTouchInput
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
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.time.Instant

private fun ids(prefix: String) = (0 until 6).map { "$prefix$it" }

private val ACTIVE = Constellation(1, "ship", "The Voyager", "c0", ids("c"), lit = listOf("c0", "c1", "c2"))

private val DONE = listOf(
    Constellation(2, "lantern", "The Lantern", "d0", ids("d"), lit = ids("d"), completed = "2026-10-20T10:00:00Z"),
    Constellation(3, "bird", "The Dreamer", "e0", ids("e"), lit = ids("e"), completed = "2026-10-30T10:00:00Z"),
)

private val LOOSE = (1..3).map { "l$it" }

private val SKY = Sky(
    day = "2026-11-06",
    back = listOf("c3", "l1", "x1"),
    active = ACTIVE,
    done = DONE,
    stars = (listOf("c1", "c2") + ids("d").drop(1) + ids("e").drop(1) + LOOSE).map { SkyStar(it, "2026-11-01T10:00:00Z") },
)

private val FINDS = (ids("c") + ids("d") + ids("e") + LOOSE + "x1").associateWith { testCard(it, title = "Find $it") } +
    ("c3" to testCard("c3", title = "Harbour at dusk"))

// Every star but c2 has been seen light: c2 waits for the next look up.
private val SEEN = SKY.stars.map { it.id }.toSet() - "c2"

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class SkyTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun everyServerFigureHasSixJoinedStarsArtAndWordsForWhatItHolds() {
        // The server's FIGURES in rust/crates/core/src/sky.rs, in its order.
        val server = listOf("ship", "cup", "book", "mountain", "lantern", "camera", "harp", "tree", "house", "compass", "anvil", "bird", "wave", "coin")
        assertEquals(server, FIGURES.map { it.id })
        for (figure in FIGURES) {
            assertEquals(figure.id, 6, figure.stars.size)
            assertTrue(figure.id, figure.stars.all { it.x in 0f..FIGURE_BOX && it.y in 0f..FIGURE_BOX })
            assertTrue(figure.id, figure.edges.all { (a, b) -> a != b && a in figure.stars.indices && b in figure.stars.indices })
            assertEquals(figure.id, figure.stars.indices.toSet(), figure.edges.flatMap { listOf(it.first, it.second) }.toSet())
            assertTrue(figure.id, aboutOf(figure.id).isNotBlank())
            assertTrue(figure.id, figureStrokes(figure.art).isNotEmpty())
        }
        assertEquals("ship", figureOf("a figure from a newer server").id)
    }

    @Test
    fun thePlanLightsWaitingFindsInFrontOfThePersonAndAFinishedOneFirst() {
        val plan = skyPlan(SKY, SEEN, emptySet()).plan
        assertEquals(1L, plan.center?.id)
        assertEquals(listOf("c2"), plan.pending)
        assertEquals(listOf(2L, 3L), plan.slots.map { it.id })
        assertEquals(LOOSE, plan.loose)
        // A star still flying to the button is not lit and does not wait yet.
        val flying = skyPlan(SKY, SEEN + "c2", setOf("c1")).plan
        assertFalse("c1" in flying.center!!.lit)
        assertTrue(flying.pending.isEmpty())
        // A constellation finished while the sky was away comes to the middle first, then the next forms.
        val finished = SKY.copy(done = DONE + Constellation(4, "cup", "The Hearth", "f0", ids("f"), lit = ids("f"), completed = "2026-11-06T09:00:00Z"),
            stars = SKY.stars + SkyStar("f5", "2026-11-06T09:00:00Z"))
        val staged = skyPlan(finished, SEEN + "c2", emptySet()).plan
        assertEquals(4L, staged.center?.id)
        assertEquals(listOf(1L), staged.next.map { it.id })
        assertEquals(listOf("f5"), staged.pending)
        // Before the seen stars are read nothing waits.
        assertTrue(skyPlan(SKY, null, emptySet()).plan.pending.isEmpty())
    }

    @Test
    fun aFindIsNamedInOneLine() {
        assertEquals("Harbour at dusk", findTitle(testCard("a", title = "Harbour   at\ndusk")))
        val long = findTitle(testCard("b", title = "A very long title for a find that goes on far past one line under its star"))
        assertTrue(long.length <= 46 && long.endsWith("…"))
        val now = Instant.parse("2026-11-06T12:00:00Z")
        assertEquals(2f, hologramStage(testCard("c").copy(bumped = "2026-08-01T00:00:00Z", created = "2026-08-01T00:00:00Z"), now))
        assertEquals(3f, hologramStage(testCard("d").copy(bumped = "2024-01-01T00:00:00Z", created = "2024-01-01T00:00:00Z"), now))
    }

    @Test
    fun pullingDownAtTheTopLooksUpAtTheSkyWhereAStarLightsAndASwipeUpComesBack() {
        val state by mutableStateOf(BackTodayState(SKY, FINDS.filterKeys { it in SKY.back }.values.sortedBy { SKY.back.indexOf(it.id) }, FINDS, SEEN))
        var refreshes = 0
        val seen = mutableListOf<String>()
        compose.setPhoneContent(false) {
            EverythingContent(
                feed = FeedState(cards = (1..12).map { testCard("g$it", title = "Gallery $it") }, endReached = true), searching = false, searchResults = null,
                typeChips = emptyList(), filter = null, suggestions = emptyList(),
                onChoose = {}, onRemoveFilter = {}, onLoadMore = {}, onLoadMoreResults = {},
                onRetry = {}, onOpenCard = {}, onOpenSettings = {}, onAdd = {},
                backToday = state, onRefresh = { refreshes++ }, onSeen = { seen += it },
            )
        }
        compose.mainClock.advanceTimeBy(100)
        compose.onRoot().captureRoboImage("screenshots/sky_00_gallery.png")
        compose.onAllNodesWithContentDescription("A find from your past, flickering as a hologram").assertCountEquals(2)
        compose.onNodeWithContentDescription("Your sky, 1 waiting to light").assertExists()

        // Pull down from the top of the page: the camera tilts up with the finger.
        compose.onRoot().performTouchInput {
            down(Offset(centerX, 500f))
            repeat(24) { moveBy(Offset(0f, 25f)); advanceEventTime(16) }
        }
        compose.mainClock.advanceTimeBy(64)
        compose.onRoot().captureRoboImage("screenshots/sky_01_pull.png")
        compose.onRoot().performTouchInput { up() }
        compose.mainClock.advanceTimeBy(1200)
        compose.onRoot().captureRoboImage("screenshots/sky_02_looking_up.png")
        assertEquals("looking up syncs the library", 1, refreshes)
        compose.mainClock.advanceTimeBy(1700)
        compose.onRoot().captureRoboImage("screenshots/sky_03_sky.png")
        compose.mainClock.advanceTimeBy(4000)
        compose.onRoot().captureRoboImage("screenshots/sky_04_lit.png")
        assertEquals(listOf("c2"), seen)
        compose.onNodeWithContentDescription("Your sky").assertExists()
        compose.onAllNodesWithContentDescription("Close the sky").assertCountEquals(0)

        // A swipe up comes back down to the page.
        compose.onRoot().performTouchInput {
            down(Offset(centerX, 1800f))
            repeat(20) { moveBy(Offset(0f, -40f)); advanceEventTime(16) }
            up()
        }
        compose.mainClock.advanceTimeBy(3200)
        compose.onAllNodesWithContentDescription("Your sky").assertCountEquals(0)
        compose.onAllNodesWithContentDescription("A find from your past, flickering as a hologram").assertCountEquals(2)
    }

    @Test
    fun aTappedStarCondensesIntoItsFindAndASwipeGlidesToTheNextStar() {
        val sky = SkyState()
        val opened = mutableListOf<String>()
        compose.setPhoneContent(true) {
            Box(Modifier.fillMaxSize()) {
                SkyBackdrop(sky, skyPlan(SKY, SKY.stars.map { it.id }.toSet(), emptySet()), FINDS, paused = false, onOpenFind = { opened += it }, onSeen = {}, onLookUp = {})
                SkyOverlay(sky)
            }
        }
        compose.mainClock.advanceTimeByFrame()
        sky.lookUp()
        compose.mainClock.advanceTimeBy(3000)
        assertTrue(sky.lookingUp)

        // Harbour at dusk is the dark star at index 3, the mast's top.
        val (x, y) = assertNotNullPoint(sky, 3)
        compose.onRoot().performTouchInput { click(Offset(x, y)) }
        compose.mainClock.advanceTimeBy(1100)
        compose.onRoot().captureRoboImage("screenshots/sky_05_condensing.png")
        compose.mainClock.advanceTimeBy(2000)
        compose.onRoot().captureRoboImage("screenshots/sky_06_condensed.png")
        assertEquals("Harbour at dusk", sky.scene.memTitle)
        assertTrue("moving in does not open the find", opened.isEmpty())

        // From the mast's top, a swipe up and to the left glides down to the sail's right foot (index 4).
        compose.onRoot().performTouchInput {
            down(Offset(centerX, centerY))
            repeat(10) { moveBy(Offset(-30f, -30f)); advanceEventTime(16) }
            up()
        }
        compose.mainClock.advanceTimeBy(450)
        compose.onRoot().captureRoboImage("screenshots/sky_07_gliding.png")
        compose.mainClock.advanceTimeBy(3000)
        assertEquals("c4", sky.scene.focusedCard)
        assertTrue("a swipe does not open the find", opened.isEmpty())

        // A tap on the find opens it; a tap on empty sky moves back out.
        compose.onRoot().performTouchInput { click(Offset(centerX, height * 0.9f)) }
        assertEquals(listOf("c4"), opened)
        compose.onRoot().performTouchInput { click(Offset(width * 0.05f, height * 0.12f)) }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(null, sky.scene.focusedCard)
    }

    private fun assertNotNullPoint(sky: SkyState, i: Int): Pair<Float, Float> {
        val size = compose.onRoot().fetchSemanticsNode().size
        val point = sky.painter.starPoint(sky.scene, i, size.width.toFloat(), size.height.toFloat())
        assertNotNull(point)
        return point!!
    }
}

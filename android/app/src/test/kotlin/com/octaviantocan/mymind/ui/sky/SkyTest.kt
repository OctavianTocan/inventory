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
import kotlin.math.abs

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
        // The find shows whole: inside the screen with a margin, above its title, every point of light inside its box.
        val view = sky.scene.find!!
        assertTrue(view.boxW <= sky.scene.viewW - 2 * SkyScene.FIND_MARGIN + 0.01f)
        assertTrue(SkyScene.FIND_Y + view.boxH / 2f < Proto.H - 260f)
        assertTrue(view.count > 200)
        for (k in 0 until view.count) {
            assertTrue(abs(view.tx[k]) <= view.boxW / 2f && abs(view.ty[k] - SkyScene.FIND_Y) <= view.boxH / 2f)
        }

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

    @Test
    fun aVideoFindShowsWholeUnderItsStar() {
        val sky = SkyState()
        compose.setPhoneContent(true) {
            Box(Modifier.fillMaxSize()) {
                SkyBackdrop(sky, skyPlan(SKY, SKY.stars.map { it.id }.toSet(), emptySet()), FINDS, paused = false, onOpenFind = {}, onSeen = {}, onLookUp = {})
                SkyOverlay(sky)
            }
        }
        compose.mainClock.advanceTimeByFrame()
        sky.lookUp()
        compose.mainClock.advanceTimeBy(3000)
        // After the sky last composed, which hands the scene the library's pictures.
        val frame = videoFrame()
        sky.scene.picture = { frame }
        val (x, y) = assertNotNullPoint(sky, 3)
        compose.onRoot().performTouchInput { click(Offset(x, y)) }
        compose.mainClock.advanceTimeBy(3200)
        compose.onRoot().captureRoboImage("screenshots/sky_10_video_find.png")
        val view = sky.scene.find!!
        assertEquals(16f / 9f, view.boxW / view.boxH, 0.01f)
        assertEquals(sky.scene.viewW - 2 * SkyScene.FIND_MARGIN, view.boxW, 0.01f)
    }

    // A 16:9 frame like a video's thumbnail: a dusk sky, a sun, a dark hill and a white house.
    private fun videoFrame(): android.graphics.Bitmap {
        val bitmap = android.graphics.Bitmap.createBitmap(480, 270, android.graphics.Bitmap.Config.ARGB_8888)
        val canvas = android.graphics.Canvas(bitmap)
        val paint = android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG)
        paint.shader = android.graphics.LinearGradient(0f, 0f, 0f, 270f, 0xFF2B4C8C.toInt(), 0xFFF2A65A.toInt(), android.graphics.Shader.TileMode.CLAMP)
        canvas.drawRect(0f, 0f, 480f, 270f, paint)
        paint.shader = null
        paint.color = 0xFFFFF4D6.toInt()
        canvas.drawCircle(340f, 120f, 46f, paint)
        paint.color = 0xFF1C1F2A.toInt()
        canvas.drawOval(-120f, 190f, 600f, 420f, paint)
        paint.color = 0xFFF5F5F0.toInt()
        canvas.drawRect(90f, 150f, 190f, 215f, paint)
        val roof = android.graphics.Path().apply { moveTo(80f, 152f); lineTo(140f, 110f); lineTo(200f, 152f); close() }
        canvas.drawPath(roof, paint)
        return bitmap
    }

    @Test
    fun aFindFitsTheScreenInItsOwnShape() {
        // Tavi's Galaxy Z Fold cover screen, 1080 by 2520 pixels: 362 units wide. A video's frame keeps its shape.
        val (w, h) = findBox(16f / 9f, 362f)
        assertEquals(362f - 2 * SkyScene.FIND_MARGIN, w, 0.01f)
        assertEquals(w * 9f / 16f, h, 0.01f)
        // Open, the Fold is wide: a square find stops at its height.
        val (sw, sh) = findBox(1f, 937f)
        assertEquals(SkyScene.FIND_MAX_H, sw, 0.01f)
        assertEquals(SkyScene.FIND_MAX_H, sh, 0.01f)
        // A tall picture on a phone stops at its height, narrower than the screen.
        val (tw, th) = findBox(9f / 16f, 390f)
        assertEquals(SkyScene.FIND_MAX_H, th, 0.01f)
        assertTrue(tw < 390f - 2 * SkyScene.FIND_MARGIN)
        assertEquals(1f, boxEdge(0.5f, 0f), 0.001f)
        assertEquals(0f, boxEdge(0f, 0f), 0.001f)
    }

    @Test
    fun finishingAConstellationKeepsItInItsPlaceAndTheNextGathersInTheMiddle() {
        val at = "2026-11-06T10:00:00Z"
        val one = Constellation(1, "ship", "The Voyager", "c0", ids("c"), lit = ids("c") - "c5")
        var today by mutableStateOf(Sky(day = "2026-11-06", active = one, stars = ids("c").drop(1).dropLast(1).map { SkyStar(it, at) }))
        var seen by mutableStateOf(today.stars.map { it.id }.toSet())
        val sky = SkyState()
        var completed = 0
        compose.setPhoneContent(true) {
            Box(Modifier.fillMaxSize()) {
                SkyBackdrop(sky, skyPlan(today, seen, emptySet()), FINDS, paused = false,
                    onOpenFind = { id -> today = withVisits(today, listOf(SkyStar(id, at)), at) },
                    onSeen = { seen = seen + it }, onLookUp = {}, onCompleted = { completed++ })
                SkyOverlay(sky)
            }
        }
        compose.mainClock.advanceTimeByFrame()
        sky.lookUp()
        compose.mainClock.advanceTimeBy(3000)

        // Open the last dark star's find from the sky, then come back from it.
        val (x, y) = assertNotNullPoint(sky, 5)
        compose.onRoot().performTouchInput { click(Offset(x, y)) }
        compose.mainClock.advanceTimeBy(3000)
        assertEquals("c5", sky.scene.focusedCard)
        compose.onRoot().performTouchInput { click(Offset(centerX, height * 0.9f)) }
        compose.mainClock.advanceTimeBy(100)
        assertEquals("finished on the phone at once", null, today.active)
        compose.runOnIdle { sky.scene.returned() }
        compose.mainClock.advanceTimeBy(14000)
        assertEquals(1, completed)
        assertEquals("the finished one stays in its place, not back in the middle", null, sky.scene.active)
        compose.onRoot().captureRoboImage("screenshots/sky_08_finished.png")

        // The server's next constellation gathers in the middle.
        today = today.copy(active = Constellation(5, "lantern", "The Lantern", "n0", ids("n"), lit = listOf("n0")))
        compose.mainClock.advanceTimeBy(2500)
        compose.onRoot().captureRoboImage("screenshots/sky_09_next_gathering.png")
        compose.mainClock.advanceTimeBy(3000)
        assertEquals(5L, sky.scene.active?.id)
    }

    @Test
    fun aTappedFinishedConstellationComesBackToTheMiddleAndBackReturnsItToItsPlace() {
        val sky = SkyState()
        compose.setPhoneContent(true) {
            Box(Modifier.fillMaxSize()) {
                SkyBackdrop(sky, skyPlan(SKY, SKY.stars.map { it.id }.toSet(), emptySet()), FINDS, paused = false, onOpenFind = {}, onSeen = {}, onLookUp = {})
                SkyOverlay(sky)
            }
        }
        compose.mainClock.advanceTimeByFrame()
        sky.lookUp()
        compose.mainClock.advanceTimeBy(3000)
        val scene = sky.scene

        // The Dreamer finished last; a tap on it in its place brings it forward, its name gold, the Voyager stepping back.
        val size = compose.onRoot().fetchSemanticsNode().size
        val (x, y) = sky.painter.finishedPoint(scene, 1, size.width.toFloat(), size.height.toFloat())!!
        compose.onRoot().performTouchInput { click(Offset(x, y)) }
        compose.mainClock.advanceTimeBy(2400)
        assertEquals(3L, scene.middle?.id)
        assertEquals("The Dreamer", scene.labelName)
        assertEquals(1f, scene.labelDone)
        assertEquals(1f, scene.middle!!.scale, 0.001f)
        assertEquals(0f, scene.active!!.dim, 0.001f)
        compose.onRoot().captureRoboImage("screenshots/sky_11_finished_one_back.png")

        // Its stars work as the middle one's: a tap moves in on a find, and nothing lights.
        val (sx, sy) = assertNotNullPoint(sky, 2)
        compose.onRoot().performTouchInput { click(Offset(sx, sy)) }
        compose.mainClock.advanceTimeBy(3200)
        assertEquals("e2", scene.focusedCard)

        // Back moves out to the whole sky, then sends the Dreamer to its place and brings the Voyager forward again.
        compose.runOnIdle { scene.back() }
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(null, scene.focusedCard)
        compose.runOnIdle { scene.back() }
        compose.mainClock.advanceTimeBy(2400)
        assertEquals(null, scene.visiting)
        assertEquals(1L, scene.middle?.id)
        assertEquals("The Voyager", scene.labelName)
        assertEquals(0f, scene.labelDone)
        assertEquals(1f, scene.active!!.dim, 0.001f)
        assertEquals(SkyScene.SLOT_SCALE, scene.cons.first { it.id == 3L }.scale, 0.001f)
        assertTrue("still in the sky", sky.lookingUp)
    }

    @Test
    fun aSwipeSidewaysGoesBackThroughEveryFinishedConstellationAndForwardAgain() {
        // Eight finished, two more than have places in the sky.
        val figures = listOf("cup", "book", "mountain", "lantern", "camera", "harp", "tree", "house")
        val done = figures.mapIndexed { k, figure ->
            val cards = (0 until 6).map { "o${k}_$it" }
            Constellation(10L + k, figure, "Old $k", cards[0], cards, lit = cards, completed = "2026-10-${10 + k}T10:00:00Z")
        }
        var today by mutableStateOf(SKY.copy(done = done))
        val sky = SkyState()
        compose.setPhoneContent(true) {
            Box(Modifier.fillMaxSize()) {
                SkyBackdrop(sky, skyPlan(today, today.stars.map { it.id }.toSet(), emptySet()), FINDS, paused = false, onOpenFind = {}, onSeen = {}, onLookUp = {})
                SkyOverlay(sky)
            }
        }
        compose.mainClock.advanceTimeByFrame()
        sky.lookUp()
        compose.mainClock.advanceTimeBy(3000)
        val scene = sky.scene
        assertEquals(7, scene.cons.size)
        fun swipe(dx: Float) {
            compose.onRoot().performTouchInput {
                down(Offset(centerX, centerY))
                repeat(10) { moveBy(Offset(dx, 0f)); advanceEventTime(16) }
                up()
            }
            compose.mainClock.advanceTimeBy(2400)
        }

        // Right goes back in time: the newest finished first, down to the oldest, which had no place and came in.
        swipe(30f)
        assertEquals(17L, scene.middle?.id)
        // A new plan waits while one is brought back.
        today = today.copy(active = Constellation(5, "coin", "The Coin", "n0", ids("n"), lit = listOf("n0")))
        repeat(7) { swipe(30f) }
        assertEquals(10L, scene.middle?.id)
        assertEquals("Old 0", scene.labelName)
        assertEquals(1L, scene.active?.id)
        compose.onRoot().captureRoboImage("screenshots/sky_12_oldest_one_back.png")
        swipe(30f)
        assertEquals("at the oldest it stays", 10L, scene.middle?.id)
        assertEquals(0f, scene.middle!!.x, 0.001f)

        // Left comes forward again, to the one forming; the old ones with no place leave the sky.
        repeat(8) { swipe(-30f) }
        assertEquals(null, scene.visiting)
        assertTrue(scene.cons.none { it.id == 10L || it.id == 11L })
        // Then the plan that waited shows: the server's next constellation gathers in the middle.
        compose.mainClock.advanceTimeBy(6000)
        assertEquals(5L, scene.middle?.id)
    }

    private fun assertNotNullPoint(sky: SkyState, i: Int): Pair<Float, Float> {
        val size = compose.onRoot().fetchSemanticsNode().size
        val point = sky.painter.starPoint(sky.scene, i, size.width.toFloat(), size.height.toFloat())
        assertNotNull(point)
        return point!!
    }
}

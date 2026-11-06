package com.octaviantocan.mymind.ui.sky

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onRoot
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
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.SidePages
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.shell.rememberSidePages
import com.octaviantocan.mymind.ui.theme.PageScheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

private fun ids(prefix: String) = (0 until 6).map { "$prefix$it" }

private val SKY = Sky(
    day = "2026-11-06",
    back = listOf("c3", "x1"),
    active = Constellation(1, "ship", "The Voyager", "c0", ids("c"), lit = listOf("c0", "c1", "c2")),
    done = listOf(Constellation(2, "lantern", "The Lantern", "d0", ids("d"), lit = ids("d"), completed = "2026-10-20T10:00:00Z")),
    stars = (listOf("c1", "c2") + ids("d").drop(1)).map { SkyStar(it, "2026-11-01T10:00:00Z") },
)

private val FINDS = (ids("c") + ids("d") + "x1").associateWith { testCard(it, title = "Find $it") }

@RunWith(AndroidJUnit4::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class SkyShellTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun theSkyTakesTheWholeScreenAndOwnsItsSwipes() {
        val today = BackTodayState(SKY, listOf(FINDS.getValue("c3"), FINDS.getValue("x1")), FINDS, SKY.stars.map { it.id }.toSet())
        val sky = SkyState()
        var page by mutableIntStateOf(1)
        var outer: com.octaviantocan.mymind.ui.shell.ShellImmersion? = null
        compose.setPhoneContent(true) {
            ShellFrame(Tab.Everything, {}, PageScheme.Dark) {
                outer = com.octaviantocan.mymind.ui.shell.LocalShellImmersion.current
                val pages = rememberSidePages(page, 3) { page = it }
                SidePages(pages, 3) { shown ->
                    if (shown == 1) {
                        EverythingContent(
                            feed = FeedState(cards = (1..12).map { testCard("g$it", title = "Gallery $it") }, endReached = true), searching = false,
                            searchResults = null, typeChips = emptyList(), filter = null, suggestions = emptyList(),
                            onChoose = {}, onRemoveFilter = {}, onLoadMore = {}, onLoadMoreResults = {},
                            onRetry = {}, onOpenCard = {}, onOpenSettings = {}, onAdd = {},
                            backToday = today, sky = sky,
                        )
                    } else {
                        Box(Modifier.fillMaxSize())
                    }
                }
            }
        }
        compose.mainClock.advanceTimeBy(100)
        compose.onRoot().captureRoboImage("screenshots/sky_shell_00_page.png")
        compose.onNodeWithContentDescription("Search").assertExists()
        compose.runOnIdle { sky.lookUp() }
        compose.mainClock.advanceTimeBy(4000)
        compose.onRoot().captureRoboImage("screenshots/sky_shell_01_sky.png")
        // The gallery under the sky also tells the shell how far it is in reels; the sky still takes the whole screen.
        assertEquals("the bar slides away under the sky", 1f, outer!!.amount(), 0.001f)
        assertNotNull("the status bar takes the sky's colour", outer!!.strip())

        // A sideways swipe in the sky moves between its stars, never to the next page, wherever it starts.
        for (y in listOf(0.5f, 0.93f)) compose.onRoot().performTouchInput {
            down(Offset(width * 0.8f, height * y))
            repeat(20) { moveBy(Offset(-30f, 0f)); advanceEventTime(16) }
            up()
        }
        compose.mainClock.advanceTimeBy(1500)
        compose.onRoot().captureRoboImage("screenshots/sky_shell_02_swiped.png")
        assertEquals("a swipe in the sky stays on the page", 1, page)
    }
}

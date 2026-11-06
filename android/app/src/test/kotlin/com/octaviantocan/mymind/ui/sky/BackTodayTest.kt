package com.octaviantocan.mymind.ui.sky

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Constellation
import com.octaviantocan.mymind.data.FeedMemory
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.LibraryMemory
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.data.SkyStar
import com.octaviantocan.mymind.ui.PHONE_QUALIFIERS
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config

private const val DAY = "2026-11-05"

private val FINDS = listOf(testCard("old1", title = "Harbour at dusk"), testCard("old2", title = "Ferry times"), testCard("old3", title = "Lighthouse"))

// A library whose sky is [sky], or fails as offline when null, and which keeps the visits it is sent.
private class SkyLibrary(var sky: Sky?, val fake: FakeLibrary = FakeLibrary(cardsById = FINDS.associateBy { it.id })) : Library by fake {
    val visits = mutableListOf<String>()

    override suspend fun sky(day: String): ApiResult<Sky> = sky?.let { ApiResult.Ok(it) } ?: ApiResult.Failed(ApiFailure.Network)

    override suspend fun visit(cardId: String): ApiResult<Unit> = ApiResult.Ok(Unit).also { visits += cardId }
}

private class SkyMemory(var kept: String? = null) : LibraryMemory {
    override fun feed(name: String): FeedMemory = error("not used")

    override suspend fun scheme(): String? = null

    override suspend fun saveScheme(name: String) {}

    override suspend fun sky(): String? = kept

    override suspend fun saveSky(json: String) {
        kept = json
    }
}

private val TODAY = Sky(
    day = DAY,
    back = listOf("old1", "old2", "old3"),
    active = Constellation(id = 1, figure = "ship", name = "The Voyager", seed = "new", cards = listOf("new", "old1", "old2"), lit = listOf("new")),
)

@RunWith(AndroidJUnit4::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class BackTodayTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun todaysFindsComeFromTheSkyAndAVisitLightsTheirStar() = runTest {
        val library = SkyLibrary(TODAY)
        val memory = SkyMemory()
        val model = BackTodayModel(this, library, memory) { DAY }
        model.load()
        advanceUntilIdle()
        assertEquals(listOf("old1", "old2", "old3"), model.state.value.cards.map { it.id })
        assertTrue(model.state.value.uncovered.isEmpty())

        model.uncover(FINDS[0])
        advanceUntilIdle()
        assertEquals(setOf("old1"), model.state.value.uncovered)
        assertEquals(listOf("new", "old1"), model.state.value.sky?.active?.lit)
        assertEquals(listOf("old1"), library.visits)

        // The sky reads the cards of its stars that the library still holds.
        model.loadFinds()
        advanceUntilIdle()
        assertEquals(setOf("old1", "old2"), model.state.value.finds.keys)

        // Offline the next launch the same day shows the kept band, the find still uncovered.
        val offline = BackTodayModel(this, SkyLibrary(null), memory) { DAY }
        offline.load()
        advanceUntilIdle()
        assertEquals(listOf("old1", "old2", "old3"), offline.state.value.cards.map { it.id })
        assertEquals(setOf("old1"), offline.state.value.uncovered)

        // A day later the kept band is yesterday's, so nothing shows without the server.
        val tomorrow = BackTodayModel(this, SkyLibrary(null), memory) { "2026-11-06" }
        tomorrow.load()
        advanceUntilIdle()
        assertTrue(tomorrow.state.value.cards.isEmpty())
    }

    @Test
    fun aVisitNotYetSentStaysUncoveredWhenTheServerAnswersAgain() = runTest {
        val library = SkyLibrary(TODAY)
        val model = BackTodayModel(this, library, null) { DAY }
        model.load()
        advanceUntilIdle()
        model.uncover(FINDS[1])
        model.load()
        advanceUntilIdle()
        assertEquals(setOf("old2"), model.state.value.uncovered)
        assertEquals(listOf(SkyStar("old2")), model.state.value.sky?.stars?.map { it.copy(visited = null) })
    }

    @Test
    fun lightingTheLastStarFinishesTheConstellationUntilTheServerGivesTheNext() = runTest {
        val library = SkyLibrary(TODAY)
        val model = BackTodayModel(this, library, null) { DAY }
        model.load()
        advanceUntilIdle()
        model.uncover(FINDS[0])
        model.uncover(FINDS[1])
        advanceUntilIdle()
        // Its last star lit: it is finished on the phone at once, as the server finishes it.
        assertEquals(null, model.state.value.sky?.active)
        assertEquals(listOf(1L), model.state.value.sky?.done?.map { it.id })

        // A server that has not had the visits yet still answers it as forming; the phone keeps it finished.
        model.reload()
        advanceUntilIdle()
        assertEquals(null, model.state.value.sky?.active)
        assertEquals(listOf(1L), model.state.value.sky?.done?.map { it.id })

        // Once it has them, it answers with the next one.
        val finished = TODAY.active!!.copy(lit = TODAY.active!!.cards, completed = "2026-11-05T10:00:00Z")
        val next = Constellation(id = 2, figure = "lantern", name = "The Lantern", seed = "newer", cards = listOf("newer", "old3"), lit = listOf("newer"))
        library.sky = TODAY.copy(active = next, done = listOf(finished), stars = listOf(SkyStar("old1"), SkyStar("old2")))
        model.reload()
        advanceUntilIdle()
        assertEquals(2L, model.state.value.sky?.active?.id)
        assertEquals(listOf(1L), model.state.value.sky?.done?.map { it.id })
    }

    @Test
    fun aFindOpenedFromBackTodayWaitsOnTheStarButtonUntilItsStarLightsInTheSky() = runTest {
        val library = SkyLibrary(TODAY.copy(stars = listOf(SkyStar("old3"))))
        val model = BackTodayModel(this, library, SkyMemory(), today = { DAY })
        model.load()
        advanceUntilIdle()
        // On the first run every star already in the sky counts as seen.
        assertEquals(setOf("old3"), model.state.value.seen)
        assertEquals(0, model.state.value.waiting)

        model.uncover(FINDS[0])
        advanceUntilIdle()
        assertEquals("its star flies only once the find closes", setOf("old1"), model.state.value.arriving)
        assertEquals(0, model.state.value.waiting)
        model.landed("old1")
        assertEquals(1, model.state.value.waiting)
        model.markSeen("old1")
        advanceUntilIdle()
        assertEquals(0, model.state.value.waiting)

        // One opened from its own star never flies; it lights where it is.
        model.uncover(FINDS[1], arriving = false)
        advanceUntilIdle()
        assertTrue(model.state.value.arriving.isEmpty())
    }

    @Test
    fun aHologramFindRestoresFromTheTapThenOpens() {
        val state = BackTodayState(TODAY.copy(stars = listOf(SkyStar("old3"))), FINDS)
        val uncovered = mutableListOf<String>()
        val opened = mutableListOf<String>()
        var shakes = 0
        compose.mainClock.autoAdvance = false
        compose.setContent {
            MymindTheme(dark = false) { BackTodayBand(state, { card, _ -> uncovered += card.id }, { opened += it.id }, onShake = { shakes++ }) }
        }
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithText("Back today").assertIsDisplayed()
        compose.onAllNodesWithContentDescription("A find from your past, flickering as a hologram").assertCountEquals(2)
        compose.onAllNodesWithContentDescription("Harbour at dusk").assertCountEquals(0)
        compose.onNodeWithContentDescription("Lighthouse").assertIsDisplayed()

        compose.onAllNodesWithContentDescription("A find from your past, flickering as a hologram")[0].performClick()
        compose.mainClock.advanceTimeByFrame()
        assertEquals(listOf("old1"), uncovered)
        assertTrue("the find opens once it has cleared", opened.isEmpty())
        compose.mainClock.advanceTimeBy(1500)
        assertEquals(listOf("old1"), opened)
        assertEquals(1, shakes)
    }
}

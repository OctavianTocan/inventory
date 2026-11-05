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
    fun aCoveredFindIsHiddenUntilTappedThenOpensAsItsLeavesBlowAway() {
        val state = BackTodayState(TODAY.copy(stars = listOf(SkyStar("old3"))), FINDS)
        val uncovered = mutableListOf<String>()
        val opened = mutableListOf<String>()
        compose.mainClock.autoAdvance = false
        compose.setContent { MymindTheme(dark = false) { BackTodayBand(state, { uncovered += it.id }, { opened += it.id }) } }
        compose.mainClock.advanceTimeByFrame()
        compose.onNodeWithText("Back today").assertIsDisplayed()
        compose.onAllNodesWithContentDescription("A find from your past, under fallen leaves").assertCountEquals(2)
        compose.onAllNodesWithContentDescription("Harbour at dusk").assertCountEquals(0)
        compose.onNodeWithContentDescription("Lighthouse").assertIsDisplayed()

        compose.onAllNodesWithContentDescription("A find from your past, under fallen leaves")[0].performClick()
        compose.mainClock.advanceTimeByFrame()
        assertEquals(listOf("old1"), uncovered)
        assertTrue("the find opens while its leaves fly", opened.isEmpty())
        compose.mainClock.advanceTimeBy(OPEN_AFTER_MS + 32)
        assertEquals(listOf("old1"), opened)
    }
}

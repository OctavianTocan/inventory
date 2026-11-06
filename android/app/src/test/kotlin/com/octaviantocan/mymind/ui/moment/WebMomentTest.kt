package com.octaviantocan.mymind.ui.moment

import androidx.compose.ui.geometry.Rect
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Space
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.util.Locale

/**
 * share-moment-shooting-star: the app's side of the prototype's own moment. What the page says moves the moment on, the
 * page's sounds become the app's beats, and the tag step turns the same way round as the page's card.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36])
class WebMomentTest {
    private val sent = ArrayList<String>()
    private val web = WebMoment(null, null).also { it.sink = { s -> sent += s }; it.density = 2f }
    private val find = MomentFind(title = "Leica M6", kind = "example.com", icon = R.drawable.ic_link, tags = listOf("35mm"), number = 1486)

    @Test
    fun itStartsOnceThePageIsReady() {
        web.start(findJson(find, 1485, 1, null))
        assertTrue("Nothing runs before the page is ready", sent.isEmpty())
        web.receive("ready", "{}")
        assertEquals(WebStatus.Playing, web.status)
        assertTrue(sent.single().startsWith("InvMoment.start("))
    }

    @Test
    fun aPageThatFailsBeforePlayingGivesWayButNotOnceItPlays() {
        web.receive("fail", """{"m":"no WebGL"}""")
        assertEquals(WebStatus.Failed, web.status)

        val other = WebMoment(null, null).also { it.sink = {} }
        other.receive("ready", "")
        other.start(findJson(find, null, 1, null))
        other.receive("fail", "{}")
        assertEquals(WebStatus.Playing, other.status)
    }

    @Test
    fun theCardHandsOverItsRectThenComesBackWithWhatWasChosen() {
        web.receive("ready", "")
        web.start(findJson(find, 1485, 1, null))
        web.receive("tags", """{"rect":{"x":100,"y":200,"w":40,"h":60},"turnMs":230}""")
        assertEquals(StarPhase.Settled, web.phase)
        assertEquals(Rect(160f, 340f, 240f, 460f), web.card)

        web.tagsClosed(null, emptyList())
        assertEquals("It waits for the tag step to close first", StarPhase.Settled, web.phase)
        web.closeTags()
        assertTrue(web.closing)
        web.tagsClosed(Space("s1", "Cameras", "#4f8a5b", count = 4), listOf("35mm", "film"))
        assertEquals(StarPhase.Leaving, web.phase)
        val done = sent.last()
        assertTrue(done.startsWith("InvMoment.tagsDone("))
        val chosen = Json.parseToJsonElement(done.removePrefix("InvMoment.tagsDone(").removeSuffix(")")).jsonObject
        assertEquals("Cameras", chosen["space"]!!.jsonObject["label"]!!.jsonPrimitive.content)
        assertEquals(2, chosen["tags"]!!.jsonArray.size)

        web.receive("done", """{"skipped":false}""")
        assertEquals(StarPhase.Done, web.phase)
    }

    @Test
    fun aTapSkipsItAll() {
        web.receive("skip", "")
        assertTrue(web.skipped)
        assertEquals(StarPhase.Leaving, web.phase)
        web.receive("done", """{"skipped":true}""")
        assertEquals(StarPhase.Done, web.phase)
    }

    @Test
    fun theRarityIsKeptFromTheStarsBreakOff() {
        web.rarity = Rarity.Legendary
        assertEquals("InvMoment.rarity(5)", sent.last())
        web.receive("rarity", """{"r":3}""")
        assertEquals(Rarity.Rare, web.shownRarity)
    }

    @Test
    fun thePagesSoundsBecomeTheAppsBeats() {
        assertEquals(Cue.Ting(2), cueOf("ting", 2.0, 5))
        assertEquals(Cue.Meteor(1.9f), cueOf("meteor", 1.9, 5))
        assertEquals(Cue.Boom(0.62f), cueOf("boom", 0.62, 3))
        assertEquals(Cue.Chime(Rarity.Legendary), cueOf("chime", 5.0, 5))
        assertEquals("All five stars of a Legendary find stamp in", (0..4).map { Cue.Star(it) }, (0..4).map { cueOf("tick", 1300.0 + it * 200, 5) })
        assertEquals(Cue.Count, cueOf("tick", 1800.0, 5))
        assertNull(cueOf("tick", 900.0, 5))
        assertNull(cueOf("crack", null, 5))
    }

    @Test
    fun theFindReachesThePageAsTheCardShowsIt() {
        val was = Locale.getDefault()
        Locale.setDefault(Locale.UK)
        try {
            val json = findJson(find.copy(tags = (1..20).map { "t$it" }), 1485, 2, null)
            assertEquals("No. 1,486", json["no"]!!.jsonPrimitive.content)
            assertEquals("link", json["glyph"]!!.jsonPrimitive.content)
            assertEquals(1485, json["count"]!!.jsonPrimitive.int)
            assertEquals(12, json["tags"]!!.jsonArray.size)
            assertEquals("null", json["space"].toString())
        } finally {
            Locale.setDefault(was)
        }
    }

    /**
     * The page turns its card to fl0 + 90°, and past edge on its right half comes nearer (three.js turns it about y,
     * anticlockwise from above). The tag step carries on from there, so at [TURN_SIGN] × 45° its right edge is taller.
     * A layer turns as a view does (both are a RenderNode), so this reads a view's own matrix.
     */
    @Test
    fun theTagStepTurnsOnTheWayThePagesCardTurned() {
        val context = androidx.test.core.app.ApplicationProvider.getApplicationContext<android.content.Context>()
        val view = android.view.View(context)
        view.layout(0, 0, 200, 284)
        view.pivotX = 100f; view.pivotY = 142f
        view.cameraDistance = 14f * 72f
        view.rotationY = TURN_SIGN * 45f
        val edges = floatArrayOf(0f, 0f, 0f, 284f, 200f, 0f, 200f, 284f)
        view.matrix.mapPoints(edges)
        val left = edges[3] - edges[1]
        val right = edges[7] - edges[5]
        assertTrue("The right edge is taller (left $left, right $right)", right > left * 1.05f)
    }
}

package com.octaviantocan.mymind.ui.menu

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Edit
import com.octaviantocan.mymind.data.edited
import com.octaviantocan.mymind.ui.library.FakeLibrary
import com.octaviantocan.mymind.ui.library.storedRatio
import com.octaviantocan.mymind.ui.library.turn
import com.octaviantocan.mymind.ui.library.pinned
import com.octaviantocan.mymind.ui.library.testCard
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.ByteArrayOutputStream

class CardActionsTest {
    private val saved = mutableMapOf<String, ByteArray>()

    private val saver = FileSaver { name, _, write ->
        val out = ByteArrayOutputStream()
        write(out).also { if (it is ApiResult.Ok) saved[name] = out.toByteArray() }
    }

    private fun actionsOf(items: List<CardMenuItem>) = items.map { it.action }

    @Test
    fun articleWithSourceOutsideASpace() {
        val items = cardMenuItems(testCard("a", type = "Article", url = "https://example.com/a"), spaceId = null)
        assertEquals(
            listOf(CardMenuAction.ViewSource, CardMenuAction.AddTags, CardMenuAction.AddToSpace, CardMenuAction.Pin, CardMenuAction.DeleteCard),
            actionsOf(items),
        )
        assertTrue(items.none { it.locked })
    }

    @Test
    fun aCardWithPageTextOpensReaderModeFirst() = runTest {
        val prose = Json.parseToJsonElement("""{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Words."}]}]}""")
        val article = testCard("a", type = "Article").copy(prose = prose)
        assertEquals(CardMenuItem(CardMenuAction.ReaderMode), cardMenuItems(article, spaceId = null).first())
        assertTrue(cardMenuItems(testCard("a", type = "Article", flags = 1 shl 1), null).none { it.action == CardMenuAction.ReaderMode })
        assertEquals(CardMenuOutcome.Read(article), CardActions(FakeLibrary(), saver).perform(CardMenuItem(CardMenuAction.ReaderMode), article, null))
    }

    @Test
    fun noteInASpaceThatIsPinned() {
        val items = cardMenuItems(testCard("n", type = "Note", flags = 512), spaceId = "s1")
        assertEquals(
            listOf(CardMenuAction.Copy, CardMenuAction.AddTags, CardMenuAction.RemoveFromSpace, CardMenuAction.Unpin, CardMenuAction.DeleteCard),
            actionsOf(items),
        )
    }

    @Test
    fun imageGetsSameVibeDownloadAndCopyButNotForSvg() {
        val png = testCard("i", type = "Image", blob = Blob(type = "image/png", path = "u/i.png"))
        assertEquals(
            listOf(CardMenuItem(CardMenuAction.SameVibe, locked = true), CardMenuItem(CardMenuAction.Download), CardMenuItem(CardMenuAction.Copy)),
            cardMenuItems(png, null).filter { it.action in setOf(CardMenuAction.SameVibe, CardMenuAction.Download, CardMenuAction.Copy) },
        )
        assertFalse(CardMenuItem(CardMenuAction.SameVibe, locked = true) in cardMenuItems(png, null, setOf("sameVibe")))
        val svg = testCard("i", type = "Image", blob = Blob(type = "image/svg+xml"))
        assertTrue(cardMenuItems(svg, null).none { it.action == CardMenuAction.Copy })
    }

    @Test
    fun writesGoToTheLibraryAndReportTheirOutcome() = runTest {
        val library = FakeLibrary()
        val actions = CardActions(library, saver)
        val card = testCard("c", url = "https://example.com/c")

        assertEquals(CardMenuOutcome.OpenUrl("https://example.com/c"), actions.perform(CardMenuItem(CardMenuAction.ViewSource), card, null))
        val note = testCard("n", type = "Note", note = "text of n")
        assertEquals(CardMenuOutcome.Copy("text of n"), actions.perform(CardMenuItem(CardMenuAction.Copy), note, null))
        val pinned = actions.perform(CardMenuItem(CardMenuAction.Pin), card, null) as CardMenuOutcome.PinChanged
        assertTrue(pinned.card.pinned)
        val unpinned = actions.perform(CardMenuItem(CardMenuAction.Unpin), pinned.card, null) as CardMenuOutcome.PinChanged
        assertFalse(unpinned.card.pinned)
        assertEquals(CardMenuOutcome.RemovedFromSpace("s1", "c"), actions.perform(CardMenuItem(CardMenuAction.RemoveFromSpace), card, "s1"))
        assertEquals(CardMenuOutcome.Deleted("c"), actions.perform(CardMenuItem(CardMenuAction.DeleteCard), card, null))
        assertEquals(listOf("pin:c:true", "pin:c:false", "unspace:s1:c", "trash:c"), library.calls)
    }

    @Test
    fun rotateTurnsAPictureAQuarterAtATimeAndAFullTurnMakesItUprightAgain() = runTest {
        val library = FakeLibrary()
        val actions = CardActions(library, saver)
        var photo = testCard("p", type = "Image", blob = Blob(type = "image/jpeg", path = "u/p.jpg", width = 400, height = 300))
        assertTrue(CardMenuItem(CardMenuAction.Rotate) in cardMenuItems(photo, null))
        assertEquals(4f / 3f, storedRatio(photo))
        val turns = (1..4).map {
            photo = (actions.perform(CardMenuItem(CardMenuAction.Rotate), photo, null) as CardMenuOutcome.Rotated).card
            photo.rotation
        }
        assertEquals(listOf(90, 180, 270, null), turns)
        assertEquals(listOf("rotate:p:90", "rotate:p:180", "rotate:p:270", "rotate:p:0"), library.calls)
        assertEquals("a picture turned sideways stands the other way", 3f / 4f, storedRatio(photo.copy(rotation = 90)))
        assertEquals(270, photo.copy(rotation = -90).turn)
        assertEquals(180, photo.edited(Edit.Rotate("p", 180))?.rotation)
        assertNull(photo.copy(rotation = 90).edited(Edit.Rotate("p", 0))?.rotation)
    }

    @Test
    fun onlyPicturesTheyKeptCanBeRotated() {
        val article = testCard("a", type = "Article", url = "https://example.com/a")
        assertTrue(cardMenuItems(article, null).none { it.action == CardMenuAction.Rotate })
        val video = testCard("v", type = "Video", blob = Blob(type = "video/mp4", path = "u/v.mp4"))
        assertTrue(cardMenuItems(video, null).none { it.action == CardMenuAction.Rotate })
    }

    @Test
    fun downloadSavesTheOriginalFileUnderTheCardId() = runTest {
        val library = FakeLibrary(media = mapOf("u/i.png" to byteArrayOf(1, 2, 3)))
        val actions = CardActions(library, saver)
        val image = testCard("i", type = "Image", blob = Blob(type = "image/png", path = "u/i.png"))

        assertEquals(CardMenuOutcome.Downloaded("i.png"), actions.perform(CardMenuItem(CardMenuAction.Download), image, null))
        assertEquals(listOf<Byte>(1, 2, 3), saved["i.png"]?.toList())
        val missing = image.copy(id = "m", blob = Blob(type = "image/png", path = "u/m.png"))
        assertEquals(CardMenuOutcome.Failed(ApiFailure.Server(404)), actions.perform(CardMenuItem(CardMenuAction.Download), missing, null))
        assertFalse("m.png" in saved)
        assertEquals(listOf("media:u/i.png", "media:u/m.png"), library.calls)
    }

    @Test
    fun aPictureCopiesAndSavesItsPictureAndALinkWithACoverDoesNot() = runTest {
        val actions = CardActions(FakeLibrary(), saver)
        val photo = testCard("p", type = "Image", blob = Blob(type = "image/png", path = "u/p.png"))
        assertEquals(CardMenuOutcome.CopyPicture(photo), actions.perform(CardMenuItem(CardMenuAction.Copy), photo, null))
        // A picture saved from the web keeps no file, so it is saved as drawn.
        val fromWeb = testCard("w", type = "Image", url = "https://example.com/w.jpg").copy(cover = Cover("https://example.com/w.jpg", 400, 300))
        assertTrue(CardMenuItem(CardMenuAction.Download) in cardMenuItems(fromWeb, null))
        assertEquals(CardMenuOutcome.SavePicture(fromWeb), actions.perform(CardMenuItem(CardMenuAction.Download), fromWeb, null))
        val article = testCard("a", type = "Article", url = "https://example.com/a").copy(cover = Cover("https://example.com/a.jpg", 400, 300))
        assertTrue(cardMenuItems(article, null).none { it.action == CardMenuAction.Copy || it.action == CardMenuAction.Download })
    }

    @Test
    fun manyFindsCopyTheirWordsAndLinksOneAfterAnother() {
        val note = testCard("n", type = "Note", note = "my words")
        val link = testCard("l", type = "Article", title = "A page", url = "https://example.com/a")
        val photo = testCard("p", type = "Image", blob = Blob(type = "image/png", path = "u/p.png"))
        assertEquals("my words\n\nA page\nhttps://example.com/a", manyWords(listOf(note, link, photo.copy(title = null))))
        assertNull(manyWords(listOf(photo.copy(title = null))))
    }

    @Test
    fun downloadNamesAndCopyText() {
        assertEquals("a.jpeg", downloadName(testCard("a", blob = Blob(type = "image/jpeg", path = "x/a.JPEG"))))
        assertEquals("b.svg", downloadName(testCard("b", blob = Blob(type = "image/svg+xml", path = "x/b"))))
        assertEquals("c", downloadName(testCard("c", blob = Blob())))
        assertEquals("Title", copyText(testCard("q", type = "Quote", title = "Title")))
        assertEquals(null, copyText(testCard("w")))
        assertEquals(null, copyText(testCard("i", type = "Image", blob = Blob(type = "image/png"))))
    }

    @Test
    fun menuRowsThatOpenSomethingElse() = runTest {
        val actions = CardActions(FakeLibrary(), saver)
        val card = testCard("c")
        assertEquals(CardMenuOutcome.ShowSimilar(card), actions.perform(CardMenuItem(CardMenuAction.SameVibe), card, null))
        assertEquals(CardMenuOutcome.OpenSubmenu(CardSubmenu.Tags), actions.perform(CardMenuItem(CardMenuAction.AddTags), card, null))
        assertEquals(CardMenuOutcome.OpenSubmenu(CardSubmenu.Spaces), actions.perform(CardMenuItem(CardMenuAction.AddToSpace), card, null))
    }

    @Test
    fun inertAndLockedRowsSendNothing() = runTest {
        val library = FakeLibrary()
        val actions = CardActions(library, saver)
        val card = testCard("c")
        listOf(CardMenuAction.ReaderMode, CardMenuAction.Download, CardMenuAction.Copy, CardMenuAction.ViewSource).forEach {
            assertEquals(CardMenuOutcome.NotAvailable(it), actions.perform(CardMenuItem(it), card, null))
        }
        assertEquals(CardMenuOutcome.NotAvailable(CardMenuAction.SameVibe), actions.perform(CardMenuItem(CardMenuAction.SameVibe, locked = true), card, null))
        assertEquals(CardMenuOutcome.NotAvailable(CardMenuAction.DeleteCard), actions.perform(CardMenuItem(CardMenuAction.DeleteCard, locked = true), card, null))
        assertTrue(library.calls.isEmpty())
    }

    @Test
    fun refusedWritesReportTheFailure() = runTest {
        val library = FakeLibrary().apply { writeFailure = ApiFailure.Server(500) }
        val outcome = CardActions(library, saver).perform(CardMenuItem(CardMenuAction.DeleteCard), testCard("c"), null)
        assertEquals(CardMenuOutcome.Failed(ApiFailure.Server(500)), outcome)
    }

    @Test
    fun shareSendsANotesWordsAndAPostsWordsWithItsLinkButNotALinksPage() {
        assertEquals("Just a thought", shareText(testCard("n", type = "Note", title = null, note = "Just a thought")))
        assertEquals("Hello\n\nhttps://x.com/a/1",
            shareText(testCard("x", type = "XPost", url = "https://x.com/a/1").copy(description = "Hello")))
        assertNull(shareText(testCard("a", type = "Article", url = "https://example.com/a")))
        assertNull(shareText(testCard("e", type = "Note", title = null, note = " ")))
    }
}

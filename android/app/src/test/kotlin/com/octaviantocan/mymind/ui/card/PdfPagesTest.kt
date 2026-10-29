package com.octaviantocan.mymind.ui.card

import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.ui.library.MosaicSpan
import com.octaviantocan.mymind.ui.library.mosaicSpan
import com.octaviantocan.mymind.ui.library.pdfStandsTall
import com.octaviantocan.mymind.ui.library.testCard
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PdfPagesTest {
    private val pdf = testCard("doc", type = "Document", title = "Fell walks")
        .copy(blob = Blob("u/doc/fell-walks.pdf", "application/pdf"), cover = Cover("/media/u/doc/fell-walks.pdf;page(1)", 642, 840))

    @Test
    fun aDocumentWithAPdfFileIsAPdf() {
        assertTrue(isPdf(pdf))
        assertTrue("The file's name is enough", isPdf(pdf.copy(blob = Blob("u/doc/notes.PDF"))))
        assertFalse("Without a file there is nothing to draw", isPdf(pdf.copy(blob = null)))
        assertFalse(isPdf(testCard("img", type = "Image").copy(blob = Blob("u/img/a.pdf", "application/pdf"))))
    }

    @Test
    fun aPdfStandsTwoSquaresHighWithItsFirstPage() {
        assertTrue(pdfStandsTall(pdf))
        assertEquals(MosaicSpan(1, 2), mosaicSpan(pdf))
        assertFalse("Without a picture of its first page it keeps the text tile", pdfStandsTall(pdf.copy(cover = null, blob = null)))
    }

    @Test
    fun theDetailLineSaysWhatIsKnown() {
        assertEquals("PDF · 48 pages · 12 min read · Saved 24 Oct", pdfDetail(48, 12, "Saved 24 Oct"))
        assertEquals("PDF · 1 page", pdfDetail(1, null, null))
        assertEquals("PDF · Saved 24 Oct", pdfDetail(null, 0, "Saved 24 Oct"))
    }

    @Test
    fun linesAreJoinedBackIntoParagraphs() {
        val page = """
            Approaches
            Most visitors arrive at Windermere by the railway and see the lake first from the
            station road. It is a poor intro-
            duction. Walk instead up the lane behind the station.

            From Ambleside the roads go north and west into the valleys and
            keep your legs for the hills.
            12
        """.trimIndent()
        assertEquals(
            listOf(
                "Approaches Most visitors arrive at Windermere by the railway and see the lake first from the station road. It is a poor introduction. Walk instead up the lane behind the station.",
                "From Ambleside the roads go north and west into the valleys and keep your legs for the hills.",
            ),
            pdfParagraphs(listOf(page)).lines(),
        )
        assertEquals("Each page ends its paragraph", listOf("One page", "Next page"), pdfParagraphs(listOf("One page", "Next page")).lines())
        assertEquals("", pdfParagraphs(listOf("", "  \n 3 ")))
    }
}

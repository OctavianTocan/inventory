package com.octaviantocan.mymind.share

import org.junit.Assert.assertEquals
import org.junit.Test

class ShareIntentTest {
    private class Probe(val sizes: Map<String, Long> = emptyMap(), val locked: Set<String> = emptySet()) : ShareProbe {
        override fun size(uri: String) = sizes[uri]
        override fun isLockedPdf(uri: String) = uri in locked
    }

    private val send = "android.intent.action.SEND"
    private val multiple = "android.intent.action.SEND_MULTIPLE"

    private fun parse(request: ShareRequest, signedIn: Boolean = true, probe: ShareProbe = Probe()) = parseShare(request, signedIn, probe)

    @Test
    fun signedOutIsNoLogin() {
        assertEquals(ShareState.NoLogin, parse(ShareRequest(send, "text/plain", null, "https://example.com", null), signedIn = false))
    }

    @Test
    fun aLinkIsOneTextItem() {
        val state = parse(ShareRequest(send, "text/plain", null, " https://example.com ", null))
        assertEquals(ShareState.Ready(listOf(ShareItem.Text("https://example.com"))), state)
    }

    @Test
    fun textFallsBackToTheIntentData() {
        assertEquals(ShareState.Ready(listOf(ShareItem.Text("https://a.test"))), parse(ShareRequest("android.intent.action.VIEW", "", null, null, "https://a.test")))
    }

    @Test
    fun selectedTextIsSaved() {
        assertEquals(ShareState.Ready(listOf(ShareItem.Text("quote"))), parse(ShareRequest(ACTION_PROCESS_TEXT, "text/plain", null, "quote", null)))
    }

    @Test
    fun emptyOrMissingTextIsMalformed() {
        assertEquals(ShareState.Malformed, parse(ShareRequest(send, "text/plain", null, "  ", null)))
        assertEquals(ShareState.Malformed, parse(ShareRequest(send, "text/plain", null, null, null)))
        assertEquals(ShareState.Malformed, parse(ShareRequest(send, "text/plain", listOf("content://a"), null, null)))
    }

    @Test
    fun onePictureIsOneStream() {
        assertEquals(ShareState.Ready(listOf(ShareItem.Stream("content://a"))), parse(ShareRequest(send, "image/png", listOf("content://a"), null, null)))
    }

    @Test
    fun aVideoAndAPdfAreStreams() {
        assertEquals(ShareState.Ready(listOf(ShareItem.Stream("content://v"))), parse(ShareRequest(send, "video/mp4", listOf("content://v"), null, null)))
        assertEquals(ShareState.Ready(listOf(ShareItem.Stream("content://p"))), parse(ShareRequest(send, "application/pdf", listOf("content://p"), null, null)))
    }

    @Test
    fun aFileOverTwentyFiveMegabytesIsTooBig() {
        val probe = Probe(sizes = mapOf("content://a" to MAX_SHARE_BYTES + 1))
        assertEquals(ShareState.FileTooBig, parse(ShareRequest(send, "image/png", listOf("content://a"), null, null), probe = probe))
        val exact = Probe(sizes = mapOf("content://a" to MAX_SHARE_BYTES))
        assertEquals(ShareState.Ready(listOf(ShareItem.Stream("content://a"))), parse(ShareRequest(send, "image/png", listOf("content://a"), null, null), probe = exact))
    }

    @Test
    fun aLockedPdfIsRefused() {
        val probe = Probe(locked = setOf("content://p"))
        assertEquals(ShareState.PdfHasPassword, parse(ShareRequest(send, "application/pdf", listOf("content://p"), null, null), probe = probe))
    }

    @Test
    fun severalStreamsKeepTheirOrder() {
        val state = parse(ShareRequest(multiple, "image/*", listOf("content://a", "content://b"), null, null))
        assertEquals(ShareState.Ready(listOf(ShareItem.Stream("content://a"), ShareItem.Stream("content://b"))), state)
    }

    @Test
    fun noStreamsInAMultipleShareIsMalformed() {
        assertEquals(ShareState.Malformed, parse(ShareRequest(multiple, "image/*", null, null, null)))
        assertEquals(ShareState.Malformed, parse(ShareRequest(multiple, "image/*", emptyList(), null, null)))
    }

    @Test
    fun moreThanTwentyStreamsAreTooMany() {
        val uris = (1..21).map { "content://$it" }
        assertEquals(ShareState.TooManyItems, parse(ShareRequest(multiple, "image/*", uris, null, null)))
        assertEquals(20, (parse(ShareRequest(multiple, "image/*", uris.take(20), null, null)) as ShareState.Ready).items.size)
    }

    @Test
    fun oneBigFileAmongSeveralRefusesTheShare() {
        val probe = Probe(sizes = mapOf("content://b" to MAX_SHARE_BYTES + 1))
        assertEquals(ShareState.FileTooBig, parse(ShareRequest(multiple, "image/*", listOf("content://a", "content://b"), null, null), probe = probe))
    }
}

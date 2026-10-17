package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.ui.describe
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class UploadQueueTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private fun file(name: String, text: String, mime: String = "image/jpeg") =
        UploadFile(name, mime, text.length.toLong()) { text.byteInputStream() }

    private fun TestScope.queueOf(library: FakeLibrary, uploaded: MutableList<String> = mutableListOf()) =
        UploadQueue(this, library, onUploaded = { id, space -> uploaded += "$id:$space" }, clock = { 0L }, chunkSize = 2, io = dispatcher)

    @Test
    fun filesGoInPiecesOneAfterAnotherAndJoinTheSpace() = runTest(dispatcher) {
        val library = FakeLibrary()
        val uploaded = mutableListOf<String>()
        val queue = queueOf(library, uploaded)
        queue.add(listOf(file("a.jpg", "hello"), file("b.mp4", "hi", "video/mp4")), "s1")
        assertEquals(listOf("a.jpg", "b.mp4"), queue.state.value.items.map { it.name })
        advanceUntilIdle()

        assertEquals(
            listOf(
                "authorize:2",
                "chunk:token1:null:a.jpg:0+2/5",
                "chunk:token1:u1:a.jpg:2+2/5",
                "chunk:token1:u1:a.jpg:4+1/5",
                "media:u1:s1",
                "space:s1:card-u1",
                "chunk:token2:null:b.mp4:0+2/2",
                "media:u2:s1",
                "space:s1:card-u2",
            ),
            library.calls,
        )
        assertEquals("hello", library.uploaded.getValue("u1").toString())
        assertEquals("hi", library.uploaded.getValue("u2").toString())
        assertEquals(listOf("card-u1:s1", "card-u2:s1"), uploaded)
        assertTrue(queue.state.value.items.isEmpty())
    }

    @Test
    fun theLibraryOnlyCreatesTheCard() = runTest(dispatcher) {
        val library = FakeLibrary()
        queueOf(library).add(listOf(file("a.jpg", "ab")), null)
        advanceUntilIdle()
        assertEquals(listOf("authorize:1", "chunk:token1:null:a.jpg:0+2/2", "media:u1:null"), library.calls)
    }

    @Test
    fun emptyAndOversizedFilesFailWithoutAskingTheServer() = runTest(dispatcher) {
        val library = FakeLibrary()
        val queue = queueOf(library)
        val huge = UploadFile("big.mov", "video/quicktime", UploadQueue.MAX_UPLOAD_SIZE + 1) { error("must not be read") }
        queue.add(listOf(file("empty.png", ""), huge), null)
        advanceUntilIdle()
        assertTrue(library.calls.isEmpty())
        val items = queue.state.value.items
        assertEquals(listOf("This file is empty.", "This file is larger than 500 MB."), items.map { it.error })
        assertFalse(items.any { it.running })
        queue.dismiss(items.first().id)
        assertEquals(listOf("big.mov"), queue.state.value.items.map { it.name })
    }

    @Test
    fun aRefusedPieceFailsItsFileAndTheNextFileStillGoes() = runTest(dispatcher) {
        val library = FakeLibrary()
        library.chunkFailure = ApiFailure.Server(409)
        val queue = queueOf(library)
        queue.add(listOf(file("a.jpg", "abc")), null)
        advanceUntilIdle()
        assertEquals(describe(ApiFailure.Server(409)), queue.state.value.items.single().error)
        assertTrue(library.calls.none { it.startsWith("media:") })

        library.chunkFailure = null
        queue.add(listOf(file("b.jpg", "b")), null)
        advanceUntilIdle()
        assertEquals(listOf("a.jpg"), queue.state.value.items.map { it.name })
        assertTrue("media:u2:null" in library.calls)
    }

    @Test
    fun aFailedAuthorizationFailsTheWholeBatch() = runTest(dispatcher) {
        val library = FakeLibrary()
        library.writeFailure = ApiFailure.Network
        val queue = queueOf(library)
        queue.add(listOf(file("a.jpg", "a"), file("b.jpg", "b")), null)
        advanceUntilIdle()
        assertEquals(listOf(describe(ApiFailure.Network), describe(ApiFailure.Network)), queue.state.value.items.map { it.error })
    }

    @Test
    fun moreThanAHundredFilesAreAuthorizedInBatches() = runTest(dispatcher) {
        val library = FakeLibrary()
        queueOf(library).add((1..101).map { file("f$it.jpg", "x") }, null)
        advanceUntilIdle()
        assertEquals(listOf("authorize:100", "authorize:1"), library.calls.filter { it.startsWith("authorize:") })
        assertEquals(101, library.calls.count { it.startsWith("media:") })
    }

    @Test
    fun theDetailsOpenAndClose() = runTest(dispatcher) {
        val queue = queueOf(FakeLibrary())
        queue.toggleDetails()
        assertTrue(queue.state.value.details)
        queue.toggleDetails()
        assertFalse(queue.state.value.details)
    }

    @Test
    fun anUploadedCardOpensTheLibraryAndTheSpaceIsReadAgain() = runTest(dispatcher) {
        val library = FakeLibrary(
            pages = mapOf(null to CardPage(listOf(testCard("old")))),
            cardsById = mapOf("card-u1" to testCard("card-u1", type = "Image")),
        )
        val viewModel = LibraryViewModel(library, io = dispatcher)
        advanceUntilIdle()
        viewModel.spaceFeed("s1")
        advanceUntilIdle()
        library.calls.clear()
        viewModel.upload(listOf(file("a.jpg", "ab")), "s1")
        advanceUntilIdle()
        assertEquals(listOf("card-u1", "old"), viewModel.everything.state.value.cards.map { it.id })
        assertTrue("card:card-u1" in library.calls)
        assertTrue("spaceCards:s1:null" in library.calls)
        assertTrue("spaces" in library.calls)
    }
}

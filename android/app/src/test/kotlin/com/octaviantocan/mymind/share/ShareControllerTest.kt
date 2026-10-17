package com.octaviantocan.mymind.share

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.ui.library.FakeLibrary
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ShareControllerTest {
    private val dispatcher = StandardTestDispatcher()

    private fun file(name: String) = UploadFile(name, "image/jpeg", 2) { "hi".byteInputStream() }

    @Test
    fun aLinkIsSavedThenTaggedNotedAndFiled() = runTest(dispatcher) {
        val library = FakeLibrary()
        val controller = ShareController(this, library, dispatcher) { null }
        controller.start(listOf(ShareItem.Text("https://example.com")))
        advanceUntilIdle()
        val id = controller.cards.value.single()
        val result = controller.finish(ShareDetails(listOf("read", "later"), " a note ", setOf("s1")))
        assertEquals(ApiResult.Ok(Unit), result)
        assertEquals(
            listOf("content:https://example.com:clipboard:null", "tag:$id:read", "tag:$id:later", "note:$id:a note", "space:s1:$id"),
            library.calls,
        )
    }

    @Test
    fun detailsGoOnEveryUploadedCard() = runTest(dispatcher) {
        val library = FakeLibrary()
        val controller = ShareController(this, library, dispatcher) { uri -> file(uri) }
        controller.start(listOf(ShareItem.Stream("a.jpg"), ShareItem.Stream("b.jpg")))
        val result = controller.finish(ShareDetails(listOf("trip"), "", setOf("s1")))
        assertEquals(ApiResult.Ok(Unit), result)
        assertEquals(2, controller.cards.value.size)
        controller.cards.value.forEach { id ->
            assertTrue("tag:$id:trip" in library.calls)
            assertTrue("space:s1:$id" in library.calls)
        }
        assertTrue(library.calls.none { it.startsWith("note:") })
    }

    @Test
    fun nothingSavedIsAFailure() = runTest(dispatcher) {
        val library = FakeLibrary().apply { writeFailure = ApiFailure.Network }
        val controller = ShareController(this, library, dispatcher) { null }
        controller.start(listOf(ShareItem.Text("hello")))
        advanceUntilIdle()
        assertEquals(ApiResult.Failed(ApiFailure.Network), controller.finish(ShareDetails()))
    }

    @Test
    fun anUnreadableFileDoesNotStopTheOthers() = runTest(dispatcher) {
        val library = FakeLibrary()
        val controller = ShareController(this, library, dispatcher) { uri -> if (uri == "bad") null else file(uri) }
        controller.start(listOf(ShareItem.Stream("bad"), ShareItem.Stream("good.jpg")))
        assertEquals(ApiResult.Ok(Unit), controller.finish(ShareDetails()))
        assertEquals(1, controller.cards.value.size)
    }

    @Test
    fun theControllerIsBusyUntilEverythingIsSaved() = runTest(dispatcher) {
        val controller = ShareController(this, FakeLibrary(), dispatcher) { uri -> file(uri) }
        assertEquals(false, controller.busy)
        controller.start(listOf(ShareItem.Stream("a.jpg")))
        assertEquals(true, controller.busy)
        advanceUntilIdle()
        assertEquals(false, controller.busy)
    }

    @Test
    fun theActivityRegistrySeesABusyController() = runTest(dispatcher) {
        val activity = com.octaviantocan.mymind.update.UploadActivity()
        val controller = ShareController(this, FakeLibrary(), dispatcher, activity) { uri -> file(uri) }
        controller.start(listOf(ShareItem.Text("https://example.com")))
        assertEquals(true, activity.busy())
        advanceUntilIdle()
        assertEquals(false, activity.busy())
    }
}

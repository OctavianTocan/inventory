package com.octaviantocan.mymind.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.test.core.app.ApplicationProvider
import coil3.ImageLoader
import coil3.SingletonImageLoader
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import com.octaviantocan.mymind.ui.library.EverythingContent
import com.octaviantocan.mymind.ui.library.FeedState
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okio.Buffer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/** The real gallery must isolate its public account-photo request from the authenticated app image loader. */
@OptIn(coil3.annotation.DelicateCoilApi::class)
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS)
class GalleryAccountPhotoTest {
    @get:Rule val compose = createComposeRule()

    @Test fun accountPhotosNeverUseTheLibrarySessionBearer() {
        val authenticatedRequests = AtomicInteger()
        val client = OkHttpClient.Builder().addInterceptor { chain ->
            authenticatedRequests.incrementAndGet()
            chain.proceed(chain.request().newBuilder().header("Authorization", "Bearer synthetic-test-session").build())
        }.build()
        val authenticatedLoader = ImageLoader.Builder(ApplicationProvider.getApplicationContext())
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { client })) }.build()
        val server = MockWebServer()
        val picture = checkNotNull(javaClass.getResourceAsStream("/gallery/forest.jpg")).use { it.readBytes() }
        server.enqueue(MockResponse().setHeader("Content-Type", "image/jpeg").setBody(Buffer().write(picture)))
        server.start()
        SingletonImageLoader.setUnsafe(authenticatedLoader)
        try {
            compose.setPhoneContent(false) {
                EverythingContent(
                    feed = FeedState(endReached = true), searching = false, searchResults = null,
                    typeChips = emptyList(), filter = null, suggestions = emptyList(),
                    onChoose = {}, onRemoveFilter = {}, onLoadMore = {}, onLoadMoreResults = {},
                    onRetry = {}, onOpenCard = {}, onOpenSettings = {}, onAdd = {},
                    photoUrl = server.url("/avatar.jpg").toString(),
                )
            }
            compose.waitUntil(10_000) { server.requestCount > 0 }
            val request = checkNotNull(server.takeRequest(2, TimeUnit.SECONDS))
            assertNull(request.getHeader("Authorization"))
            assertEquals(0, authenticatedRequests.get())
            compose.onNodeWithContentDescription("Account settings").assertIsDisplayed()
        } finally {
            SingletonImageLoader.reset()
            authenticatedLoader.shutdown()
            client.connectionPool.evictAll()
            client.dispatcher.executorService.shutdown()
            server.shutdown()
        }
    }
}

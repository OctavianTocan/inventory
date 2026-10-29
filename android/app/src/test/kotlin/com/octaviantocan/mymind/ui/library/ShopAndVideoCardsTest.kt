package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.AppInfo
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Product
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ShopAndVideoCardsTest {
    private val photo = Cover(src = "https://example.test/lamp.jpg", width = 900, height = 1200)

    private fun product(price: String?, brand: String? = null) = testCard("p", type = "Product", title = "Paper lamp")
        .copy(product = Product(price, brand), cover = photo, siteName = "Hollow & Pine", domain = "hollowpine.test")

    @Test
    fun aPriceShowsItsCurrencysSignAndDropsEmptyPence() {
        assertEquals("£24", productPrice(product("GBP 24.00")))
        assertEquals("$18.50", productPrice(product("USD 18.5")))
        assertEquals("CHF 30", productPrice(product("CHF 30")))
        assertEquals("€9.99", productPrice(product("eur 9.99")))
        assertEquals("A price the shop wrote its own way stays as it is", "From £12", productPrice(product("From £12")))
        assertNull(productPrice(product(null)))
        assertNull(productPrice(product("  ")))
    }

    @Test
    fun aProductNamesItsBrandElseItsShop() {
        assertEquals("Norrland", productShop(product("GBP 24.00", brand = "Norrland")))
        assertEquals("Hollow & Pine", productShop(product("GBP 24.00")))
        assertEquals("hollowpine.test", productShop(product(null).copy(siteName = " ")))
    }

    @Test
    fun aProductWithAPhotoStandsTwoSquaresHighAsAPin() {
        assertTrue(productStandsTall(product("GBP 24.00")))
        assertEquals(MosaicSpan(1, 2), mosaicSpan(product("GBP 24.00")))
        assertFalse("Without a photo it keeps the text tile", productStandsTall(product("GBP 24.00").copy(cover = null)))
        assertEquals(MosaicSpan(1, 1), mosaicSpan(product("GBP 24.00").copy(cover = null)))
    }

    @Test
    fun anAppNamesTheStoreItIsOn() {
        val app = testCard("a", type = "SoftwareApplication", title = "Trailhead").copy(domain = "play.google.com")
        assertEquals(AppStore("Google Play", R.drawable.ic_si_googleplay), appStore(app.copy(app = AppInfo("play-store"))))
        assertEquals(AppStore("App Store", R.drawable.ic_si_appstore), appStore(app.copy(app = AppInfo("app-store"))))
        assertEquals(AppStore("play.google.com", null), appStore(app))
        assertNull(appStore(app.copy(domain = null)))
    }

    @Test
    fun aVideoFromAnotherSiteShowsItsTitleAndChannelUnderItsStill() {
        val video = testCard("y", type = "YouTubeVideo", title = "Folding a paper lamp")
            .copy(author = "by Maya Walks", cover = Cover(src = "https://example.test/still.jpg", width = 1280, height = 720))
        assertEquals(VideoCaption("Folding a paper lamp", "Maya Walks"), videoCaption(video))
        assertEquals(VideoCaption("Folding a paper lamp", null), videoCaption(video.copy(author = null)))
        assertNull("A note of its own keeps the note beside the picture", videoCaption(video.copy(note = "For the hallway")))
        assertNull("Without a still there is nothing to caption", videoCaption(video.copy(cover = null)))
        assertNull("Only videos from other sites", videoCaption(video.copy(type = "Video")))
        assertFalse("A captioned video has words, so a pinch scales it", galleryTileIsPicture(video))
    }
}

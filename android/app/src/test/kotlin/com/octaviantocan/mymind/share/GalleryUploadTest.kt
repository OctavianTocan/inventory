package com.octaviantocan.mymind.share

import android.content.Context
import android.net.Uri
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.galleryUploadOf
import com.octaviantocan.mymind.data.mediaStoreUriOf
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Shadows.shadowOf

@RunWith(AndroidJUnit4::class)
class GalleryUploadTest {
    private val context = ApplicationProvider.getApplicationContext<android.app.Application>()

    @Test
    fun aGalleryShareIsItsOwnMediaStoreEntry() {
        val shared = Uri.parse("content://media/external/video/media/42")
        assertEquals(shared, mediaStoreUriOf(shared))
    }

    @Test
    fun googlePhotosGivesTheEntryItWrapsForAnItemOnThePhone() {
        val shared = Uri.parse("content://com.google.android.apps.photos.contentprovider/-1/2/content%3A%2F%2Fmedia%2Fexternal%2Fvideo%2Fmedia%2F42/ORIGINAL/NONE/video%2Fmp4/1234")
        assertEquals(Uri.parse("content://media/external/video/media/42"), mediaStoreUriOf(shared))
    }

    @Test
    fun cloudOnlyPhotosAndOtherAppsHaveNoEntry() {
        assertNull(mediaStoreUriOf(Uri.parse("content://com.google.android.apps.photos.contentprovider/0/1/mediakey%3A%2Flocal%253Aabc/ORIGINAL/NONE/1")))
        assertNull(mediaStoreUriOf(Uri.parse("content://com.whatsapp.provider.media/item/1")))
    }

    @Test
    fun withoutGalleryAccessTheShareIsCopied() {
        assertNull(galleryUploadOf(context.contentResolver, Uri.parse("content://media/external/video/media/42"), canReadGallery = false, trustPhotos = true))
    }

    @Test
    fun aPhotosLinkFromAnAppPosingAsPhotosIsNotUnwrapped() {
        val shared = Uri.parse("content://com.google.android.apps.photos.contentprovider/-1/2/content%3A%2F%2Fmedia%2Fexternal%2Fvideo%2Fmedia%2F42/ORIGINAL")
        assertNull(mediaStoreUriOf(shared, trustPhotos = false))
        assertFalse(GalleryAccess.photosIsGenuine(context))
    }

    @Test
    fun aGalleryEntrySentWithoutAGrantIsNotRead() {
        val gallery = Uri.parse("content://media/external/video/media/42")
        assertFalse(GalleryAccess.mayRead(gallery, shareGranted = false))
        assertTrue(GalleryAccess.mayRead(gallery, shareGranted = true))
        assertTrue(GalleryAccess.mayRead(Uri.parse("content://com.whatsapp.provider.media/item/1"), shareGranted = false))
    }

    @Test
    fun theQuestionComesOnceAndOnlyForGalleryItems() {
        val gallery = listOf(ShareItem.Stream("content://media/external/video/media/42"))
        assertFalse(GalleryAccess.shouldAsk(context, listOf(ShareItem.Stream("content://com.whatsapp.provider.media/item/1"))))
        assertTrue(GalleryAccess.shouldAsk(context, gallery))
        GalleryAccess.markAsked(context)
        assertFalse(GalleryAccess.shouldAsk(context, gallery))
        shadowOf(context).grantPermissions(*GalleryAccess.permissions)
        assertTrue(GalleryAccess.granted(context as Context))
    }
}

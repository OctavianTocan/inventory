package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.Blob
import com.octaviantocan.mymind.data.Cover
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class VideoPreviewsTest {
    private val server = BuildConfig.SERVER_URL.trimEnd('/')

    @Test fun lengthsReadLikeAPlayerClock() {
        assertEquals("0:00", videoLength(0))
        assertEquals("0:09", videoLength(9_400))
        assertEquals("1:05", videoLength(65_000))
        assertEquals("59:59", videoLength(3_599_000))
        assertEquals("1:00:00", videoLength(3_600_000))
    }

    @Test fun anUploadedVideoPreviewsASmallSilentCopyUnderItsStillCover() {
        val card = testCard("clip", type = "Video")
            .copy(cover = Cover("/media/u/clip;600x1000-t.webp", 600, 1000), blob = Blob("u/clip", "video/mp4"))
        assertEquals("$server/media/u/clip;480x600.mp4", previewVideo(card))
        assertEquals("/media/u/clip;600x1000-t.webp", galleryPicture(card))
    }

    @Test fun aMovingCoverPlaysItselfAndShowsAStillFrame() {
        val card = testCard("gif", type = "Image").copy(cover = Cover("/media/u/a;400x300.mp4", 400, 300, type = "video"))
        assertEquals("$server/media/u/a;400x300.mp4", previewVideo(card))
        assertEquals("/media/u/a;400x300.webp", galleryPicture(card))
        assertEquals("/media/u/b;800x1000.webp", videoPoster("/media/u/b;800x1000/hevc.mp4"))
    }

    @Test fun aVideoKeptWithoutACoverShowsAServerStillAndPlaysInTheReader() {
        val card = testCard("raw", type = "Video").copy(blob = Blob("u/raw", "video/mp4", 1080, 1920))
        assertEquals("/media/u/raw", serverVideo(card))
        assertEquals("/media/u/raw;800x1000-t.webp", galleryPicture(card))
        assertEquals("$server/media/u/raw;480x600.mp4", previewVideo(card))
        assertEquals("$server/media/u/raw", readerVideo(card))
    }

    @Test fun theReaderPlaysTheOriginalOfAMovingCover() {
        val card = testCard("gif", type = "Image").copy(cover = Cover("/media/u/a;400x300.mp4", 400, 300, type = "video"))
        assertEquals("$server/media/u/a;400x300.mp4", readerVideo(card))
        assertNull(readerVideo(testCard("photo", type = "Image").copy(cover = Cover("/media/u/p.jpg", 400, 300))))
    }

    @Test fun videosOnOtherSitesNeverReachTheLibraryClient() {
        val youtube = testCard("yt", type = "YouTubeVideo").copy(cover = Cover("https://i.ytimg.com/vi/x/hq.jpg", 1280, 720))
        val remote = testCard("r", type = "Video").copy(cover = Cover("https://cdn.example.test/a.mp4", type = "video"),
            blob = Blob("https://cdn.example.test/a.mp4", "video/mp4"))
        assertNull(previewVideo(youtube))
        assertNull(previewVideo(remote))
        assertNull(readerVideo(youtube))
        assertNull(readerVideo(remote))
        assertNull(videoPoster("https://cdn.example.test/a.mp4"))
    }
}

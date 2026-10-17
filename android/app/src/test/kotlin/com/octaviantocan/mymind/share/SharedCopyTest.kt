package com.octaviantocan.mymind.share

import android.content.Context
import android.net.Uri
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.octaviantocan.mymind.data.copiedUploadOf
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File

@RunWith(AndroidJUnit4::class)
class SharedCopyTest {
    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val directory = File(context.cacheDir, "shared-test").also { it.deleteRecursively() }

    @Test
    fun theCopyOutlivesTheSharedFileAndGoesAwayAfterReading() {
        val shared = File(context.filesDir, "picture.png").apply { writeText("picture bytes") }
        val upload = copiedUploadOf(context.contentResolver, Uri.fromFile(shared), directory)
        shared.delete()
        assertNotNull(upload)
        assertEquals("picture.png", upload!!.name)
        assertEquals(13L, upload.size)
        assertEquals("picture bytes", upload.open().use { it.readBytes().decodeToString() })
        assertFalse(directory.listFiles().orEmpty().any())
    }

    @Test
    fun aMissingFileGivesNoUpload() {
        assertNull(copiedUploadOf(context.contentResolver, Uri.fromFile(File(context.filesDir, "gone.png")), directory))
        assertFalse(directory.listFiles().orEmpty().any())
    }
}

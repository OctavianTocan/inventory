package com.octaviantocan.mymind.ui.menu

import android.content.ContentResolver
import android.content.ContentValues
import android.os.Environment
import android.provider.MediaStore
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.IOException
import java.io.OutputStream

/** Saves a file where the user finds it. */
fun interface FileSaver {
    /**
     * @param name File name.
     * @param mimeType Media type of the file, or null when unknown.
     * @param write Fills the file; a failure it returns removes the file again.
     * @return Success, or why the file could not be saved.
     */
    suspend fun save(name: String, mimeType: String?, write: suspend (OutputStream) -> ApiResult<Unit>): ApiResult<Unit>
}

/**
 * Saves files into the shared Downloads folder through MediaStore, which needs no storage permission.
 *
 * @param resolver Content resolver of the app.
 */
class DownloadsSaver(private val resolver: ContentResolver) : FileSaver {
    override suspend fun save(name: String, mimeType: String?, write: suspend (OutputStream) -> ApiResult<Unit>): ApiResult<Unit> =
        withContext(Dispatchers.IO) {
            val values = ContentValues().apply {
                put(MediaStore.Downloads.DISPLAY_NAME, name)
                put(MediaStore.Downloads.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS)
                if (mimeType != null) put(MediaStore.Downloads.MIME_TYPE, mimeType)
                put(MediaStore.Downloads.IS_PENDING, 1)
            }
            val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values)
                ?: return@withContext ApiResult.Failed(ApiFailure.Server(0))
            val result = try {
                resolver.openOutputStream(uri)?.use { write(it) } ?: ApiResult.Failed(ApiFailure.Server(0))
            } catch (_: IOException) {
                ApiResult.Failed(ApiFailure.Network)
            }
            if (result is ApiResult.Ok) {
                resolver.update(uri, ContentValues().apply { put(MediaStore.Downloads.IS_PENDING, 0) }, null, null)
            } else {
                resolver.delete(uri, null, null)
            }
            result
        }
}

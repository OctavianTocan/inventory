package com.octaviantocan.mymind.data

import android.content.ContentResolver
import android.net.Uri
import android.provider.OpenableColumns
import java.io.File
import java.io.FileNotFoundException
import java.io.InputStream

/**
 * A file to upload.
 *
 * @property name File name the server stores.
 * @property mime Media type.
 * @property size Length in bytes.
 * @property open Opens the file at its start; the caller closes the stream.
 */
class UploadFile(val name: String, val mime: String, val size: Long, val open: () -> InputStream)

/**
 * Describes a file the system picker returned.
 *
 * @param resolver Resolver that reads the file.
 * @param uri The picked file.
 * @return The file, or null when it cannot be read.
 */
fun uploadFileOf(resolver: ContentResolver, uri: Uri): UploadFile? {
    var name: String? = null
    var size: Long? = null
    runCatching {
        resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) {
                cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME).takeIf { it >= 0 && !cursor.isNull(it) }?.let { name = cursor.getString(it) }
                cursor.getColumnIndex(OpenableColumns.SIZE).takeIf { it >= 0 && !cursor.isNull(it) }?.let { size = cursor.getLong(it) }
            }
        }
    }
    val length = size ?: runCatching { resolver.openInputStream(uri)?.use { countBytes(it) } }.getOrNull() ?: return null
    val mime = resolver.getType(uri) ?: "application/octet-stream"
    return UploadFile(name ?: uri.lastPathSegment ?: "upload", mime, length) {
        resolver.openInputStream(uri) ?: throw FileNotFoundException(uri.toString())
    }
}

/**
 * Describes a photo the camera saved.
 *
 * @param file The photo.
 * @return The file as a JPEG upload.
 */
fun uploadFileOf(file: File): UploadFile = UploadFile(file.name, "image/jpeg", file.length()) { file.inputStream() }

/** Length of what is left in [stream]. */
private fun countBytes(stream: InputStream): Long {
    val buffer = ByteArray(64 * 1024)
    var total = 0L
    while (true) {
        val read = stream.read(buffer)
        if (read < 0) return total
        total += read
    }
}

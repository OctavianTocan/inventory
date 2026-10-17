package com.octaviantocan.mymind.data

import android.content.ContentResolver
import android.net.Uri
import android.provider.OpenableColumns
import java.io.File
import java.io.FileNotFoundException
import java.io.FilterInputStream
import java.io.IOException
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
 * Copies a shared file into [directory], so it can still be read after the share sheet closes and the sharing app's
 * permission ends.
 *
 * @param resolver Resolver that reads the shared file.
 * @param uri The shared file.
 * @param directory Where the copy goes; closing a stream of the copy deletes it.
 * @return The copy as an upload, or null when the shared file cannot be read.
 */
fun copiedUploadOf(resolver: ContentResolver, uri: Uri, directory: File): UploadFile? {
    val shared = uploadFileOf(resolver, uri) ?: return null
    directory.mkdirs()
    val copy = File.createTempFile("share", null, directory)
    val length = try {
        shared.open().use { input -> copy.outputStream().use { input.copyTo(it) } }
    } catch (_: IOException) {
        copy.delete()
        return null
    } catch (_: SecurityException) {
        copy.delete()
        return null
    }
    return UploadFile(shared.name, shared.mime, length) {
        object : FilterInputStream(copy.inputStream()) {
            override fun close() {
                super.close()
                copy.delete()
            }
        }
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

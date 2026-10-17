package com.octaviantocan.mymind.share

import android.content.ContentResolver
import android.content.Intent
import android.graphics.pdf.PdfRenderer
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns

/**
 * Reads the parts the share target needs out of an intent.
 *
 * @param intent The intent the share target was started with.
 * @return Its action, MIME type, streams, text and data.
 */
fun requestOf(intent: Intent): ShareRequest {
    val streams = if (intent.action == Intent.ACTION_SEND_MULTIPLE) {
        streamList(intent)?.map { it.toString() }
    } else {
        streamOne(intent)?.let { listOf(it.toString()) }
    }
    val text = intent.getCharSequenceExtra(Intent.EXTRA_TEXT) ?: intent.getCharSequenceExtra(Intent.EXTRA_PROCESS_TEXT)
    return ShareRequest(intent.action, intent.type.orEmpty(), streams, text?.toString(), intent.dataString)
}

@Suppress("DEPRECATION")
private fun streamOne(intent: Intent): Uri? =
    if (Build.VERSION.SDK_INT >= 33) intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java) else intent.getParcelableExtra(Intent.EXTRA_STREAM)

@Suppress("DEPRECATION")
private fun streamList(intent: Intent): List<Uri>? =
    if (Build.VERSION.SDK_INT >= 33) intent.getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri::class.java) else intent.getParcelableArrayListExtra(Intent.EXTRA_STREAM)

/** [ShareProbe] that asks a content resolver. */
class ContentProbe(private val resolver: ContentResolver) : ShareProbe {
    override fun size(uri: String): Long? = runCatching {
        resolver.query(Uri.parse(uri), arrayOf(OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst() && !cursor.isNull(0)) cursor.getLong(0) else null
        }
    }.getOrNull()

    override fun isLockedPdf(uri: String): Boolean {
        val descriptor = try {
            resolver.openFileDescriptor(Uri.parse(uri), "r")
        } catch (_: Exception) {
            null
        } ?: return false
        return descriptor.use {
            try {
                PdfRenderer(it).close()
                false
            } catch (_: SecurityException) {
                true
            } catch (_: Exception) {
                false
            }
        }
    }
}

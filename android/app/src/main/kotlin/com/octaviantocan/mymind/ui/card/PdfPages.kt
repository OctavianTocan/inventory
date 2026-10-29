package com.octaviantocan.mymind.ui.card

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.pdf.PdfRenderer
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.annotation.RequiresApi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.core.content.FileProvider
import com.octaviantocan.mymind.data.Card
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.io.Closeable
import java.io.File
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.roundToInt

/** A PDF the reader shows page by page. */
interface PdfPages : Closeable {
    /** How many pages it has, once [load] has said yes. */
    val count: Int

    /**
     * Reads how many pages it has and their shapes.
     *
     * @return False when the phone can't draw it, such as a PDF locked with a password.
     */
    suspend fun load(): Boolean

    /**
     * @param index A page, from 0.
     * @return The page's width over its height.
     */
    fun ratio(index: Int): Float

    /**
     * @param index A page, from 0.
     * @param width How many pixels wide to draw it.
     * @return The page on white, or null when it can't be drawn.
     */
    suspend fun draw(index: Int, width: Int): ImageBitmap?

    /** @return Its words as paragraphs, one a line ([pdfParagraphs]); null when the phone can't read them or it has none. */
    suspend fun text(): String?

    /**
     * Hands the PDF to the phone's own viewer.
     *
     * @param context A context that can start activities.
     */
    fun view(context: Context)
}

/** Where the reader gets a card's PDF: fetched once to the phone, then opened to draw. */
interface PdfSource {
    /**
     * @param card A card holding a PDF ([isPdf]).
     * @return The file on the phone, fetched from the library server the first time; null when it can't be fetched.
     */
    suspend fun file(card: Card): File?

    /**
     * @param file A PDF on the phone, from [file].
     * @return It ready to [PdfPages.load]; the caller closes it.
     */
    fun open(file: File): PdfPages
}

/** The reader's [PdfSource]; without one, a PDF shows only its first page, as the server draws it. */
val LocalPdfSource = staticCompositionLocalOf<PdfSource?> { null }

/**
 * @param card A card.
 * @return True for a card holding a PDF file, uploaded or fetched from a link.
 */
fun isPdf(card: Card): Boolean {
    val blob = card.blob ?: return false
    val path = blob.path ?: return false
    return card.type == "Document" && (blob.type == PDF_TYPE || path.endsWith(".pdf", ignoreCase = true))
}

/**
 * The card's PDF, opened while the reader shows it and closed when it goes.
 *
 * @param card The card the reader shows.
 * @return Its pages once fetched and loaded; null before then, without a [LocalPdfSource] or when it can't be drawn.
 */
@Composable
internal fun rememberPdfPages(card: Card): PdfPages? {
    val source = LocalPdfSource.current
    var file by remember(card.id) { mutableStateOf<File?>(null) }
    LaunchedEffect(source, card.id, card.blob?.path) {
        if (source != null && isPdf(card)) file = source.file(card)
    }
    val pages = remember(source, file) { file?.let { source?.open(it) } }
    DisposableEffect(pages) { onDispose { pages?.close() } }
    var ready by remember(pages) { mutableStateOf(false) }
    LaunchedEffect(pages) { ready = pages?.load() == true }
    return pages?.takeIf { ready }
}

/**
 * Joins a PDF's lines back into paragraphs: a blank line or a short line ending a sentence closes one, a word broken
 * with a hyphen at a line's end is mended, and lines that are only a page number are dropped.
 *
 * @param pages Each page's words, its lines as the PDF sets them.
 * @return The paragraphs, one a line, as Reader Mode reads them; empty when there are no words.
 */
fun pdfParagraphs(pages: List<String>): String {
    val paragraphs = mutableListOf<String>()
    val current = StringBuilder()
    fun close() {
        if (current.isNotBlank()) paragraphs += current.toString().trim()
        current.clear()
    }
    pages.forEach { page ->
        val lines = page.lines().map { it.trim() }
        val full = lines.maxOfOrNull { it.length } ?: 0
        lines.forEach { line ->
            when {
                line.isEmpty() -> close()
                PAGE_NUMBER.matches(line) -> Unit
                else -> {
                    if (current.endsWith("-") && line.first().isLowerCase()) current.setLength(current.length - 1)
                    else if (current.isNotEmpty()) current.append(' ')
                    current.append(line)
                    if (line.last() in SENTENCE_ENDS && line.length < full * SHORT_LINE) close()
                }
            }
        }
        close()
    }
    return paragraphs.joinToString("\n")
}

/**
 * Fetches a card's PDF into [dir] through [fetch], keeping the newest few there so a PDF opened again draws at once.
 *
 * @param dir The phone's folder for fetched PDFs.
 * @param fetch Writes the card's file to the given file; false when it could not.
 */
class FetchedPdfs(private val dir: File, private val fetch: suspend (Card, File) -> Boolean) : PdfSource {
    override suspend fun file(card: Card): File? = withContext(Dispatchers.IO) {
        val path = card.blob?.path ?: return@withContext null
        val name = card.id.replace(UNSAFE_NAME, "_") + "-" + Integer.toHexString(path.hashCode())
        val file = File(dir, "$name.pdf")
        if (file.length() > 0) return@withContext file.also { it.setLastModified(System.currentTimeMillis()) }
        dir.mkdirs()
        val part = File(dir, "$name.part")
        val fetched = runCatching { fetch(card, part) }.getOrDefault(false) && part.length() > 0 && part.renameTo(file)
        part.delete()
        if (!fetched) return@withContext null
        dir.listFiles { it: File -> it.name.endsWith(".pdf") }.orEmpty().sortedByDescending { it.lastModified() }.drop(KEPT_PDFS)
            .forEach { it.delete() }
        file
    }

    override fun open(file: File): PdfPages = RenderedPdf(file)
}

/** A PDF on the phone, drawn with the system's [PdfRenderer], one page at a time. */
private class RenderedPdf(private val file: File) : PdfPages {
    private val lock = Mutex()
    private val closed = AtomicBoolean(false)
    private val shut = AtomicBoolean(false)
    private var descriptor: ParcelFileDescriptor? = null
    private var renderer: PdfRenderer? = null
    private var ratios = FloatArray(0)

    override val count: Int get() = ratios.size

    override suspend fun load(): Boolean = locked {
        runCatching {
            val opened = ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY)
            descriptor = opened
            val pdf = PdfRenderer(opened)
            renderer = pdf
            ratios = FloatArray(pdf.pageCount) { index -> pdf.openPage(index).use { it.width.toFloat() / it.height.coerceAtLeast(1) } }
            ratios.isNotEmpty()
        }.getOrDefault(false)
    } ?: false

    override fun ratio(index: Int): Float = ratios.getOrElse(index) { A4_RATIO }

    override suspend fun draw(index: Int, width: Int): ImageBitmap? = locked {
        val pdf = renderer?.takeIf { index in ratios.indices && width > 0 } ?: return@locked null
        runCatching {
            pdf.openPage(index).use { page ->
                val bitmap = Bitmap.createBitmap(width, (width / ratios[index]).roundToInt().coerceAtLeast(1), Bitmap.Config.ARGB_8888)
                bitmap.eraseColor(android.graphics.Color.WHITE)
                page.render(bitmap, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY)
                bitmap.asImageBitmap()
            }
        }.getOrNull()
    }

    override suspend fun text(): String? {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.VANILLA_ICE_CREAM) return null
        val pages = (0 until count).map { index ->
            locked { renderer?.let { pdf -> runCatching { pageText(pdf, index) }.getOrNull() } } ?: return null
        }
        return pdfParagraphs(pages).ifBlank { null }
    }

    override fun view(context: Context) {
        val uri = FileProvider.getUriForFile(context, "${context.packageName}.provider", file)
        val intent = Intent(Intent.ACTION_VIEW).setDataAndType(uri, PDF_TYPE).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        runCatching { context.startActivity(intent) }
    }

    override fun close() {
        closed.set(true)
        if (lock.tryLock()) {
            try {
                shutDown()
            } finally {
                lock.unlock()
            }
        }
    }

    // Runs [block] on the IO threads with the renderer to itself; null once the reader has gone, and whoever ends
    // last after it has gone closes the renderer.
    private suspend fun <T> locked(block: () -> T?): T? = try {
        lock.withLock { withContext(Dispatchers.IO) { if (closed.get()) null else block() } }
    } finally {
        if (closed.get() && lock.tryLock()) {
            try {
                shutDown()
            } finally {
                lock.unlock()
            }
        }
    }

    // Closes the renderer and its file once, by whoever holds the lock when the reader has gone.
    private fun shutDown() {
        if (!shut.compareAndSet(false, true)) return
        runCatching { renderer?.close() }
        runCatching { descriptor?.close() }
    }

    @RequiresApi(Build.VERSION_CODES.VANILLA_ICE_CREAM)
    private fun pageText(pdf: PdfRenderer, index: Int): String = pdf.openPage(index).use { page -> page.textContents.joinToString("\n") { it.text } }
}

private const val PDF_TYPE = "application/pdf"
private const val A4_RATIO = 0.707f
private const val SHORT_LINE = 0.8f
private const val KEPT_PDFS = 12
private val PAGE_NUMBER = Regex("""^(?:page\s+)?\d{1,4}(?:\s*(?:of|/)\s*\d{1,4})?$""", RegexOption.IGNORE_CASE)
private val SENTENCE_ENDS = setOf('.', '!', '?', ':', '"', '”', '’', ')')
private val UNSAFE_NAME = Regex("[^A-Za-z0-9_-]")

package com.octaviantocan.mymind.share

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.lifecycle.lifecycleScope
import com.octaviantocan.mymind.AppContainer
import com.octaviantocan.mymind.MainActivity
import com.octaviantocan.mymind.MymindApp
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.data.copiedUploadOf
import com.octaviantocan.mymind.data.galleryUploadOf
import com.octaviantocan.mymind.data.uploadFileOf
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.menu.suggestSpace
import com.octaviantocan.mymind.ui.moment.FindCount
import com.octaviantocan.mymind.ui.moment.ShareMoment
import com.octaviantocan.mymind.ui.moment.rarityIn
import com.octaviantocan.mymind.ui.moment.sourceOfShare
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareProblem
import com.octaviantocan.mymind.ui.share.ShareSheet
import com.octaviantocan.mymind.ui.share.ShareSheetState
import com.octaviantocan.mymind.ui.share.SharedFile
import com.octaviantocan.mymind.ui.share.thymindTags
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.time.Instant
import kotlin.math.roundToInt

/** Receives text, links, pictures, videos, audio files and PDFs shared from other apps and saves them to the library. */
class ShareActivity : ComponentActivity() {
    private var galleryAnswer: CompletableDeferred<Unit>? = null

    // Android grants a share's files only when the sending app could read them itself (ui/share/CONTRACTS).
    private val shareGranted: Boolean
        get() = intent.flags and Intent.FLAG_GRANT_READ_URI_PERMISSION != 0
    private val askGallery = registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
        galleryAnswer?.complete(Unit)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        // The sheet draws its own way in and out, so the window neither slides nor fades on top of it.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            overrideActivityTransition(OVERRIDE_TRANSITION_OPEN, 0, 0)
            overrideActivityTransition(OVERRIDE_TRANSITION_CLOSE, 0, 0)
        }
        val container = (application as MymindApp).container
        val intent = intent
        lifecycleScope.launch {
            container.session.restore()
            val signedIn = container.session.state.value is SessionState.SignedIn
            val state = parseShare(requestOf(intent), signedIn, ContentProbe(contentResolver))
            if (state is ShareState.Ready && GalleryAccess.shouldAsk(this@ShareActivity, state.items)) {
                GalleryAccess.markAsked(this@ShareActivity)
                // Asked once, before saving, so a big video can upload from the Gallery's own file.
                val answer = CompletableDeferred<Unit>()
                galleryAnswer = answer
                askGallery.launch(GalleryAccess.permissions)
                answer.await()
            }
            setContent {
                MymindTheme(dark = container.themePrefs.chromeDark(isSystemInDarkTheme()), theme = container.themePrefs.theme) {
                    when (state) {
                        is ShareState.Ready -> SavingSheet(state, container)
                        else -> ShareProblem(
                            state,
                            onAction = {
                                if (state == ShareState.NoLogin) startActivity(Intent(this@ShareActivity, MainActivity::class.java))
                                leave()
                            },
                            onClose = ::leave,
                        )
                    }
                }
            }
        }
    }

    @Composable
    private fun SavingSheet(ready: ShareState.Ready, container: AppContainer) {
        val scope = container.shareScope
        val library = container.library
        val online = remember { container.connectivity.isOnline() }
        val controller = remember {
            ShareController(scope, library, activity = container.uploadActivity) { uri ->
                withContext(Dispatchers.IO) {
                    val shared = Uri.parse(uri)
                    if (!GalleryAccess.mayRead(shared, shareGranted)) return@withContext null
                    galleryUploadOf(contentResolver, shared, GalleryAccess.granted(applicationContext), GalleryAccess.photosIsGenuine(applicationContext))
                        ?: copiedUploadOf(contentResolver, shared, sharedDirectory())
                }
            }.also { it.start(ready.items) }
        }
        val state = remember { ShareSheetState(ready.items) }
        val nearby = remember { mutableListOf<Card>() }
        val suggest = {
            if (!state.touched) state.suggestion = state.cards.firstOrNull()?.let { suggestSpace(it, nearby, state.spaces) }
        }
        val reread: suspend () -> Unit = {
            val ids = state.cards.map { it.id }.ifEmpty { null }
            if (ids != null) {
                state.cards = ids.mapIndexed { at, id ->
                    val card = (library.card(id) as? ApiResult.Ok)?.value ?: state.cards[at]
                    // A repeat save folded into an earlier card shows that card from now on.
                    card.mergedInto?.let { earlier -> (library.card(earlier) as? ApiResult.Ok)?.value?.also { state.merged = true } } ?: card
                }
                suggest()
            }
        }
        LaunchedEffect(Unit) {
            state.files = withContext(Dispatchers.IO) {
                ready.items.filterIsInstance<ShareItem.Stream>().mapNotNull { item ->
                    uploadFileOf(contentResolver, Uri.parse(item.uri))?.let { SharedFile(item.uri, it.name, it.mime, it.size) }
                }
            }
        }
        LaunchedEffect(Unit) {
            state.allTags = (library.tags() as? ApiResult.Ok)?.value.orEmpty()
            state.recentTags = (library.recentTags() as? ApiResult.Ok)?.value.orEmpty()
            state.spaces = (library.spaces() as? ApiResult.Ok)?.value?.spaces.orEmpty()
            nearby += (library.cards(null) as? ApiResult.Ok)?.value?.cards.orEmpty()
            suggest()
        }
        LaunchedEffect(Unit) {
            controller.progress.collect { progress ->
                val fresh = progress.saved.drop(state.cards.size).map { id -> (library.card(id) as? ApiResult.Ok)?.value ?: placeholderOf(id) }
                state.cards = state.cards + fresh
                state.saved = progress.saved.size
                state.failed = progress.failed
                state.problem = progress.problem
                state.phase = when {
                    !progress.done -> SavePhase.Saving
                    progress.saved.isEmpty() -> SavePhase.Failed
                    !online && ready.items.all { it is ShareItem.Text } -> SavePhase.Kept
                    else -> SavePhase.Saved
                }
                suggest()
            }
        }
        // While the sheet is open, the server reads what was saved; each sync brings its title, picture and tags.
        // ThyMind's tags are looked for every READ_GAP_MS, and the sheet waits for them up to TAG_READS reads.
        LaunchedEffect(Unit) {
            if (!online) return@LaunchedEffect
            state.findingTags = ready.items.size == 1
            for (read in 1..READS) {
                delay(READ_GAP_MS)
                container.librarySync.sync()
                reread()
                if (state.findingTags && (thymindTags(state.cards.firstOrNull()).isNotEmpty() || read >= TAG_READS)) state.findingTags = false
            }
            state.findingTags = false
        }
        val finish = { details: ShareDetails -> if (!details.isEmpty()) scope.launch { controller.finish(details) } }
        if (!rememberSheetMotionEnabled()) {
            ShareSheet(state, onClose = { finish(state.details()); leave() })
            return
        }
        val started = remember { System.currentTimeMillis() }
        ShareMoment(
            state,
            count = { countFind(container, ready.items.firstOrNull(), state, started) },
            onDetails = { finish(it) },
            onSky = ::blurBehind,
            onClosed = ::leave,
        )
    }

    /**
     * Counts the library for what was shared: how many earlier cards came from the same source, and how many the
     * gallery showed before, leaving out what this share saved (find-rarity-from-library).
     */
    private suspend fun countFind(container: AppContainer, item: ShareItem?, state: ShareSheetState, started: Long): FindCount? {
        val cards = withContext(Dispatchers.IO) { container.libraryCopy.gallery() } ?: return null
        val source = when (item) {
            is ShareItem.Text -> sourceOfShare(item.text.trim().takeIf { it.startsWith("http") }, null)
            is ShareItem.Stream -> sourceOfShare(null, runCatching { contentResolver.getType(Uri.parse(item.uri)) }.getOrNull() ?: "")
            null -> return null
        }
        val saved = state.cards.mapTo(HashSet()) { it.id }
        // A save can reach the copy before its id reaches the sheet; anything made since the share began is this one.
        val earlier = cards.filter { it.id !in saved && (createdAt(it.created) ?: 0L) < started - 2_000L }
        return FindCount(rarityIn(earlier, source), earlier.size)
    }

    /** Blurs the app that shared behind the sky as it comes in, where the phone can blur behind a window. */
    private fun blurBehind(amount: Float) {
        val windows = getSystemService(WindowManager::class.java) ?: return
        if (!windows.isCrossWindowBlurEnabled) return
        val radius = (amount * BLUR_DP * resources.displayMetrics.density).roundToInt()
        if (radius == blurShown) return
        blurShown = radius
        window.attributes = window.attributes.apply {
            blurBehindRadius = radius
            flags = if (radius > 0) flags or WindowManager.LayoutParams.FLAG_BLUR_BEHIND else flags and WindowManager.LayoutParams.FLAG_BLUR_BEHIND.inv()
        }
    }
    private var blurShown = 0

    /** Closes without the window's own animation, since the sheet has already gone. */
    private fun leave() {
        finish()
        @Suppress("DEPRECATION")
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.UPSIDE_DOWN_CAKE) overridePendingTransition(0, 0)
    }

    /** Folder of the copies of shared files, without copies left over from a day ago or longer. */
    private fun sharedDirectory(): File = File(cacheDir, SHARED_DIRECTORY).also { directory ->
        val old = System.currentTimeMillis() - STALE_COPY_MS
        directory.listFiles()?.filter { it.lastModified() < old }?.forEach { it.delete() }
    }

    private companion object {
        const val SHARED_DIRECTORY = "shared"
        const val STALE_COPY_MS = 24L * 60 * 60 * 1000
        const val READS = 16
        const val TAG_READS = 10
        const val READ_GAP_MS = 750L
        const val BLUR_DP = 24f
    }
}

/** @return When a card was made, in milliseconds, or null when its time cannot be read. */
private fun createdAt(created: String): Long? = runCatching { Instant.parse(created).toEpochMilli() }.getOrNull()

/** A card the library cannot give yet, so the saved row still has one to fill in. */
private fun placeholderOf(id: String) = Card(id = id, type = "Placeholder", flags = 0, created = "", modified = "", bumped = "")

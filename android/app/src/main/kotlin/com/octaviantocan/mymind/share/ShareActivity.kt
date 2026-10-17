package com.octaviantocan.mymind.share

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.lifecycle.lifecycleScope
import com.octaviantocan.mymind.MainActivity
import com.octaviantocan.mymind.MymindApp
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.data.copiedUploadOf
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareProblem
import com.octaviantocan.mymind.ui.share.ShareSheet
import com.octaviantocan.mymind.ui.share.ShareSheetState
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import androidx.compose.foundation.isSystemInDarkTheme

/** Receives text, links, pictures, videos and PDFs shared from other apps and saves them to the library. */
class ShareActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val container = (application as MymindApp).container
        val intent = intent
        lifecycleScope.launch {
            container.session.restore()
            val signedIn = container.session.state.value is SessionState.SignedIn
            val state = parseShare(requestOf(intent), signedIn, ContentProbe(contentResolver))
            setContent {
                MymindTheme(dark = isSystemInDarkTheme()) {
                    when (state) {
                        is ShareState.Ready -> SavingSheet(state, container)
                        else -> ShareProblem(
                            state,
                            onAction = {
                                if (state == ShareState.NoLogin) startActivity(Intent(this@ShareActivity, MainActivity::class.java))
                                finish()
                            },
                            onClose = { finish() },
                        )
                    }
                }
            }
        }
    }

    @Composable
    private fun SavingSheet(ready: ShareState.Ready, container: com.octaviantocan.mymind.AppContainer) {
        val scope = container.shareScope
        val controller = remember {
            ShareController(scope, container.repository, activity = container.uploadActivity) { uri ->
                withContext(Dispatchers.IO) { copiedUploadOf(contentResolver, Uri.parse(uri), sharedDirectory()) }
            }.also { it.start(ready.items) }
        }
        val state = remember { ShareSheetState() }
        LaunchedEffect(Unit) {
            val library = container.repository
            state.allTags = (library.tags() as? ApiResult.Ok)?.value.orEmpty()
            state.recentTags = (library.recentTags() as? ApiResult.Ok)?.value.orEmpty()
            state.spaces = (library.spaces() as? ApiResult.Ok)?.value?.spaces.orEmpty()
        }
        LaunchedEffect(Unit) { controller.queue.state.collect { state.uploads = it } }
        ShareSheet(
            state,
            onSave = {
                state.phase = SavePhase.Saving
                scope.launch {
                    when (controller.finish(state.details())) {
                        is ApiResult.Ok -> {
                            state.phase = SavePhase.Saved
                            delay(CLOSE_DELAY_MS)
                            finish()
                        }
                        is ApiResult.Failed -> state.phase = SavePhase.Failed
                    }
                }
            },
            onToggleUploads = controller.queue::toggleDetails,
            onDismissUpload = controller.queue::dismiss,
        )
    }

    /** Folder of the copies of shared files, without copies left over from a day ago or longer. */
    private fun sharedDirectory(): File = File(cacheDir, SHARED_DIRECTORY).also { directory ->
        val old = System.currentTimeMillis() - STALE_COPY_MS
        directory.listFiles()?.filter { it.lastModified() < old }?.forEach { it.delete() }
    }

    private companion object {
        const val CLOSE_DELAY_MS = 1_200L
        const val SHARED_DIRECTORY = "shared"
        const val STALE_COPY_MS = 24L * 60 * 60 * 1000
    }
}

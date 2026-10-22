package com.octaviantocan.mymind

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.widget.Toast
import coil3.SingletonImageLoader
import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.Connectivity
import com.octaviantocan.mymind.data.KeystoreTokenStore
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.LibrarySync
import com.octaviantocan.mymind.data.LocalFirstLibrary
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Outbox
import com.octaviantocan.mymind.data.OutboxWorker
import com.octaviantocan.mymind.data.PictureWorker
import com.octaviantocan.mymind.data.PrivateAccountProfileStore
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.data.refusedNotice
import com.octaviantocan.mymind.ui.menu.DownloadsSaver
import com.octaviantocan.mymind.update.AutoInstaller
import com.octaviantocan.mymind.update.SessionApkInstaller
import com.octaviantocan.mymind.update.UploadActivity
import com.octaviantocan.mymind.update.UploadService
import com.octaviantocan.mymind.update.UpdateNotifier
import com.octaviantocan.mymind.update.UpdatePrefs
import com.octaviantocan.mymind.update.Updater
import com.octaviantocan.mymind.update.WhatsNew
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import java.io.File

/** Manual dependency graph, created once per process. */
class AppContainer(context: Context) {
    val connectivity = Connectivity(context)
    val session = Session(KeystoreTokenStore(context))
    val httpClient: OkHttpClient = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session, BuildConfig.SERVER_URL)).build()
    val repository = Repository(MymindApi(httpClient, BuildConfig.SERVER_URL), session, PrivateAccountProfileStore(context))

    // The phone's copy of the library, its waiting changes and its pictures belong to one session and are wiped
    // whenever the app is signed out.
    val libraryCopy = LibraryCopy(context, session).also { copy ->
        val app = context.applicationContext
        CoroutineScope(SupervisorJob() + Dispatchers.Default).launch {
            session.state.collect { state ->
                if (state != SessionState.SignedOut) return@collect
                OutboxWorker.cancel(app)
                PictureWorker.cancel(app)
                copy.clear()
                withContext(Dispatchers.IO) { SingletonImageLoader.get(app).run { memoryCache?.clear(); diskCache?.clear() } }
            }
        }
    }
    private val main = Handler(Looper.getMainLooper())

    /** Changes to cards made without network, sent once the phone is online; a refused one shows a short notice. */
    val outbox = Outbox(
        libraryCopy,
        repository,
        onQueued = { OutboxWorker.schedule(context.applicationContext) },
        onRefused = { _, card -> main.post { Toast.makeText(context.applicationContext, refusedNotice(card), Toast.LENGTH_LONG).show() } },
    )
    val librarySync = LibrarySync(libraryCopy, repository::changes, outbox::send, onChanged = { PictureWorker.schedule(context.applicationContext) })

    /** The library the signed-in screens use: the phone's copy first where it has what they need, the server otherwise. */
    val library = LocalFirstLibrary(repository, libraryCopy, librarySync, outbox)
    val downloads = DownloadsSaver(context.applicationContext.contentResolver)
    val updatePrefs = UpdatePrefs(context)
    val updateNotifier = UpdateNotifier(context.applicationContext)
    val updateScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    // Shares are saved here, not in the sheet, so leaving the sheet does not cancel an upload.
    val shareScope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    val updater = Updater(
        client = httpClient,
        downloadsUrl = BuildConfig.DOWNLOADS_URL,
        installedCode = BuildConfig.VERSION_CODE,
        directory = File(context.cacheDir, "updates"),
        installer = SessionApkInstaller(context.applicationContext),
        prefs = updatePrefs,
    )
    val uploadActivity = UploadActivity(onStart = { UploadService.start(context.applicationContext) })
    val autoInstaller = AutoInstaller(
        updater,
        canInstall = { context.packageManager.canRequestPackageInstalls() },
        uploads = uploadActivity,
        askToAllow = updateNotifier::allowNeeded,
    )
    val whatsNew = WhatsNew(
        updatePrefs,
        BuildConfig.VERSION_CODE,
        runCatching { context.assets.open("CHANGELOG.md").bufferedReader().use { it.readText() } }.getOrDefault(""),
    )
}

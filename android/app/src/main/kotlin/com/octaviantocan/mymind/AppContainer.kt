package com.octaviantocan.mymind

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.widget.Toast
import coil3.SingletonImageLoader
import com.octaviantocan.mymind.data.AiAccount
import com.octaviantocan.mymind.data.AiClient
import com.octaviantocan.mymind.data.AiPrefs
import com.octaviantocan.mymind.data.Analytics
import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.Connectivity
import com.octaviantocan.mymind.data.FeedbackClient
import com.octaviantocan.mymind.data.HubListener
import com.octaviantocan.mymind.data.KeystoreTokenStore
import com.octaviantocan.mymind.data.LibraryCopy
import com.octaviantocan.mymind.data.LibrarySync
import com.octaviantocan.mymind.data.LocalFirstLibrary
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Outbox
import com.octaviantocan.mymind.data.OutboxWorker
import com.octaviantocan.mymind.data.PictureWorker
import com.octaviantocan.mymind.data.PrivateAccountProfileStore
import com.octaviantocan.mymind.data.RecentSearches
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.data.refusedNotice
import com.octaviantocan.mymind.feedback.FeedbackNotifier
import com.octaviantocan.mymind.feedback.FeedbackStore
import com.octaviantocan.mymind.feedback.FeedbackTracker
import com.octaviantocan.mymind.feedback.FeedbackWorker
import com.octaviantocan.mymind.ui.menu.DownloadsSaver
import com.octaviantocan.mymind.ui.theme.ThemePrefs
import com.octaviantocan.mymind.update.SessionApkInstaller
import com.octaviantocan.mymind.update.UploadActivity
import com.octaviantocan.mymind.update.UploadService
import com.octaviantocan.mymind.update.UpdateInstaller
import com.octaviantocan.mymind.update.UpdateNotifier
import com.octaviantocan.mymind.update.UpdatePrefs
import com.octaviantocan.mymind.update.UpdateWorker
import com.octaviantocan.mymind.update.Updater
import com.octaviantocan.mymind.update.WhatsNew
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import java.io.File

/** Manual dependency graph, created once per process. */
private const val EVENTS_EVERY_MS = 60_000L

class AppContainer(context: Context) {
    val connectivity = Connectivity(context)
    val session = Session(KeystoreTokenStore(context))
    val httpClient: OkHttpClient = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session, BuildConfig.SERVER_URL)).build()
    val repository = Repository(MymindApi(httpClient, BuildConfig.SERVER_URL), session, PrivateAccountProfileStore(context))

    /** The account's own Claude Code and Codex, for the Profile sheet and AI search. */
    val ai = AiAccount(AiClient(httpClient, BuildConfig.SERVER_URL), AiPrefs(context), session)

    /** The feedback jobs this phone waits on. */
    val feedbackStore = FeedbackStore(context)

    /** Sends feedback and follows its jobs: their notification, the background check and the update check on a release. */
    val feedback = FeedbackTracker(
        api = FeedbackClient(httpClient, BuildConfig.SERVER_URL),
        store = feedbackStore,
        notifier = FeedbackNotifier(context.applicationContext),
        onShipped = { UpdateWorker.checkNow(context.applicationContext) },
        schedule = { FeedbackWorker.schedule(context.applicationContext) },
    )

    /** The colour theme picked on this phone. */
    val themePrefs = ThemePrefs(context)

    /** The searches last pressed on this phone, for the opened search page. */
    val recentSearches = RecentSearches(context)

    /** The phone's own usage events (`docs/analytics.md`), kept apart from the outbox and sent about once a minute. */
    val analytics = Analytics(File(context.filesDir, "events.json")).also { Track.sink = it }
    private val analyticsScope = CoroutineScope(SupervisorJob() + Dispatchers.IO).also { scope ->
        scope.launch {
            while (true) {
                delay(EVENTS_EVERY_MS)
                sendEvents()
            }
        }
    }

    /** Sends the waiting usage events now when signed in and online; otherwise only writes them to their file. */
    fun sendEvents() {
        analyticsScope.launch {
            if (session.state.value is SessionState.SignedIn && connectivity.isOnline()) {
                analytics.send(BuildConfig.VERSION_NAME, repository::events)
            } else {
                analytics.fold()
            }
        }
    }

    // The phone's copy of the library, its waiting changes, its pictures and its recent searches belong to one
    // session and are wiped whenever the app is signed out.
    val libraryCopy = LibraryCopy(context, session).also { copy ->
        val app = context.applicationContext
        CoroutineScope(SupervisorJob() + Dispatchers.Default).launch {
            session.state.collect { state ->
                if (state != SessionState.SignedOut) return@collect
                OutboxWorker.cancel(app)
                PictureWorker.cancel(app)
                copy.clear()
                recentSearches.clear()
                analytics.clear()
                withContext(Dispatchers.IO) { SingletonImageLoader.get(app).run { memoryCache?.clear(); diskCache?.clear() } }
            }
        }
    }
    private val main = Handler(Looper.getMainLooper())

    /** Changes made on the phone, sent through the server's push once it can be reached; a refused one shows a short notice. */
    val outbox = Outbox(
        libraryCopy,
        repository,
        onQueued = { OutboxWorker.schedule(context.applicationContext) },
        onRefused = { edit, card -> main.post { Toast.makeText(context.applicationContext, refusedNotice(edit, card), Toast.LENGTH_LONG).show() } },
        push = repository::push,
    )
    val librarySync = LibrarySync(libraryCopy, repository::changes, outbox::send, onChanged = { PictureWorker.schedule(context.applicationContext) })

    /** The library the signed-in screens use: the phone's copy first where it has what they need, the server otherwise. */
    val library = LocalFirstLibrary(repository, libraryCopy, librarySync, outbox)

    /** The server's announcements of changes, heard while the app is on screen. */
    val hub = HubListener(httpClient, BuildConfig.SERVER_URL, CoroutineScope(SupervisorJob() + Dispatchers.Default))
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
    val updateInstaller = UpdateInstaller(
        updater,
        canInstall = { context.packageManager.canRequestPackageInstalls() },
        uploads = uploadActivity,
    )
    val whatsNew = WhatsNew(
        updatePrefs,
        BuildConfig.VERSION_CODE,
        runCatching { context.assets.open("CHANGELOG.md").bufferedReader().use { it.readText() } }.getOrDefault(""),
    )
}

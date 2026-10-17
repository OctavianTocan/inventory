package com.octaviantocan.mymind

import android.content.Context
import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.Connectivity
import com.octaviantocan.mymind.data.KeystoreTokenStore
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.ui.menu.DownloadsSaver
import com.octaviantocan.mymind.update.SessionApkInstaller
import com.octaviantocan.mymind.update.UpdateNotifier
import com.octaviantocan.mymind.update.UpdatePrefs
import com.octaviantocan.mymind.update.Updater
import com.octaviantocan.mymind.update.WhatsNew
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import okhttp3.OkHttpClient
import java.io.File

/** Manual dependency graph, created once per process. */
class AppContainer(context: Context) {
    val connectivity = Connectivity(context)
    val session = Session(KeystoreTokenStore(context))
    val httpClient: OkHttpClient = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session)).build()
    val repository = Repository(MymindApi(httpClient, BuildConfig.SERVER_URL), session)
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
    val whatsNew = WhatsNew(
        updatePrefs,
        BuildConfig.VERSION_CODE,
        runCatching { context.assets.open("CHANGELOG.md").bufferedReader().use { it.readText() } }.getOrDefault(""),
    )
}

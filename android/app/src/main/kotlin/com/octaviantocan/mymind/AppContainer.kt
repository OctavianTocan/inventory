package com.octaviantocan.mymind

import android.content.Context
import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.Connectivity
import com.octaviantocan.mymind.data.KeystoreTokenStore
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.ui.menu.DownloadsSaver
import okhttp3.OkHttpClient

/** Manual dependency graph, created once per process. */
class AppContainer(context: Context) {
    val connectivity = Connectivity(context)
    val session = Session(KeystoreTokenStore(context))
    val httpClient: OkHttpClient = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session)).build()
    val repository = Repository(MymindApi(httpClient, BuildConfig.SERVER_URL), session)
    val downloads = DownloadsSaver(context.applicationContext.contentResolver)
}

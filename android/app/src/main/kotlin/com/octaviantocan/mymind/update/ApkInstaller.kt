package com.octaviantocan.mymind.update

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

/** Hands a verified package to Android. */
interface ApkInstaller {
    /**
     * Starts the install of [file]; Android reports the outcome to [InstallStatusReceiver].
     *
     * @param file Verified package.
     * @param beforeCommit Called with the session's id before it is committed, since Android can answer at once.
     * @throws java.io.IOException when the session cannot be written or committed.
     */
    suspend fun install(file: File, beforeCommit: (Int) -> Unit)
}

/** [ApkInstaller] that commits a `PackageInstaller` session which asks for no user action where Android allows it. */
class SessionApkInstaller(private val context: Context) : ApkInstaller {
    override suspend fun install(file: File, beforeCommit: (Int) -> Unit) = withContext(Dispatchers.IO) {
        val installer = context.packageManager.packageInstaller
        installer.mySessions.forEach { runCatching { installer.abandonSession(it.sessionId) } }
        val params = PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL).apply {
            setAppPackageName(context.packageName)
            setSize(file.length())
            setRequireUserAction(PackageInstaller.SessionParams.USER_ACTION_NOT_REQUIRED)
        }
        val id = installer.createSession(params)
        installer.openSession(id).use { session ->
            file.inputStream().use { input ->
                session.openWrite(SESSION_FILE, 0, file.length()).use { output ->
                    input.copyTo(output)
                    session.fsync(output)
                }
            }
            beforeCommit(id)
            val callback = Intent(context, InstallStatusReceiver::class.java).setPackage(context.packageName)
                .putExtra(PackageInstaller.EXTRA_SESSION_ID, id)
            val pending = PendingIntent.getBroadcast(context, id, callback, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE)
            session.commit(pending.intentSender)
        }
    }

    private companion object {
        const val SESSION_FILE = "mymind.apk"
    }
}

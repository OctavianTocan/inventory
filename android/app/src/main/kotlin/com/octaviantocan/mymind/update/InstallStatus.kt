package com.octaviantocan.mymind.update

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import androidx.core.content.IntentCompat
import com.octaviantocan.mymind.MymindApp

/** What to do with the outcome Android reported for a commit. */
sealed interface StatusAction {
    /** Nothing more to do. */
    data object None : StatusAction

    /**
     * Show the system install prompt now.
     *
     * @property intent The prompt's activity intent.
     */
    data class OpenPrompt(val intent: Intent) : StatusAction

    /**
     * Tell the user in a notification that a tap is needed.
     *
     * @property intent The prompt's activity intent.
     */
    data class NotifyPrompt(val intent: Intent) : StatusAction
}

/** Receives the status of the committed install session and passes it to the updater. */
class InstallStatusReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val container = (context.applicationContext as MymindApp).container
        val status = intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)
        val message = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE)
        val confirm = IntentCompat.getParcelableExtra(intent, Intent.EXTRA_INTENT, Intent::class.java)
        when (val action = container.updater.onInstallStatus(status, message, confirm)) {
            is StatusAction.OpenPrompt -> context.startActivity(action.intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            is StatusAction.NotifyPrompt -> container.updateNotifier.promptNeeded(container.updater.currentRelease(), action.intent)
            StatusAction.None -> Unit
        }
    }
}

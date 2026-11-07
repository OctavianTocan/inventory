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

    /** The status is about a session the app no longer follows, such as one a retry abandoned; it is ignored. */
    data object Stale : StatusAction

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
        val session = intent.getIntExtra(PackageInstaller.EXTRA_SESSION_ID, -1)
        // The prompt is never started from here: the visible update screen opens it, so Android's rules for starting
        // activities from the background cannot hide or break it.
        when (val action = container.updater.onInstallStatus(status, message, confirm, session)) {
            StatusAction.Stale -> Unit
            is StatusAction.NotifyPrompt -> container.updateNotifier.promptNeeded(container.updater.currentRelease(), action.intent)
            // A cancel is the user's own answer to Android's prompt, so only other failures tell a user who left.
            StatusAction.None -> if (status != PackageInstaller.STATUS_FAILURE_ABORTED && container.updater.state.value is UpdateState.Failed && !container.updater.foreground) {
                container.updateNotifier.stopped(container.updater.currentRelease())
            }
        }
    }
}

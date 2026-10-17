package com.octaviantocan.mymind.update

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings

/**
 * Wording of the "Check for updates" row for a state.
 *
 * @param state The updater's state.
 * @param installAllowed Whether Android lets this app install packages.
 * @return Short text for the trailing side of the row.
 */
fun updateStatusText(state: UpdateState, installAllowed: Boolean = true): String = when (state) {
    UpdateState.None -> ""
    UpdateState.UpToDate -> "Up to date"
    UpdateState.Checking -> "Checking"
    is UpdateState.Downloading -> "Downloading ${(state.progress * 100).toInt()}%"
    is UpdateState.Ready ->
        if (installAllowed) "Installs when you leave. Tap to install now"
        else "${state.release.versionName} is ready, allow installs below"
    is UpdateState.Installing -> "Installing"
    is UpdateState.Failed -> "Failed: ${state.reason}"
}

/**
 * The system screen where the user allows this app to install packages.
 *
 * @param context Any context of the app.
 * @return Intent for `Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES` of this package.
 */
fun allowInstallsIntent(context: Context): Intent =
    Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${context.packageName}")).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

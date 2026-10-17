package com.octaviantocan.mymind.update

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner

/**
 * Follows whether any activity of the app is started, and reacts when the app leaves the screen. Attach it to the
 * process lifecycle so a configuration change does not count as leaving.
 *
 * @param updater Receives the on-screen flag.
 * @param onLeft Called each time the app leaves the screen.
 */
class ForegroundTracker(private val updater: Updater, private val onLeft: () -> Unit) : DefaultLifecycleObserver {
    override fun onStart(owner: LifecycleOwner) {
        updater.foreground = true
    }

    override fun onStop(owner: LifecycleOwner) {
        updater.foreground = false
        onLeft()
    }
}

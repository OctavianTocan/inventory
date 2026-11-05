package com.octaviantocan.mymind.update

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner

/**
 * Follows whether any activity of the app is started, so the install receiver knows whether it may open Android's
 * prompt itself and the background check knows whether to notify. Attach it to the process lifecycle so a
 * configuration change does not count as leaving.
 *
 * @param updater Receives the on-screen flag.
 */
class ForegroundTracker(private val updater: Updater) : DefaultLifecycleObserver {
    override fun onStart(owner: LifecycleOwner) {
        updater.foreground = true
    }

    override fun onStop(owner: LifecycleOwner) {
        updater.foreground = false
    }
}

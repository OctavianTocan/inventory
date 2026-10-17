package com.octaviantocan.mymind.ui

import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.ui.test.junit4.ComposeTestRule

/** Runs [change] on the UI thread and publishes its state writes, so the next manual clock advance recomposes. */
fun ComposeTestRule.change(change: () -> Unit) = runOnIdle {
    change()
    Snapshot.sendApplyNotifications()
}

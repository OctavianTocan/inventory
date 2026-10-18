package com.octaviantocan.mymind.update

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.getAndUpdate
import kotlinx.coroutines.flow.update

/**
 * Counts the app's uploads in flight, so an update does not interrupt them and the app stays running for them.
 *
 * @param onStart Called when an upload starts while none was in flight.
 */
class UploadActivity(private val onStart: () -> Unit = {}) {
    private val mutableCount = MutableStateFlow(0)

    /** Number of uploads in flight. */
    val count: StateFlow<Int> = mutableCount

    /** Counts one more upload in flight; every call is paired with one [end]. */
    fun begin() {
        if (mutableCount.getAndUpdate { it + 1 } == 0) onStart()
    }

    /** Counts one upload as finished, failed or cancelled. */
    fun end() = mutableCount.update { maxOf(0, it - 1) }

    /** @return True when any upload is in flight. */
    fun busy(): Boolean = mutableCount.value > 0
}

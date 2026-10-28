package com.octaviantocan.mymind.data

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async

/** How a sync ended. */
sealed interface SyncOutcome {
    /** @property changed True when the copy took in any card, removal or new start. */
    data class Done(val changed: Boolean) : SyncOutcome

    /** The server has no changes feed yet, so the app keeps reading the library page by page. */
    data object Unavailable : SyncOutcome

    /** @property failure Why the feed could not be read, such as no network. */
    data class Failed(val failure: ApiFailure) : SyncOutcome
}

/**
 * Brings the phone's copy of the library up to date from the server's changes feed, one answer at a time, each kept
 * whole so an interrupted sync resumes where it stopped. Changes waiting on the phone go to the server first, so the
 * feed's answers already hold them. Runs in its own scope, so a closed screen does not cut it off.
 *
 * @param copy The phone's copy.
 * @param fetch Reads one answer of the feed after a position, or from the start for null.
 * @param send Sends the changes waiting on the phone; null once none waits, else the failure that stopped it.
 * @param onChanged Called after a sync that changed the copy.
 * @param scope Scope the syncs run in.
 */
class LibrarySync(
    private val copy: LibraryCopy,
    private val fetch: suspend (since: String?) -> ApiResult<ChangesPage>,
    private val send: suspend () -> ApiFailure? = { null },
    private val onChanged: () -> Unit = {},
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) {
    private var running: Deferred<SyncOutcome>? = null

    // A sync started to follow the running one that has not begun yet; it will see every change made until it begins.
    private var waiting: Deferred<SyncOutcome>? = null

    /** False once the server answered that it has no changes feed; a later sync asks again. */
    @Volatile
    var available = true
        private set

    /**
     * Starts a sync, or joins the one running.
     *
     * @param afterCurrent True for a sync that begins after the running one ends, so it sees a write made since that
     *   one began; a sync already waiting to follow it is joined, since it begins later still.
     * @return The sync's outcome, when it ends.
     */
    fun start(afterCurrent: Boolean = false): Deferred<SyncOutcome> = synchronized(this) {
        val current = running?.takeIf { it.isActive }
        if (current != null && !afterCurrent) return current
        waiting?.takeIf { it.isActive }?.let { return it }
        lateinit var next: Deferred<SyncOutcome>
        next = scope.async(start = CoroutineStart.LAZY) {
            current?.join()
            synchronized(this@LibrarySync) { if (waiting === next) waiting = null }
            pull().also { if (it == SyncOutcome.Done(changed = true)) onChanged() }
        }
        running = next
        if (current != null) waiting = next
        next.also { it.start() }
    }

    /**
     * Syncs and waits for the outcome.
     *
     * @param afterCurrent As for [start].
     * @return How the sync ended.
     */
    suspend fun sync(afterCurrent: Boolean = false): SyncOutcome = start(afterCurrent).await()

    private suspend fun pull(): SyncOutcome {
        send()?.let { return SyncOutcome.Failed(it) }
        var changed = false
        var restart = false
        while (true) {
            val since = if (restart) null else copy.syncToken()
            val page = when (val answer = fetch(since)) {
                is ApiResult.Ok -> if (restart) answer.value.copy(reset = true) else answer.value
                is ApiResult.Failed -> when {
                    // The server no longer reads the kept position, so the copy starts over.
                    answer.failure == ApiFailure.Server(400) && since != null -> {
                        restart = true
                        continue
                    }
                    answer.failure is ApiFailure.Server -> {
                        available = false
                        return SyncOutcome.Unavailable
                    }
                    else -> return SyncOutcome.Failed(answer.failure)
                }
            }
            restart = false
            available = true
            if (!copy.apply(page)) return SyncOutcome.Done(changed)
            changed = changed || page.reset || page.cards.isNotEmpty() || page.removed.isNotEmpty()
            if (!page.more || page.next == since) return SyncOutcome.Done(changed)
        }
    }
}

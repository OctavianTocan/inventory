package com.octaviantocan.mymind.feedback

import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.FeedbackApi
import com.octaviantocan.mymind.data.FeedbackProblem
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.data.isFinal
import kotlinx.coroutines.CancellationException

/** State of the job that shipped a release. */
const val SHIPPED = "shipped"

/**
 * Sends feedback and follows its jobs for the send page and the background check alike: every answer is shown in the
 * job's one notification when its state changed, a job stays in the [FeedbackStore] until it reaches a final state,
 * and a job that shipped starts the update check.
 *
 * @param api The server's feedback routes.
 * @param store The jobs still waited on.
 * @param notifier Shows a job's notification.
 * @param onShipped Starts the update check, called once when a job shipped.
 * @param schedule Starts the background check that polls while any job is active.
 */
class FeedbackTracker(
    private val api: FeedbackApi,
    private val store: FeedbackStore,
    private val notifier: FeedbackNotifier,
    private val onShipped: () -> Unit,
    private val schedule: () -> Unit,
) {
    /** @return The id of the job the phone waits on, or null when none is active. */
    fun activeId(): String? = store.active().lastOrNull()

    /** @return True while any job has not reached a final state. */
    fun hasActive(): Boolean = store.hasActive()

    /**
     * Sends the feedback, then waits on its job, shows its notification and starts the background check.
     *
     * @param text The feedback.
     * @param choice The engine, model and effort to implement it with.
     * @param clientId Random id of this send attempt, the same for every retry of it.
     * @return The job as the server first describes it.
     * @throws FeedbackProblem When the server refuses or cannot be reached; nothing is kept then.
     */
    suspend fun send(text: String, choice: AiChoice, clientId: String): FeedbackStatus {
        val sent = api.send(text, choice, clientId)
        store.add(sent.id)
        schedule()
        return try {
            refresh(sent.id)
        } catch (e: CancellationException) {
            throw e
        } catch (_: FeedbackProblem) {
            handle(sent)
        }
    }

    /**
     * Asks where a job stands and acts on the answer.
     *
     * @param id The job's id.
     * @return The job's state.
     * @throws FeedbackProblem When the server refuses or cannot be reached; an unknown job is also dropped.
     */
    suspend fun refresh(id: String): FeedbackStatus {
        val status = try {
            api.status(id)
        } catch (e: FeedbackProblem) {
            if (e.notFound) store.remove(id)
            throw e
        }
        return handle(status)
    }

    /**
     * Asks about every active job.
     *
     * @return True when a job is still active afterwards, so the background check should run again.
     */
    suspend fun pollActive(): Boolean {
        for (id in store.active()) {
            try {
                refresh(id)
            } catch (e: CancellationException) {
                throw e
            } catch (_: FeedbackProblem) {
                // The next check asks again.
            }
        }
        return store.hasActive()
    }

    private fun handle(status: FeedbackStatus): FeedbackStatus {
        val changed = store.lastState(status.id) != status.state
        if (status.isFinal) store.remove(status.id)
        if (changed) {
            store.setLastState(status.id, status.state)
            notifier.show(status)
            if (status.state == SHIPPED) onShipped()
        }
        return status
    }
}

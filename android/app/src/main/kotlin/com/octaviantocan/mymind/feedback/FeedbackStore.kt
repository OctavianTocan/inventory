package com.octaviantocan.mymind.feedback

import android.content.Context
import android.content.SharedPreferences

/**
 * The feedback jobs this phone still waits on, and the last state each was seen in, kept in shared preferences so the
 * background check knows what to ask about after the app restarts.
 *
 * @param prefs The preferences file holding the ids.
 */
class FeedbackStore(private val prefs: SharedPreferences) {
    constructor(context: Context) : this(context.applicationContext.getSharedPreferences(NAME, Context.MODE_PRIVATE))

    /** @return The ids of jobs that have not reached a final state, oldest first. */
    fun active(): List<String> = prefs.getString(ACTIVE, "").orEmpty().split(SEPARATOR).filter { it.isNotEmpty() }

    /** @return True while any job is waiting for a final state. */
    fun hasActive(): Boolean = active().isNotEmpty()

    /**
     * Starts waiting on a job; one already waited on stays where it is.
     *
     * @param id The job's id.
     */
    fun add(id: String) {
        val now = active()
        if (id !in now) prefs.edit().putString(ACTIVE, (now + id).joinToString(SEPARATOR)).apply()
    }

    /**
     * Stops waiting on a job, such as when it reached a final state.
     *
     * @param id The job's id.
     */
    fun remove(id: String) {
        prefs.edit().putString(ACTIVE, (active() - id).joinToString(SEPARATOR)).apply()
    }

    /**
     * @param id The job's id.
     * @return The state the phone last showed for it, or null if never.
     */
    fun lastState(id: String): String? = prefs.getString(STATE + id, null)

    /**
     * Remembers the state shown for a job, so an unchanged state is not shown again.
     *
     * @param id The job's id.
     * @param state The state just shown.
     */
    fun setLastState(id: String, state: String) {
        prefs.edit().putString(STATE + id, state).apply()
    }

    private companion object {
        const val NAME = "feedback"
        const val ACTIVE = "active"
        const val STATE = "state."
        const val SEPARATOR = ","
    }
}

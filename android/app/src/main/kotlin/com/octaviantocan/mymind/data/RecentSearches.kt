package com.octaviantocan.mymind.data

import android.content.Context
import android.content.SharedPreferences
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/**
 * The last few searches pressed on this phone, newest first, which the opened search page offers again. They never
 * leave the phone and are wiped when the app signs out.
 */
class RecentSearches(private val prefs: SharedPreferences) {
    constructor(context: Context) : this(context.applicationContext.getSharedPreferences(NAME, Context.MODE_PRIVATE))

    private val mutable = MutableStateFlow(read())

    /** The searches, newest first, at most [KEPT]. */
    val searches: StateFlow<List<String>> = mutable

    /**
     * Puts [text] first, dropping an older copy of it that differs only in case and the oldest search past [KEPT].
     *
     * @param text The search as it was pressed; blank text is ignored.
     */
    fun add(text: String) {
        val clean = text.trim().replace(Regex("\\s+"), " ")
        if (clean.isEmpty()) return
        write(listOf(clean) + mutable.value.filterNot { it.equals(clean, ignoreCase = true) })
    }

    /** Forgets every search. */
    fun clear() = write(emptyList())

    private fun read(): List<String> = prefs.getString(KEY, null).orEmpty().split('\n').filter { it.isNotBlank() }.take(KEPT)

    private fun write(searches: List<String>) {
        val kept = searches.take(KEPT)
        mutable.value = kept
        prefs.edit().putString(KEY, kept.joinToString("\n")).apply()
    }

    companion object {
        /** How many searches are kept. */
        const val KEPT = 5
        private const val NAME = "recent-searches"
        private const val KEY = "searches"
    }
}

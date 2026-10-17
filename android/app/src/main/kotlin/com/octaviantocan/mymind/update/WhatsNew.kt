package com.octaviantocan.mymind.update

import com.octaviantocan.mymind.changelog.Changelog
import com.octaviantocan.mymind.changelog.ChangelogEntry

/**
 * The bundled changelog and which version's notes the user still has to see.
 *
 * @param prefs Remembers the last version whose notes were handled.
 * @param installedCode `versionCode` of the running app.
 * @param changelogText Text of the bundled `CHANGELOG.md`.
 */
class WhatsNew(private val prefs: UpdatePrefs, private val installedCode: Int, changelogText: String) {
    /** Every version of the bundled changelog, newest first; empty when the file is unreadable. */
    val entries: List<ChangelogEntry> = runCatching { Changelog.parse(changelogText) }.getOrDefault(emptyList())

    /**
     * The notes of the running version when it is new to this install, and marks them handled so they show once. A
     * fresh install has nothing to show.
     *
     * @return The running version's entry, or null.
     */
    fun takePending(): ChangelogEntry? {
        val seen = prefs.shownChangelogCode
        prefs.shownChangelogCode = installedCode
        if (seen == null || seen >= installedCode) return null
        return entries.firstOrNull { it.versionCode == installedCode }
    }
}

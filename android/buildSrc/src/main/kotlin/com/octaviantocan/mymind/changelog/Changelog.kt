package com.octaviantocan.mymind.changelog

/**
 * One version of the changelog.
 *
 * @property versionName Dotted version such as `0.2.0`.
 * @property versionCode Integer build number, higher for every release.
 * @property date Release date as `YYYY-MM-DD`.
 * @property notes The version's lines, without the leading dash.
 */
data class ChangelogEntry(val versionName: String, val versionCode: Int, val date: String, val notes: List<String>)

/** A changelog that breaks the format; the message names the line. */
class ChangelogException(message: String) : RuntimeException(message)

/** Reads `CHANGELOG.md`, the single source of the app's version. */
object Changelog {
    private val HEADING = Regex("""^## (\d+)\.(\d+)\.(\d+) \((\d+)\), (\d{4}-\d{2}-\d{2})$""")

    /**
     * Parses the changelog text and checks that versions rise from the oldest entry to the newest.
     *
     * @param text Contents of the changelog file.
     * @param source Name of the file, used in error messages.
     * @return The entries, newest first.
     * @throws ChangelogException when the file has no entry, a heading does not match
     * `## <major>.<minor>.<patch> (<code>), <YYYY-MM-DD>`, an entry has no `- ` line, or a version does not rise.
     */
    fun parse(text: String, source: String = "CHANGELOG.md"): List<ChangelogEntry> {
        val entries = mutableListOf<ChangelogEntry>()
        var heading: Triple<MatchResult, Int, String>? = null
        var notes = mutableListOf<String>()
        val lines = text.lines()

        fun close() {
            val (match, line, raw) = heading ?: return
            if (notes.isEmpty()) fail(source, line, "the entry \"$raw\" has no \"- \" line")
            val (major, minor, patch, code, date) = match.destructured
            val codeNumber = code.toIntOrNull() ?: fail(source, line, "version code $code is too large")
            entries += ChangelogEntry("$major.$minor.$patch", codeNumber, date, notes)
            notes = mutableListOf()
        }

        lines.forEachIndexed { index, raw ->
            val line = index + 1
            val trimmed = raw.trimEnd()
            when {
                trimmed.startsWith("## ") -> {
                    close()
                    val match = HEADING.matchEntire(trimmed)
                        ?: fail(source, line, "expected \"## <major>.<minor>.<patch> (<code>), <YYYY-MM-DD>\" but found \"$trimmed\"")
                    heading = Triple(match, line, trimmed)
                }
                trimmed.startsWith("- ") -> {
                    if (heading == null) fail(source, line, "a \"- \" line comes before the first \"## \" heading")
                    val note = trimmed.removePrefix("- ").trim()
                    if (note.isEmpty()) fail(source, line, "the \"- \" line is empty")
                    notes += note
                }
            }
        }
        close()
        if (entries.isEmpty()) fail(source, 1, "no \"## \" entry found")
        entries.zipWithNext().forEach { (newer, older) ->
            if (newer.versionCode <= older.versionCode) {
                fail(source, 1, "version code ${newer.versionCode} of ${newer.versionName} is not higher than ${older.versionCode} of ${older.versionName}")
            }
            if (compareVersions(newer.versionName, older.versionName) <= 0) {
                fail(source, 1, "version ${newer.versionName} is not higher than ${older.versionName}")
            }
        }
        return entries
    }

    /**
     * Orders two dotted versions numerically.
     *
     * @param a First version.
     * @param b Second version.
     * @return A negative number when [a] is lower, zero when equal, a positive number when [a] is higher.
     */
    fun compareVersions(a: String, b: String): Int {
        val left = a.split('.').map { it.toIntOrNull() ?: 0 }
        val right = b.split('.').map { it.toIntOrNull() ?: 0 }
        for (i in 0 until maxOf(left.size, right.size)) {
            val diff = left.getOrElse(i) { 0 }.compareTo(right.getOrElse(i) { 0 })
            if (diff != 0) return diff
        }
        return 0
    }

    private fun fail(source: String, line: Int, reason: String): Nothing =
        throw ChangelogException("$source:$line: $reason")
}

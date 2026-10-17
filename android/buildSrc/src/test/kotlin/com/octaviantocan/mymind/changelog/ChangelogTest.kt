package com.octaviantocan.mymind.changelog

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class ChangelogTest {
    private val good = """
        # Changelog

        ## 0.2.0 (2), 2026-10-17
        - Signs in.
        - Downloads updates.

        ## 0.1.0 (1), 2026-10-16
        - First debug builds.
    """.trimIndent()

    private fun failure(text: String): String {
        try {
            Changelog.parse(text)
        } catch (error: ChangelogException) {
            return error.message.orEmpty()
        }
        fail("expected a ChangelogException")
        return ""
    }

    @Test
    fun readsEntriesNewestFirst() {
        val entries = Changelog.parse(good)
        assertEquals(listOf("0.2.0", "0.1.0"), entries.map { it.versionName })
        assertEquals(listOf(2, 1), entries.map { it.versionCode })
        assertEquals("2026-10-17", entries[0].date)
        assertEquals(listOf("Signs in.", "Downloads updates."), entries[0].notes)
    }

    @Test
    fun emptyFileFails() {
        assertTrue(failure("# Changelog\n").contains("no \"## \" entry"))
    }

    @Test
    fun malformedHeadingNamesTheLine() {
        val message = failure("# Changelog\n\n## 0.2 (2), 2026-10-17\n- x\n")
        assertTrue(message, message.startsWith("CHANGELOG.md:3:"))
    }

    @Test
    fun badDateFails() {
        assertTrue(failure("## 0.2.0 (2), 17-10-2026\n- x\n").startsWith("CHANGELOG.md:1:"))
    }

    @Test
    fun entryWithoutBulletFails() {
        val message = failure("## 0.2.0 (2), 2026-10-17\n\n## 0.1.0 (1), 2026-10-16\n- x\n")
        assertTrue(message, message.startsWith("CHANGELOG.md:1:") && message.contains("no \"- \" line"))
    }

    @Test
    fun bulletBeforeHeadingFails() {
        assertTrue(failure("- x\n## 0.1.0 (1), 2026-10-16\n- y\n").startsWith("CHANGELOG.md:1:"))
    }

    @Test
    fun versionCodeMustRise() {
        val message = failure("## 0.2.0 (1), 2026-10-17\n- x\n\n## 0.1.0 (1), 2026-10-16\n- y\n")
        assertTrue(message, message.contains("version code 1 of 0.2.0 is not higher than 1"))
    }

    @Test
    fun versionNameMustRise() {
        val message = failure("## 0.1.0 (2), 2026-10-17\n- x\n\n## 0.1.0 (1), 2026-10-16\n- y\n")
        assertTrue(message, message.contains("version 0.1.0 is not higher than 0.1.0"))
    }

    @Test
    fun versionsCompareNumerically() {
        assertTrue(Changelog.compareVersions("0.10.0", "0.9.0") > 0)
        assertEquals(0, Changelog.compareVersions("1.0", "1.0.0"))
        assertTrue(Changelog.compareVersions("0.1.9", "0.2.0") < 0)
    }
}

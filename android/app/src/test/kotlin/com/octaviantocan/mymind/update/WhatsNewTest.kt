package com.octaviantocan.mymind.update

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class WhatsNewTest {
    private lateinit var prefs: UpdatePrefs

    private val text = """
        # Changelog

        ## 0.2.1 (3), 2026-10-18
        - Newer line.

        ## 0.2.0 (2), 2026-10-17
        - Older line.
    """.trimIndent()

    @Before
    fun setUp() {
        prefs = UpdatePrefs(ApplicationProvider.getApplicationContext<Context>().getSharedPreferences("whatsnew-test", Context.MODE_PRIVATE).also { it.edit().clear().commit() })
    }

    @Test
    fun freshInstallShowsNothing() {
        assertNull(WhatsNew(prefs, 2, text).takePending())
    }

    @Test
    fun updateShowsTheRunningVersionOnce() {
        prefs.shownChangelogCode = 2
        val first = WhatsNew(prefs, 3, text).takePending()
        assertEquals("0.2.1", first?.versionName)
        assertEquals(listOf("Newer line."), first?.notes)
        assertNull(WhatsNew(prefs, 3, text).takePending())
    }

    @Test
    fun unreadableChangelogShowsNothing() {
        prefs.shownChangelogCode = 1
        assertNull(WhatsNew(prefs, 3, "garbage").takePending())
    }
}

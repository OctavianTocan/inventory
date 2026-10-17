package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.PageConfig
import com.octaviantocan.mymind.data.pageConfigOf
import org.junit.Assert.assertEquals
import org.junit.Test

class PageConfigTest {
    @Test
    fun readsSchemeFeaturesAndGuestFromTheConfigScript() {
        val html = """
            <html><head><script id="config" type="application/json">
            {"colorScheme":"dark","features":["reader","sameVibe"],"isGuest":false,"other":{"x":1}}
            </script></head><body></body></html>
        """.trimIndent()
        assertEquals(PageConfig(colorScheme = "dark", features = setOf("reader", "sameVibe")), pageConfigOf(html))
    }

    @Test
    fun aPageWithoutTheScriptGivesAnEmptyConfig() {
        assertEquals(PageConfig(), pageConfigOf("<html><body>signed out</body></html>"))
    }
}

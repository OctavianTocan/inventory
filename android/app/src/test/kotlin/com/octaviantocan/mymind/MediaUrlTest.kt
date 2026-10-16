package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.resolveMediaUrl
import org.junit.Assert.assertEquals
import org.junit.Test

class MediaUrlTest {
    @Test
    fun joinsServerAndPathWithOneSlash() {
        assertEquals("https://h.test/media/a;1x1.png", resolveMediaUrl("https://h.test", "/media/a;1x1.png"))
        assertEquals("https://h.test/media/a", resolveMediaUrl("https://h.test/", "media/a"))
    }

    @Test
    fun absoluteUrlIsUnchanged() {
        assertEquals("https://cdn.test/x.png", resolveMediaUrl("https://h.test", "https://cdn.test/x.png"))
    }
}

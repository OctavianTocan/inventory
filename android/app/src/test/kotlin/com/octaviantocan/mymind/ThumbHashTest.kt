package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.thumbHashPicture
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.abs

class ThumbHashTest {
    // Hashes and pixels (AARRGGBB at the top left, the middle and the bottom right) from the reference Rust crate
    // `thumbhash` 0.1.0 for a 100 by 60 opaque gradient and a 40 by 100 gradient that fades out.
    private fun assertDecodes(hash: String, width: Int, height: Int, corners: List<Long>) {
        val picture = thumbHashPicture(hash)!!
        assertEquals(width, picture.width)
        assertEquals(height, picture.height)
        val at = listOf(0 to 0, width / 2 to height / 2, width - 1 to height - 1)
        at.zip(corners).forEach { (point, expected) ->
            val actual = picture.argb[point.second * width + point.first]
            for (shift in listOf(24, 16, 8, 0)) {
                val difference = abs(((actual shr shift) and 255) - ((expected shr shift).toInt() and 255))
                assertTrue("pixel $point: ${Integer.toHexString(actual)} against ${expected.toString(16)}", difference <= 2)
            }
        }
    }

    @Test
    fun anOpaqueHashDecodesAsTheReferenceDoes() =
        assertDecodes("3wcKNJqAh4eAh4d4iHeAcAj4eA==", 32, 18, listOf(0xff1e1190, 0xff7d7e81, 0xfffffa88))

    @Test
    fun aHashWithTransparencyDecodesAsTheReferenceDoes() =
        assertDecodes("o9iFGg4nkLG4eIiQsQvpWXWHcHh3d3c=", 13, 32, listOf(0x00779086, 0x7e7d8b83, 0xbfe1e875))

    @Test
    fun aBlankOrBrokenHashHasNoPicture() {
        assertNull(thumbHashPicture(""))
        assertNull(thumbHashPicture("not base64!"))
        assertNull(thumbHashPicture("AAAA"))
    }
}

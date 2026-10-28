package com.octaviantocan.mymind.data

import java.util.Base64
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.roundToInt

/**
 * A picture a ThumbHash stands for, at most 32 pixels on its long side.
 *
 * @property width Width in pixels.
 * @property height Height in pixels.
 * @property argb The pixels row by row, as colour ints with straight (not premultiplied) alpha.
 */
class ThumbPicture(val width: Int, val height: Int, val argb: IntArray)

/**
 * Decodes the ThumbHash the server keeps for a card's gallery picture (`thumbHash` on its cover or blob) into the
 * blurred picture a tile shows while the real one loads. A port of `thumbHashToRGBA` from Evan Wallace's ThumbHash
 * (https://github.com/evanw/thumbhash, MIT).
 *
 * @param hash The hash in standard base64.
 * @return The picture, or null for a blank or malformed hash.
 */
fun thumbHashPicture(hash: String): ThumbPicture? {
    if (hash.isBlank()) return null
    val bytes = runCatching { Base64.getDecoder().decode(hash) }.getOrNull() ?: return null
    return runCatching { decode(bytes) }.getOrNull()
}

private fun decode(hash: ByteArray): ThumbPicture? {
    if (hash.size < 5) return null
    fun byte(at: Int) = hash[at].toInt() and 255
    val header24 = byte(0) or (byte(1) shl 8) or (byte(2) shl 16)
    val header16 = byte(3) or (byte(4) shl 8)
    val lDc = (header24 and 63) / 63f
    val pDc = ((header24 shr 6) and 63) / 31.5f - 1f
    val qDc = ((header24 shr 12) and 63) / 31.5f - 1f
    val lScale = ((header24 shr 18) and 31) / 31f
    val hasAlpha = (header24 shr 23) != 0
    val pScale = ((header16 shr 3) and 63) / 63f
    val qScale = ((header16 shr 9) and 63) / 63f
    val isLandscape = (header16 shr 15) != 0
    val lMax = if (hasAlpha) 5 else 7
    val lx = max(3, if (isLandscape) lMax else header16 and 7)
    val ly = max(3, if (isLandscape) header16 and 7 else lMax)
    val aDc = if (hasAlpha) (byte(5) and 15) / 15f else 1f
    val aScale = if (hasAlpha) (byte(5) shr 4) / 15f else 1f
    val start = if (hasAlpha) 6 else 5
    var at = 0
    fun channel(nx: Int, ny: Int, scale: Float): FloatArray {
        val ac = ArrayList<Float>()
        for (cy in 0 until ny) {
            var cx = if (cy > 0) 0 else 1
            while (cx * ny < nx * (ny - cy)) {
                val bits = (byte(start + (at shr 1)) shr ((at and 1) shl 2)) and 15
                ac.add((bits / 7.5f - 1f) * scale)
                at++
                cx++
            }
        }
        return ac.toFloatArray()
    }
    val lAc = channel(lx, ly, lScale)
    val pAc = channel(3, 3, pScale * 1.25f)
    val qAc = channel(3, 3, qScale * 1.25f)
    val aAc = if (hasAlpha) channel(5, 5, aScale) else FloatArray(0)

    val ratio = if (isLandscape) lMax.toFloat() / (header16 and 7) else (header16 and 7).toFloat() / lMax
    val w = if (ratio > 1f) 32 else (32 * ratio).roundToInt().coerceAtLeast(1)
    val h = if (ratio > 1f) (32 / ratio).roundToInt().coerceAtLeast(1) else 32
    val argb = IntArray(w * h)
    val fx = FloatArray(7)
    val fy = FloatArray(7)
    for (y in 0 until h) {
        for (x in 0 until w) {
            var l = lDc
            var p = pDc
            var q = qDc
            var a = aDc
            for (cx in 0 until max(lx, if (hasAlpha) 5 else 3)) fx[cx] = cos(PI / w * (x + 0.5) * cx).toFloat()
            for (cy in 0 until max(ly, if (hasAlpha) 5 else 3)) fy[cy] = cos(PI / h * (y + 0.5) * cy).toFloat()
            var j = 0
            for (cy in 0 until ly) {
                var cx = if (cy > 0) 0 else 1
                val fy2 = fy[cy] * 2
                while (cx * ly < lx * (ly - cy)) {
                    l += lAc[j++] * fx[cx] * fy2
                    cx++
                }
            }
            j = 0
            for (cy in 0 until 3) {
                var cx = if (cy > 0) 0 else 1
                val fy2 = fy[cy] * 2
                while (cx < 3 - cy) {
                    val f = fx[cx] * fy2
                    p += pAc[j] * f
                    q += qAc[j] * f
                    j++
                    cx++
                }
            }
            if (hasAlpha) {
                j = 0
                for (cy in 0 until 5) {
                    var cx = if (cy > 0) 0 else 1
                    val fy2 = fy[cy] * 2
                    while (cx < 5 - cy) {
                        a += aAc[j++] * fx[cx] * fy2
                        cx++
                    }
                }
            }
            val b = l - 2f / 3f * p
            val r = (3f * l - b + q) / 2f
            val g = r - q
            argb[y * w + x] = (unit(a) shl 24) or (unit(r) shl 16) or (unit(g) shl 8) or unit(b)
        }
    }
    return ThumbPicture(w, h, argb)
}

private fun unit(value: Float): Int = (value.coerceIn(0f, 1f) * 255).toInt()

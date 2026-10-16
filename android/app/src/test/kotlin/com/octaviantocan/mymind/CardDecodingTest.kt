package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.AppJson
import com.octaviantocan.mymind.data.CardPage
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CardDecodingTest {
    private val fixture = """
    {
      "cards": [
        {
          "id": "card-full-0001", "type": "XPost", "flags": 4,
          "created": "2030-01-02T03:04:05Z", "modified": "2030-01-02T03:04:06.123456Z", "bumped": "2030-01-02T03:04:07Z",
          "deleted": "2030-02-01T00:00:00Z",
          "title": "Sample title", "description": "Sample description",
          "url": "https://example.test/post/1", "domain": "example.test", "siteName": "Example",
          "author": "Sample Author", "note": "Sample note",
          "tags": [{"id": "tag-1", "name": "alpha", "flags": 2}],
          "spaces": ["space-1", "space-2"],
          "pinPosition": -1790003960,
          "cover": {"src": "/media/a;600x414.mp4", "width": 600, "height": 414, "type": "video", "dominantColor": "#010203"},
          "blob": {"path": "blob/path", "type": "image/jpeg", "width": 820, "height": 1024, "dominantColor": "#fafafa"},
          "prose": {"type": "doc", "content": []},
          "post": {"bodyHtml": "<p>hi</p>", "author": "by someone", "hasVideo": true,
                   "gallery": [{"src": "/media/g1", "width": 10, "height": 20}]},
          "app": {"distributor": "app-store", "srcset": "/media/s 2x", "width": 250, "height": 250},
          "product": {"price": "${'$'}9", "brand": "Brand"},
          "reddit": {"title": "Thread", "subreddit": "sample"},
          "state": "ready",
          "somethingNew": {"nested": true}
        },
        {"id": "card-min-0002", "type": "HologramCard", "flags": 0,
         "created": "2030-01-01T00:00:00Z", "modified": "2030-01-01T00:00:00Z", "bumped": "2030-01-01T00:00:00Z"}
      ],
      "next": "cursor-2"
    }
    """

    @Test
    fun decodesEveryContractField() {
        val page = AppJson.decodeFromString<CardPage>(fixture)
        val card = page.cards[0]
        assertEquals("cursor-2", page.next)
        assertEquals("XPost", card.type)
        assertEquals(4, card.flags)
        assertEquals("2030-02-01T00:00:00Z", card.deleted)
        assertEquals("example.test", card.domain)
        assertEquals("Example", card.siteName)
        assertEquals("alpha", card.tags?.single()?.name)
        assertEquals(2, card.tags?.single()?.flags)
        assertEquals(listOf("space-1", "space-2"), card.spaces)
        assertEquals(-1790003960L, card.pinPosition)
        assertEquals("video", card.cover?.type)
        assertEquals("#010203", card.cover?.dominantColor)
        assertEquals(1024, card.blob?.height)
        assertEquals("image/jpeg", card.blob?.type)
        assertEquals("doc", card.prose.toString().substringAfter("type\":\"").substringBefore('"'))
        assertEquals(true, card.post?.hasVideo)
        assertEquals(20, card.post?.gallery?.single()?.height)
        assertEquals("app-store", card.app?.distributor)
        assertEquals("Brand", card.product?.brand)
        assertEquals("sample", card.reddit?.subreddit)
        assertEquals("ready", card.state)
    }

    @Test
    fun unknownTypeAndMissingOptionalFieldsDecode() {
        val card = AppJson.decodeFromString<CardPage>(fixture).cards[1]
        assertEquals("HologramCard", card.type)
        assertNull(card.title)
        assertNull(card.cover)
        assertTrue(card.tags == null)
    }
}

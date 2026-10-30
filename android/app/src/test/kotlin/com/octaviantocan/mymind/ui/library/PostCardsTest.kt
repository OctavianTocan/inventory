package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.GalleryImage
import com.octaviantocan.mymind.data.Post
import com.octaviantocan.mymind.data.PostArticle
import com.octaviantocan.mymind.data.RedditComment
import com.octaviantocan.mymind.data.RedditInfo
import com.octaviantocan.mymind.ui.card.postDay
import com.octaviantocan.mymind.ui.card.postTimeLine
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset

class PostCardsTest {
    private val posted = Instant.parse("2026-10-14T08:14:00Z")
    private val xId = (posted.toEpochMilli() - 1_288_834_974_657L) shl 22

    private fun xPost(words: String, url: String = "https://x.com/theo_maps/status/$xId") = testCard("x", type = "XPost", url = url)
        .copy(post = Post(bodyHtml = "<p>$words</p>", author = "by Theo Park"), created = "2026-10-15T09:00:00Z")

    @Test
    fun anXPostNamesItsAuthorOverTheHandleFromItsLink() {
        assertEquals(PostByline("Theo Park", "@theo_maps"), postByline(xPost("Trail maps")))
    }

    @Test
    fun aNameThatCarriesItsHandleIsSplit() {
        val card = testCard("x", type = "XPost").copy(post = Post(author = "Ana Lima (@ana)"))
        assertEquals(PostByline("Ana Lima", "@ana"), postByline(card))
    }

    @Test
    fun aRedditPostNamesItsSubredditOverItsAuthor() {
        val card = testCard("r", type = "RedditPost").copy(author = "fernhollow", reddit = RedditInfo("The longest walk?", "hiking"))
        assertEquals(PostByline("r/hiking", "u/fernhollow"), postByline(card))
    }

    @Test
    fun otherNetworksReadTheHandleFromTheirLinks() {
        val threads = testCard("t", type = "ThreadsPost", url = "https://www.threads.net/@maya.walks/post/C1").copy(author = "Maya")
        val bluesky = testCard("b", type = "BlueskyPost", url = "https://bsky.app/profile/theo.bsky.social/post/3k").copy(author = "Theo")
        val mastodon = testCard("m", type = "MastodonPost", url = "https://mastodon.social/@sam/1131").copy(author = "Sam")
        assertEquals("@maya.walks", postByline(threads)?.handle)
        assertEquals("@theo.bsky.social", postByline(bluesky)?.handle)
        assertEquals("@sam@mastodon.social", postByline(mastodon)?.handle)
    }

    @Test
    fun aBlueskyLinkThatNamesItsAuthorByIdShowsNoHandle() {
        val card = testCard("b", type = "BlueskyPost", url = "https://bsky.app/profile/did:plc:fern/post/3k").copy(post = Post(author = "by Fern Hollow"))
        assertEquals(PostByline("Fern Hollow", null), postByline(card))
    }

    @Test
    fun anXPostCarriesTheArticleItLinksTo() {
        val article = PostArticle("The quiet economics of walking cities", "A city that is good to walk in.", "https://x.com/i/article/1844")
        val card = testCard("xa", type = "XPost").copy(post = Post(author = "by Noor"), description = article.preview, article = article)
        assertEquals(article, postArticle(card))
        assertEquals("The article's opening lines are not the post's words", "", postWords(card))
        assertTrue("An article stands two squares high", postStandsTall(card))
        assertNull(postArticle(card.copy(article = article.copy(title = " "))))
        assertNull("Only X posts carry articles", postArticle(card.copy(type = "WebPage")))
    }

    @Test
    fun aRedditPostKeepsItsTopCommentsWithTheirAuthors() {
        val comments = listOf(RedditComment("moss_and_map", "Forty-one km along the coast."), RedditComment("", "Who?"), RedditComment("ridgeline", " "))
        val card = testCard("r", type = "RedditPost").copy(reddit = RedditInfo("The longest walk?", "hiking", comments))
        assertEquals(listOf(comments[0]), redditComments(card))
        assertEquals(emptyList<RedditComment>(), redditComments(card.copy(type = "XPost")))
    }

    @Test
    fun anXPostSaysWhenItWasPostedFromItsId() {
        assertEquals(posted, postedAt(xPost("Trail maps")))
        assertNull("Ids before November 2010 carry no time", postedAt(xPost("Old", "https://twitter.com/jack/status/20")))
        val today = LocalDate.parse("2026-10-29")
        assertEquals("14 Oct", postDay(xPost("Trail maps"), today, ZoneOffset.UTC))
        assertEquals("8:14 am · 14 Oct 2026 · Saved 15 Oct", postTimeLine(xPost("Trail maps"), today, ZoneOffset.UTC))
    }

    @Test
    fun aPostStandsTwoSquaresHighWhenItsWordsOrPictureNeedTheRoom() {
        val short = xPost("The best map is the one you draw after the walk.")
        val long = xPost("Took the long way home and found a street I’ve walked past for six years without once turning into it.")
        val pictured = short.copy(post = short.post?.copy(gallery = listOf(GalleryImage("https://example.test/map.jpg", 1200, 900))))
        assertFalse(postStandsTall(short))
        assertTrue(postStandsTall(long))
        assertTrue(postStandsTall(pictured))
        assertEquals(MosaicSpan(1, 2), mosaicSpan(long))
        assertEquals(MosaicSpan(1, 1), mosaicSpan(short))
        assertFalse("Only posts stand tall for their words", postStandsTall(testCard("n", type = "Note", note = long.post?.bodyHtml)))
    }

    @Test
    fun eachNetworkHasItsOwnColour() {
        assertEquals(androidx.compose.ui.graphics.Color.Black, postBrand("XPost").paper)
        assertEquals(postBrand("RedditPost").paper, postColor("RedditPost"))
        assertTrue(listOf("XPost", "ThreadsPost", "RedditPost", "BlueskyPost", "MastodonPost", "SubstackNote").all { postBrand(it).icon != null })
    }
}

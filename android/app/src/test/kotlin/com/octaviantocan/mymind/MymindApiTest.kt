package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.UploadAuthorization
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

class MymindApiTest {
    private val server = MockWebServer()
    private lateinit var api: MymindApi

    @Before
    fun setUp() {
        server.start()
        api = MymindApi(OkHttpClient(), server.url("/").toString())
    }

    @After
    fun tearDown() = server.shutdown()

    private fun answer(body: String, code: Int = 200) = server.enqueue(MockResponse().setResponseCode(code).setBody(body))

    @Test
    fun pageConfigAsksForTheEverythingDocument() = runBlocking {
        answer("""<script id="config" type="application/json">{"colorScheme":"light","features":["reader"]}</script>""")
        val config = (api.pageConfig() as ApiResult.Ok).value
        assertEquals("light", config.colorScheme)
        assertEquals(setOf("reader"), config.features)
        val request = server.takeRequest()
        assertEquals("/everything", request.path)
        assertEquals("text/html", request.getHeader("Accept"))
        assertEquals("document", request.getHeader("Sec-Fetch-Dest"))
    }

    @Test
    fun suggestionsLeaveOutAnEmptyQuery() = runBlocking {
        answer("""[{"type":"type","text":"Notes","queryText":"type:note","count":3}]""")
        answer("[]")
        assertEquals(ApiResult.Ok(listOf(Suggestion("type", "Notes", "type:note"))), api.searchSuggestions(""))
        api.searchSuggestions("road trip")
        assertEquals("/search/suggestions", server.takeRequest().path)
        assertEquals("/search/suggestions?q=road%20trip", server.takeRequest().path)
    }

    @Test
    fun tagReadsUseTheWebRoutes() = runBlocking {
        answer("""{"tags":[{"name":"recipes","count":2}]}""")
        answer("""[{"name":"travel"}]""")
        answer("""[{"name":"rec"}]""")
        assertEquals("recipes", (api.tags() as ApiResult.Ok).value.tags.single().name)
        assertEquals("travel", (api.recentTags() as ApiResult.Ok).value.single().name)
        assertEquals("rec", (api.tagSuggestions("re") as ApiResult.Ok).value.single().name)
        assertEquals("/app/v1/tags", server.takeRequest().path)
        assertEquals("/tags/recent?take=5", server.takeRequest().path)
        assertEquals("/tags/suggest?q=re", server.takeRequest().path)
    }

    @Test
    fun tagWritesPostTheNameAndDeleteTheAnnotation() = runBlocking {
        answer("""{"id":"t1","name":"recipes","flags":1}""")
        answer("", 204)
        assertEquals(ApiResult.Ok(Tag("t1", "recipes", 1)), api.addTag("c1", "recipes"))
        assertEquals(ApiResult.Ok(Unit), api.removeTag("t1"))
        val post = server.takeRequest()
        assertEquals("POST /objects/c1/tags", "${post.method} ${post.path}")
        assertEquals("""{"name":"recipes"}""", post.body.readUtf8())
        val delete = server.takeRequest()
        assertEquals("DELETE /annotations/t1", "${delete.method} ${delete.path}")
    }

    @Test
    fun spaceWritesUseTheSpaceRoutes() = runBlocking {
        answer("{}")
        answer("""{"id":"s9","name":"New","color":"#f96"}""")
        answer("", 204)
        assertEquals(ApiResult.Ok(Unit), api.addToSpace("s1", "c1"))
        assertEquals(ApiResult.Ok("s9"), api.createSpace("New", "#f96"))
        assertEquals(ApiResult.Ok(Unit), api.deleteSpace("s9"))
        val put = server.takeRequest()
        assertEquals("PUT /spaces/s1/objects/c1", "${put.method} ${put.path}")
        val create = server.takeRequest()
        assertEquals("POST /spaces", "${create.method} ${create.path}")
        assertEquals("""{"name":"New","color":"#f96"}""", create.body.readUtf8())
        val delete = server.takeRequest()
        assertEquals("DELETE /spaces/s9", "${delete.method} ${delete.path}")
    }

    @Test
    fun mediaStreamsTheFileAndReportsAMissingOne() = runBlocking {
        answer("bytes")
        answer("missing", 404)
        var read = ""
        assertEquals(ApiResult.Ok(Unit), api.media("/u/a.png") { read = it.readBytes().decodeToString() })
        assertEquals("bytes", read)
        assertEquals(ApiResult.Failed(ApiFailure.Server(404)), api.media("u/b.png") {})
        assertEquals("/media/u/a.png", server.takeRequest().path)
        assertEquals("/media/u/b.png", server.takeRequest().path)
    }

    @Test
    fun topOfMindReadsThePinnedRoute() = runBlocking {
        answer("""{"cards":[{"id":"p1","type":"Note","flags":512,"created":"x","modified":"x","bumped":"x"}],"next":null}""")
        val page = (api.topOfMind(50, null) as ApiResult.Ok).value
        assertEquals(listOf("p1"), page.cards.map { it.id })
        assertEquals("/app/v1/top-of-mind?limit=50", server.takeRequest().path)
    }

    @Test
    fun serendipityIdsComeFromTheCardsOfThePage() = runBlocking {
        answer(
            """<main data-id="page"><mymind-card class="a" data-id="c1"></mymind-card>""" +
                """<mymind-card data-id='c2'><div data-id="inner"></div></mymind-card><mymind-card data-id="c1"></mymind-card></main>""",
        )
        assertEquals(ApiResult.Ok(listOf("c1", "c2")), api.serendipityIds())
        val request = server.takeRequest()
        assertEquals("GET /serendipity", "${request.method} ${request.path}")
        assertEquals("text/html", request.getHeader("Accept"))
        assertEquals("1", request.getHeader("x-partial"))
        assertEquals(null, request.getHeader("Sec-Fetch-Dest"))
    }

    @Test
    fun uploadsAreAuthorizedSentInPiecesAndBecomeMediaCards() = runBlocking {
        answer("""[{"url":"/uploads/u1","token":"up-1"},{"url":"/uploads/u2","token":"up-2"}]""")
        answer("""{"id":"u1","blob":{"id":"u1"},"received":4,"done":false}""")
        answer("""{"id":"u1","blob":{"id":"u1"},"received":6,"done":true}""")
        answer("""{"id":"c9"}""", 201)
        answer("""{"id":"c10"}""", 201)
        val authorizations = (api.authorizeUploads(2) as ApiResult.Ok).value
        assertEquals("up-2", authorizations[1].token)
        val source = "abcdef".byteInputStream()
        val first = (api.uploadChunk(authorizations[0], null, "a photo é.jpg", "image/jpeg", 6, 0, 4, source) as ApiResult.Ok).value
        val last = (api.uploadChunk(authorizations[0], first.id, "a photo é.jpg", "image/jpeg", 6, 4, 2, source) as ApiResult.Ok).value
        assertEquals(true, last.done)
        assertEquals(ApiResult.Ok("c9"), api.createMedia(last.blob!!.id, "s1"))
        assertEquals(ApiResult.Ok("c10"), api.createMedia("u2", null))

        val authorize = server.takeRequest()
        assertEquals("POST /uploads/authorize?count=2", "${authorize.method} ${authorize.path}")
        val one = server.takeRequest()
        assertEquals("PUT /uploads/u1", "${one.method} ${one.path}")
        assertEquals("image/jpeg", one.getHeader("Content-Type"))
        assertEquals("bytes 0-3/6", one.getHeader("Content-Range"))
        assertEquals("a%20photo%20%C3%A9.jpg", one.getHeader("x-file-name"))
        assertEquals("up-1", one.getHeader("x-upload-token"))
        assertEquals(null, one.getHeader("x-upload-id"))
        assertEquals("abcd", one.body.readUtf8())
        val two = server.takeRequest()
        assertEquals("bytes 4-5/6", two.getHeader("Content-Range"))
        assertEquals("u1", two.getHeader("x-upload-id"))
        assertEquals("ef", two.body.readUtf8())
        val media = server.takeRequest()
        assertEquals("POST /objects", "${media.method} ${media.path}")
        assertEquals("""{"type":"Media","uploadId":"u1","spaceId":"s1"}""", media.body.readUtf8())
        assertEquals("""{"type":"Media","uploadId":"u2"}""", server.takeRequest().body.readUtf8())
    }

    @Test
    fun contentCreationTreatsCreatedAndBumpedAlike() = runBlocking {
        answer("""{"id":"c1"}""", 201)
        answer("""{"id":"c1"}""", 200)
        assertEquals(ApiResult.Ok("c1"), api.createFromContent("https://example.com", "clipboard", null))
        assertEquals(ApiResult.Ok("c1"), api.createFromContent("https://example.com", "clipboard", "s1"))
        val first = server.takeRequest()
        assertEquals("POST /objects", "${first.method} ${first.path}")
        assertEquals("""{"content":"https://example.com","source":"clipboard"}""", first.body.readUtf8())
        assertEquals("""{"content":"https://example.com","source":"clipboard","spaceId":"s1"}""", server.takeRequest().body.readUtf8())
    }

    @Test
    fun aNoteIsAProseDocumentWithOneParagraphPerLine() = runBlocking {
        answer("""{"id":"n1"}""")
        assertEquals(ApiResult.Ok(Unit), api.setNote("c1", "one\n\ntwo"))
        val request = server.takeRequest()
        assertEquals("PUT /objects/c1/note", "${request.method} ${request.path}")
        assertEquals(
            """{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"one"}]},{"type":"paragraph"},{"type":"paragraph","content":[{"type":"text","text":"two"}]}]}""",
            request.body.readUtf8(),
        )
    }

    @Test
    fun aFileThatEndsEarlyFailsItsPiece() = runBlocking {
        answer("{}")
        val authorization = UploadAuthorization("/uploads/u1", "up-1")
        assertEquals(
            ApiResult.Failed(ApiFailure.Network),
            api.uploadChunk(authorization, null, "a.png", "image/png", 10, 0, 10, "abc".byteInputStream()),
        )
    }
}

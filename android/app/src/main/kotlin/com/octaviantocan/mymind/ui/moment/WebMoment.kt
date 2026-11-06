package com.octaviantocan.mymind.ui.moment

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Bitmap
import android.graphics.Color as AndroidColor
import android.os.Handler
import android.os.Looper
import android.util.Base64
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.platform.LocalContext
import androidx.webkit.WebViewAssetLoader
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Space
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.floatOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import java.io.ByteArrayOutputStream
import java.text.NumberFormat

/** How far the prototype's own share moment has got in loading. */
enum class WebStatus {
    /** The page is loading or compiling its shaders. */
    Loading,

    /** It can start at once. */
    Ready,

    /** It is playing. */
    Playing,

    /** It cannot play on this phone (no WebView or no WebGL), so the app's own drawing of the moment plays instead. */
    Failed,
}

/**
 * The shooting star as the prototype's own engine plays it (share-moment-shooting-star), in a see-through WebView over
 * the app that shared: the same sky, light, particles, 3D card, stars, points and backpack as moment 03 of the
 * Inventory share moment prototype, fed the find's own title, site, number, picture and rarity. The app keeps the
 * parts that are its own: the sounds and vibrations (share-moment-sounds), the tag step on the card's back
 * (share-tag-step-icon-grid) and the blur of the page behind.
 *
 * The page, built from `android/moment-web`, talks through [post]; the app answers by running script in it.
 *
 * @param sound The sound output, or null for none.
 * @param haptics The vibration motor, or null for none.
 */
@Stable
class WebMoment internal constructor(private val sound: MomentSound?, private val haptics: MomentHaptics?) {
    /** How far the page has got. */
    var status by mutableStateOf(WebStatus.Loading)
        internal set

    /** Where the moment is, as [ShareMoment] reads it. */
    var phase by mutableStateOf(StarPhase.Playing)
        private set

    /** True once a tap skipped the moment, which skips the tag step too. */
    var skipped by mutableStateOf(false)
        private set

    /** Where the card's face stood, in pixels of the page, when it turned over for the tag step; null before. */
    var card by mutableStateOf<Rect?>(null)
        private set

    /** True while the tag step closes, before the card turns face up again on the page. */
    var closing by mutableStateOf(false)
        private set

    /** How much the sky should cover the page behind, and how long it takes to get there. */
    var sky by mutableStateOf(SkyTarget(0f, 0))
        private set

    private var kept: Rarity? = null

    /** The find's rarity once the library is counted; the star's colours follow it until it breaks off. */
    var rarity: Rarity? = null
        set(value) { field = value; if (value != null) run("InvMoment.rarity(${value.stars})") }

    /** The rarity the moment shows: the counted one, or Common once the star broke off without it. */
    val shownRarity: Rarity get() = kept ?: rarity ?: Rarity.Common

    internal var density = 1f
    internal var sink: ((String) -> Unit)? = null
    private var started = false
    private var pending: String? = null

    /** Starts the moment with what is known of the find, as soon as the page is ready. */
    internal fun start(find: JsonObject) {
        if (started) return
        started = true
        val call = "InvMoment.start(${buildJsonObject { put("find", find); rarity?.let { put("rarity", it.stars) } }})"
        if (status == WebStatus.Ready) begin(call) else pending = call
    }

    private fun begin(call: String) {
        status = WebStatus.Playing
        sound?.start()
        run(call)
    }

    /** Sends what more is known of the find: its title, number, tags or space. */
    internal fun update(find: JsonObject) { if (started) run("InvMoment.find($find)") }

    /** Sends the find's picture, already cropped to 3 by 4. */
    internal fun photo(bitmap: Bitmap) {
        val bytes = ByteArrayOutputStream().also { bitmap.compress(Bitmap.CompressFormat.JPEG, 86, it) }.toByteArray()
        run("InvMoment.photo('data:image/jpeg;base64,${Base64.encodeToString(bytes, Base64.NO_WRAP)}')")
    }

    /** Skips to the backpack, as a tap does while it plays. */
    fun skip() { if (phase == StarPhase.Playing) run("InvMoment.skip()") }

    /** Starts closing the tag step; the card turns back once the app's side has turned to its edge. */
    fun closeTags() { if (phase == StarPhase.Settled) closing = true }

    /**
     * The tag step has turned to its edge: the card comes round face up on the page with what was chosen, and flies
     * into the backpack.
     */
    internal fun tagsClosed(space: Space?, tags: List<String>) {
        if (!closing) return
        closing = false
        phase = StarPhase.Leaving
        run("InvMoment.tagsDone(${tagsJson(space, tags)})")
    }

    /** Fades it all away without the backpack, when nothing was saved after all. */
    fun abort() { run("InvMoment.abort()") }

    internal fun close() { sound?.stop() }

    /**
     * Handles one message from the page.
     *
     * @param name What happened: ready, fail, sky, rarity, sfx, tags, skip or done.
     * @param data Its details as JSON, or an empty string.
     */
    internal fun receive(name: String, data: String) {
        val o = runCatching { Json.parseToJsonElement(data).jsonObject }.getOrNull()
        when (name) {
            "ready" -> if (status == WebStatus.Loading) {
                status = WebStatus.Ready
                pending?.let { pending = null; begin(it) }
            }
            "fail" -> if (status == WebStatus.Loading || status == WebStatus.Ready) status = WebStatus.Failed
            "sky" -> sky = SkyTarget(o.num("a") ?: 0f, o.num("ms")?.toInt() ?: 0)
            "rarity" -> kept = Rarity.entries.firstOrNull { it.stars == o.int("r") } ?: Rarity.Common
            "sfx" -> cueOf(o.str("n"), o?.get("a")?.jsonPrimitive?.doubleOrNull, o.int("r"))?.let { sound?.play(it); haptics?.feel(it) }
            "tags" -> {
                val r = o?.get("rect")?.jsonObject
                val x = r.num("x"); val y = r.num("y"); val w = r.num("w"); val h = r.num("h")
                if (x != null && y != null && w != null && h != null) {
                    card = Rect(x - w / 2f, y - h / 2f, x + w / 2f, y + h / 2f).let { Rect(it.left * density, it.top * density, it.right * density, it.bottom * density) }
                    phase = StarPhase.Settled
                }
            }
            "skip" -> { skipped = true; phase = StarPhase.Leaving }
            "done" -> { phase = StarPhase.Done; sky = SkyTarget(0f, 0) }
        }
    }

    private fun run(script: String) { sink?.invoke(script) }

    /** The page's bridge: messages arrive on a WebView thread and are handled on the main one. */
    internal inner class Bridge {
        private val main = Handler(Looper.getMainLooper())

        @JavascriptInterface
        fun post(name: String, data: String) { main.post { receive(name, data) } }
    }
}

/** How much the sky covers the page behind ([amount], 0 to 1), reached in [ms]. */
data class SkyTarget(val amount: Float, val ms: Int)

/**
 * @param name A sound of the page's moment.
 * @param arg Its argument: a step, a length in seconds, a rarity or a pitch.
 * @param stars The rarity the page plays, as stars.
 * @return The beat the app plays and feels for it, or null for a sound the shooting star does not use.
 */
internal fun cueOf(name: String?, arg: Double?, stars: Int?): Cue? = when (name) {
    "shimmer" -> Cue.Shimmer
    "ting" -> Cue.Ting((arg ?: 0.0).toInt().coerceIn(0, 2))
    "meteor" -> Cue.Meteor((arg ?: 1.6).toFloat())
    "riser" -> Cue.Riser((arg ?: 1.4).toFloat())
    "suck" -> Cue.Suck
    "swoop" -> Cue.Swoop
    "thud" -> Cue.Thud
    "boom" -> Cue.Boom(arg?.toFloat()?.coerceIn(0f, 1f) ?: when (stars) { 5 -> 1f; 3 -> 0.62f; else -> 0.3f })
    "chime" -> Cue.Chime(Rarity.entries.firstOrNull { it.stars == (arg ?: 1.0).toInt() } ?: Rarity.Common)
    "tick" -> tickCue(arg ?: 0.0)

    "pop" -> Cue.Pop
    "whoosh" -> Cue.Whoosh
    else -> null
}

/**
 * @param hz A tick's pitch: the card's stars stamp in at 1300, 1500, 1700, 1900 and 2100, and the points count at 1800.
 * @return The star or count beat, or null for a tick the shooting star does not play.
 */
private fun tickCue(hz: Double): Cue? {
    if (kotlin.math.abs(hz - 1800.0) < 20.0) return Cue.Count
    val step = (hz - 1300.0) / 200.0
    val star = kotlin.math.round(step).toInt()
    return if (star in 0..4 && kotlin.math.abs(step - star) < 0.1) Cue.Star(star) else null
}

/**
 * @param find What the card shows.
 * @param before How many finds the library held before this share, or null.
 * @param saved How many finds this share added.
 * @param space The space ThyMind suggests, or null.
 * @return The find as the page reads it.
 */
internal fun findJson(find: MomentFind, before: Int?, saved: Int, space: Space?): JsonObject = buildJsonObject {
    put("title", find.title)
    put("kind", find.kind)
    put("glyph", glyphOf(find.icon))
    put("no", find.number?.let { "No. " + NumberFormat.getIntegerInstance().format(it) } ?: "")
    before?.let { put("count", it) }
    put("saved", saved)
    spaceJson(space)
    putJsonArray("tags") { find.tags.take(12).forEach { add(JsonPrimitive(it)) } }
}

private fun tagsJson(space: Space?, tags: List<String>) = buildJsonObject {
    spaceJson(space)
    putJsonArray("tags") { tags.take(12).forEach { add(JsonPrimitive(it)) } }
}

private fun kotlinx.serialization.json.JsonObjectBuilder.spaceJson(space: Space?) {
    if (space == null) { put("space", JsonPrimitive(null as String?)); return }
    putJsonObject("space") { put("label", space.name.orEmpty()); put("col", space.color ?: "#A7D3CC") }
}

private fun glyphOf(icon: Int) = when (icon) {
    R.drawable.ic_link -> "link"
    R.drawable.ic_quote -> "note"
    R.drawable.ic_image -> "image"
    R.drawable.ic_video -> "video"
    R.drawable.ic_speaker_high -> "audio"
    R.drawable.ic_pdf -> "pdf"
    else -> "file"
}

private fun JsonObject?.num(key: String): Float? = this?.get(key)?.jsonPrimitive?.floatOrNull
private fun JsonObject?.int(key: String): Int? = this?.get(key)?.jsonPrimitive?.intOrNull
private fun JsonObject?.str(key: String): String? = this?.get(key)?.jsonPrimitive?.contentOrNull

/**
 * @return A web moment that plays its sounds and feel on this phone, or null where the phone has no WebView to play it.
 */
@Composable
fun rememberWebMoment(): WebMoment? {
    val context = LocalContext.current.applicationContext
    val moment = remember { if (hasWebView(context)) WebMoment(MomentSound(context), MomentHaptics(context)) else null }
    DisposableEffect(moment) { onDispose { moment?.close() } }
    return moment
}

private fun hasWebView(context: Context) = runCatching { WebView.getCurrentWebViewPackage() != null }.getOrDefault(false) &&
    context.packageManager.hasSystemFeature("android.software.webview")

/**
 * Makes the see-through WebView the moment plays in. It loads only the moment's own page from the app's assets (and
 * the app's font); every other request and every navigation is refused, and the page has no file or content access.
 */
@SuppressLint("SetJavaScriptEnabled")
internal fun momentWebView(context: Context, moment: WebMoment): WebView {
    val assets = WebViewAssetLoader.Builder()
        .addPathHandler("/assets/", WebViewAssetLoader.AssetsPathHandler(context))
        .addPathHandler("/res/", WebViewAssetLoader.ResourcesPathHandler(context))
        .build()
    return WebView(context).apply {
        setBackgroundColor(AndroidColor.TRANSPARENT)
        isVerticalScrollBarEnabled = false
        isHorizontalScrollBarEnabled = false
        overScrollMode = WebView.OVER_SCROLL_NEVER
        settings.javaScriptEnabled = true
        settings.mediaPlaybackRequiresUserGesture = true
        settings.allowFileAccess = false
        settings.allowContentAccess = false
        settings.setSupportZoom(false)
        settings.textZoom = 100
        webViewClient = object : WebViewClient() {
            override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? =
                assets.shouldInterceptRequest(request.url) ?: WebResourceResponse("text/plain", "utf-8", 404, "Not found", emptyMap(), null)

            override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) = true
        }
        addJavascriptInterface(moment.Bridge(), "InvApp")
        moment.density = resources.displayMetrics.density
        moment.sink = { script -> post { evaluateJavascript(script, null) } }
        loadUrl(MOMENT_URL)
    }
}

/** The moment's page, served from the app's assets under the WebView's own app-assets host. */
internal const val MOMENT_URL = "https://appassets.androidplatform.net/assets/moment/index.html"

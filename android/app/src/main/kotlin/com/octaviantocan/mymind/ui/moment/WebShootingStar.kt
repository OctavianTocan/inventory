package com.octaviantocan.mymind.ui.moment

import android.content.Context
import android.graphics.Bitmap
import android.net.Uri
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.request.allowHardware
import coil3.toBitmap
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.resolveMediaUrl
import com.octaviantocan.mymind.ui.library.galleryPicture
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

/**
 * The shooting star as the prototype's own engine plays it, over the app that shared (share-moment-shooting-star):
 * the page in [WebMoment] draws the sky, the star, the 3D card, its stars and points and the backpack; this adds the
 * blur of the page behind, the find's picture, and the tag step on the card's back (share-tag-step-icon-grid). When
 * the card turns over, the page hands it over edge on and the tag step grows out of that edge to fill the screen at
 * the card's shape; when the tag step closes it shrinks back to the card and turns to its other edge, and the page
 * brings the card round face up and flies it into the backpack.
 *
 * @param web The page's moment.
 * @param find What the card shows.
 * @param before How many finds the library held before this one, for the backpack's count, or null if unknown.
 * @param saved How many finds this share added.
 * @param space The space the find goes into, as known now, or null.
 * @param tagOpen True while the tag step should show.
 * @param chosen The space and tags chosen in the tag step, read as it closes.
 * @param onOutside Called for a tap round the tag step.
 * @param onSky Called as the sky comes and goes, with how much of it covers the page, from 0 to 1.
 * @param tagStep The tag step, drawn at the size it is given.
 */
@Composable
internal fun WebShootingStar(
    web: WebMoment,
    find: MomentFind,
    before: Int?,
    saved: Int,
    space: Space?,
    tagOpen: Boolean,
    chosen: () -> Pair<Space?, List<String>>,
    onOutside: () -> Unit,
    onSky: (Float) -> Unit,
    modifier: Modifier = Modifier,
    tagStep: @Composable (Modifier) -> Unit,
) {
    val context = LocalContext.current
    val latestSky by rememberUpdatedState(onSky)
    BoxWithConstraints(modifier.fillMaxSize().semantics { contentDescription = "Saved. Tap to skip" }) {
        AndroidView(factory = { momentWebView(it, web) }, onRelease = { web.sink = null; it.destroy() }, modifier = Modifier.fillMaxSize())
        val page = findJson(find, before, saved, space)
        LaunchedEffect(Unit) {
            web.start(page)
            // A page that is not ready in time (an old phone loading its WebView for the first time) gives way to the
            // app's own drawing of the moment, so nothing is left waiting.
            delay(READY_MS)
            if (web.status == WebStatus.Loading) web.status = WebStatus.Failed
        }
        LaunchedEffect(page) { web.update(page) }
        val picture = pictureOf(find)
        LaunchedEffect(picture) { picture?.let { loadPicture(context, it) }?.let(web::photo) }

        val sky = remember { Animatable(0f) }
        LaunchedEffect(web.sky) { sky.animateTo(web.sky.amount, tween(web.sky.ms, easing = LinearEasing)) }
        LaunchedEffect(Unit) { snapshotFlow { sky.value }.collect { latestSky(it) } }

        val card = web.card ?: return@BoxWithConstraints
        if (tagOpen || web.closing) TagBack(web, card, tagOpen, chosen, onOutside, tagStep)
    }
}

/** The tag step on the card's back, turning and growing out of the card the page handed over edge on. */
@Composable
private fun TagBack(web: WebMoment, card: Rect, open: Boolean, chosen: () -> Pair<Space?, List<String>>, onOutside: () -> Unit, tagStep: @Composable (Modifier) -> Unit) {
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val density = LocalDensity.current
        val wPx = constraints.maxWidth.toFloat()
        val hPx = constraints.maxHeight.toFloat()
        val topPx = WindowInsets.statusBars.getTop(density).toFloat()
        val navPx = WindowInsets.navigationBars.getBottom(density).toFloat()
        val imePx = WindowInsets.ime.getBottom(density).toFloat()
        // M9's card is the find's card grown to fill the screen, kept to the card's own shape.
        val gap = with(density) { 18.dp.toPx() }
        val gridW = min(min(wPx - gap * 2f, (hPx - topPx - navPx - gap * 2f) * CARD_RATIO), with(density) { 440.dp.toPx() })
        val gridH = gridW / CARD_RATIO
        val gridX = (wPx - gridW) / 2f
        val gridY = topPx + (hPx - topPx - navPx - gridH) / 2f
        val grow = remember { Animatable(0f) }
        val latestChosen by rememberUpdatedState(chosen)
        LaunchedEffect(open, web.closing) {
            if (open && !web.closing) grow.animateTo(1f, tween(TURN_IN_MS, easing = TurnOut))
            else if (web.closing) {
                grow.animateTo(0f, tween(TURN_OUT_MS, easing = TurnIn))
                val (space, tags) = latestChosen()
                web.tagsClosed(space, tags)
            }
        }
        val latestOutside by rememberUpdatedState(onOutside)
        Box(Modifier.fillMaxSize().pointerInput(Unit) { detectTapGestures { if (open) latestOutside() } })
        val lifted = imePx > 0f && open
        val height = if (lifted) min(gridH, hPx - imePx - topPx - gap) else gridH
        tagStep(
            Modifier.offset { IntOffset(gridX.roundToInt(), (if (lifted) topPx + gap / 2f else gridY).roundToInt()) }
                .size(with(density) { gridW.toDp() }, with(density) { max(height, 1f).toDp() })
                .graphicsLayer {
                    if (lifted) return@graphicsLayer
                    val e = grow.value
                    val s = card.width / gridW + (1f - card.width / gridW) * e
                    transformOrigin = TransformOrigin.Center
                    scaleX = s; scaleY = s
                    translationX = (card.center.x - (gridX + gridW / 2f)) * (1f - e)
                    translationY = (card.center.y - (gridY + gridH / 2f)) * (1f - e)
                    // It comes in from the edge the page's card turned to, and leaves by the other one, so the turn
                    // carries on in one direction all the way round.
                    rotationY = TURN_SIGN * 90f * (1f - e) * if (web.closing) -1f else 1f
                    cameraDistance = 14f * density.density * 72f
                },
        )
    }
}

/** What the card's picture is: the saved card's own picture once the server has it, or the shared image. */
private fun pictureOf(find: MomentFind): Any? =
    find.card?.let { galleryPicture(it) }?.let { resolveMediaUrl(BuildConfig.SERVER_URL, it) } ?: find.shared?.let(Uri::parse)

/** Loads the picture and crops it to 3 by 4, the shape the page's card window expects, at most 600 by 800. */
private suspend fun loadPicture(context: Context, data: Any): Bitmap? = runCatching {
    val request = ImageRequest.Builder(context).data(data).size(900, 1200).allowHardware(false).build()
    val bitmap = (SingletonImageLoader.get(context).execute(request) as? SuccessResult)?.image?.toBitmap() ?: return null
    withContext(Dispatchers.Default) { cropThreeByFour(bitmap) }
}.getOrNull()

/**
 * @param bitmap A picture of any shape.
 * @return Its middle at 3 by 4, scaled down to at most 600 by 800.
 */
internal fun cropThreeByFour(bitmap: Bitmap): Bitmap {
    val w = bitmap.width; val h = bitmap.height
    val cw = min(w, h * 3 / 4); val ch = min(h, w * 4 / 3)
    val cut = Bitmap.createBitmap(bitmap, (w - cw) / 2, (h - ch) / 2, max(1, cw), max(1, ch))
    return if (cut.width <= 600) cut else Bitmap.createScaledBitmap(cut, 600, 800, true)
}

/** How long the page waits to be ready before the app's own drawing plays instead. */
private const val READY_MS = 2_500L

/** The tag step grows out of the card's edge in this long. */
private const val TURN_IN_MS = 380

/** It shrinks back and turns to the card's other edge in this long. */
private const val TURN_OUT_MS = 300

/** -1: a view turned a positive angle brings its left edge nearer, the opposite way to the page's card (WebMomentTest). */
internal const val TURN_SIGN = -1f

private val TurnOut = CubicBezierEasing(0.2f, 0.8f, 0.2f, 1f)
private val TurnIn = CubicBezierEasing(0.5f, 0f, 0.8f, 0.4f)

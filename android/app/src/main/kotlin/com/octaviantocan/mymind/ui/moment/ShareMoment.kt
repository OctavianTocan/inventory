package com.octaviantocan.mymind.ui.moment

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.share.ShareDetails
import com.octaviantocan.mymind.share.ShareItem
import com.octaviantocan.mymind.ui.share.SAVED_FADE_MS
import com.octaviantocan.mymind.ui.share.SavePhase
import com.octaviantocan.mymind.ui.share.ShareSheet
import com.octaviantocan.mymind.ui.share.ShareSheetState
import com.octaviantocan.mymind.ui.share.savedLines
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.net.URI

/**
 * What the library says about a find: how rare it is and how many finds it held before.
 *
 * @property rarity How rare the find is.
 * @property before How many cards the gallery showed before this share, or null when it is not known.
 */
data class FindCount(val rarity: Rarity, val before: Int?)

/**
 * What plays after something is shared, while it saves: the shooting star moment, then the tag grid on the card's
 * back, then the find flying into the backpack (share-moment-shooting-star, share-tag-step-icon-grid). The tag grid
 * waits [SAVED_FADE_MS] untouched and goes by itself (share-sheet-fades-by-itself). If nothing could be saved the
 * moment fades and the share sheet says why.
 *
 * The moment is the prototype's own, played by [web]; where the phone cannot play it, the app's own drawing of it
 * ([moment]) plays instead.
 *
 * @param state The share's state, filled in as saving goes and as ThyMind reads what was saved.
 * @param count Counts the library for the find; null when it cannot.
 * @param onDetails Called once with the spaces and tags chosen, when the tag step ends.
 * @param onSky Called each frame with how much of the sky covers the page, from 0 to 1.
 * @param onClosed Called once nothing is left on screen.
 */
@Composable
fun ShareMoment(
    state: ShareSheetState,
    count: suspend () -> FindCount?,
    onDetails: (ShareDetails) -> Unit,
    onSky: (Float) -> Unit,
    onClosed: () -> Unit,
    moment: StarMoment = rememberStarMoment(),
    web: WebMoment? = rememberWebMoment(),
) {
    val onWeb = web != null && web.status != WebStatus.Failed
    var before by remember { mutableStateOf<Int?>(null) }
    var tagOpen by remember { mutableStateOf(false) }
    var problem by remember { mutableStateOf(false) }
    val latestClosed by rememberUpdatedState(onClosed)
    val latestDetails by rememberUpdatedState(onDetails)
    val focus = LocalFocusManager.current
    LaunchedEffect(Unit) {
        val found = count()
        moment.rarity = found?.rarity ?: Rarity.Common
        web?.rarity = found?.rarity ?: Rarity.Common
        before = found?.before
    }
    LaunchedEffect(state.phase) {
        if (state.phase == SavePhase.Failed && !problem) { problem = true; tagOpen = false; moment.abort(); if (onWeb) web?.abort() }
    }
    val closeTags = {
        if (tagOpen) {
            tagOpen = false
            focus.clearFocus()
            latestDetails(state.details())
            if (onWeb) web?.closeTags() else moment.closeTags()
        }
    }
    val phase = if (onWeb && web != null) web.phase else moment.phase
    LaunchedEffect(phase) {
        when (phase) {
            // No space is chosen for the user; the suggested one only comes first in the row (Tavi, 6 Nov 2026).
            StarPhase.Settled -> if (!problem && !tagOpen) tagOpen = true
            StarPhase.Done -> if (!problem) latestClosed()
            else -> Unit
        }
    }
    val left = remember { Animatable(1f) }
    val counting = tagOpen && state.phase != SavePhase.Saving && !state.touched && !state.findingTags
    LaunchedEffect(counting) {
        if (!counting) return@LaunchedEffect
        delay(StarSim.TURN_MS.toLong())
        left.snapTo(1f)
        launch { left.animateTo(0f, tween(SAVED_FADE_MS.toInt(), easing = LinearEasing)) }
        // The wait is a delay, not the animation, so it lasts as long with the system's animations turned off.
        delay(SAVED_FADE_MS)
        closeTags()
    }
    BackHandler {
        when {
            phase == StarPhase.Playing -> if (onWeb) web?.skip() else moment.skip()
            tagOpen -> closeTags()
            else -> latestClosed()
        }
    }
    val find = momentFindOf(state, before?.plus(1))
    val shown = if (onWeb && web != null) web.shownRarity else moment.shownRarity
    val grid: @Composable (Modifier) -> Unit = { modifier ->
        TagGrid(state, shown.color, left = { if (counting) left.value else null },
            onTouch = { state.touched = true }, onDone = closeTags, modifier = modifier)
    }
    if (onWeb && web != null) {
        WebShootingStar(
            web, find, before, saved = state.saved.coerceAtLeast(1), space = chosenSpace(state), tagOpen = tagOpen,
            chosen = { chosenSpace(state) to (state.foundTags + state.tags).distinctBy { it.lowercase() } },
            onOutside = closeTags, onSky = onSky, tagStep = grid,
        )
    } else {
        ShootingStar(moment, find, before, saved = state.saved.coerceAtLeast(1), tagOpen = tagOpen, onOutside = closeTags, onSky = onSky, tagStep = grid)
    }
    if (problem) ShareSheet(state, onClose = onClosed)
}

/** The space the user chose for the find, or null; a suggestion alone never puts it in one. */
private fun chosenSpace(state: ShareSheetState): Space? =
    state.spaceIds.firstOrNull()?.let { id -> state.spaces.firstOrNull { it.id == id } }

/**
 * @param state The share's state.
 * @param number The find's number in the library, or null.
 * @return What the find's card shows, as the share stands now.
 */
fun momentFindOf(state: ShareSheetState, number: Int?): MomentFind {
    val item = state.items.firstOrNull()
    val card = state.cards.firstOrNull()?.takeIf { it.type != "Placeholder" }
    val mime = state.files.firstOrNull()?.mime
    val link = (item as? ShareItem.Text)?.text?.trim()?.takeIf { LINK.matches(it) }
    val kind = when {
        link != null -> card?.siteName?.takeIf { it.isNotBlank() } ?: card?.domain ?: runCatching { URI(link).host?.removePrefix("www.") }.getOrNull() ?: "Link"
        item is ShareItem.Text -> "Note"
        mime == null -> "File"
        mime.startsWith("image/") -> "Photo"
        mime.startsWith("video/") -> "Video"
        mime.startsWith("audio/") -> "Audio"
        mime == "application/pdf" -> "PDF"
        else -> "File"
    }
    val icon = when {
        link != null -> R.drawable.ic_link
        item is ShareItem.Text -> R.drawable.ic_quote
        mime?.startsWith("image/") == true -> R.drawable.ic_image
        mime?.startsWith("video/") == true -> R.drawable.ic_video
        mime?.startsWith("audio/") == true -> R.drawable.ic_speaker_high
        mime == "application/pdf" -> R.drawable.ic_pdf
        else -> R.drawable.ic_file
    }
    val shared = (item as? ShareItem.Stream)?.uri?.takeIf { mime == null || mime.startsWith("image/") }
    return MomentFind(
        title = savedLines(state).title, kind = kind, icon = icon, card = card, shared = shared,
        tags = (state.foundTags + state.tags).distinctBy { it.lowercase() }, number = number,
    )
}

private val LINK = Regex("""^https?://\S+$""")

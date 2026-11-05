package com.octaviantocan.mymind.ui.sky

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.AppJson
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.LibraryMemory
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.data.SkyStar
import com.octaviantocan.mymind.ui.library.GalleryTile
import com.octaviantocan.mymind.ui.library.TILE_CORNER
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.time.Instant
import java.time.LocalDate

/** How long the leaves take to blow off a find. */
internal const val BLOW_MS = 900

/** When the find opens while its leaves still fly, so the card rises through the last of them. */
internal const val OPEN_AFTER_MS = 520L

/**
 * Back today as the band shows it.
 *
 * @property sky The day's sky, or null before one is known for today.
 * @property cards Today's finds that the library still holds, in the server's order.
 */
data class BackTodayState(val sky: Sky? = null, val cards: List<Card> = emptyList()) {
    /** The finds opened from Back today or the sky; their leaves are gone. */
    val uncovered: Set<String> get() = sky?.stars?.mapTo(HashSet()) { it.id }.orEmpty()
}

/**
 * Today's old finds for the band under Pinned, from the server's sky, kept on the phone so the band shows at once and
 * offline the same day.
 *
 * @param scope Where reads run.
 * @param library The library: its sky, its cards and where visits go.
 * @param memory Where the last answer is kept, or null to keep none.
 * @param today The phone's date, `YYYY-MM-DD`.
 */
class BackTodayModel(
    private val scope: CoroutineScope,
    private val library: Library,
    private val memory: LibraryMemory?,
    private val today: () -> String = { LocalDate.now().toString() },
) {
    private val mutableState = MutableStateFlow(BackTodayState())
    private var loading: Job? = null

    /** The band's finds and which of them are uncovered. */
    val state: StateFlow<BackTodayState> = mutableState

    /** Shows the kept answer when it is today's, then asks the server; does nothing while a read runs. */
    fun load() {
        if (loading?.isActive == true) return
        loading = scope.launch {
            val day = today()
            if (mutableState.value.sky?.day != day) {
                val kept = memory?.sky()?.let { json -> runCatching { AppJson.decodeFromString(Sky.serializer(), json) }.getOrNull() }
                show(kept?.takeIf { it.day == day })
            }
            val answer = library.sky(day)
            if (answer is ApiResult.Ok) {
                // A visit still waiting to be sent stays uncovered until the server has it.
                val waiting = mutableState.value.sky?.takeIf { it.day == day }?.stars.orEmpty().filter { star -> answer.value.stars.none { it.id == star.id } }
                val sky = answer.value.copy(stars = answer.value.stars + waiting)
                show(sky)
                memory?.saveSky(AppJson.encodeToString(Sky.serializer(), sky))
            }
        }
    }

    /** Asks again once the date has changed since the band last showed, as when the app comes back after midnight. */
    fun loadIfNewDay() {
        if (mutableState.value.sky?.day != today()) load()
    }

    /** Reads the finds' cards again, as after the phone's copy changed: a find sent to the trash leaves the band. */
    fun reread() {
        val sky = mutableState.value.sky ?: return
        scope.launch { show(sky) }
    }

    /**
     * Uncovers a find and records the visit, which lights its star.
     *
     * @param card A find in the band.
     */
    fun uncover(card: Card) {
        val sky = mutableState.value.sky ?: return
        if (card.id in mutableState.value.uncovered) return
        val active = sky.active?.let { if (card.id in it.cards && card.id !in it.lit) it.copy(lit = it.lit + card.id) else it }
        val visited = sky.copy(stars = sky.stars + SkyStar(card.id, Instant.now().toString()), active = active)
        mutableState.update { it.copy(sky = visited) }
        scope.launch {
            library.visit(card.id)
            memory?.saveSky(AppJson.encodeToString(Sky.serializer(), visited))
        }
    }

    private suspend fun show(sky: Sky?) {
        if (sky == null) {
            mutableState.value = BackTodayState()
            return
        }
        val cards = sky.back.mapNotNull { id -> (library.card(id) as? ApiResult.Ok)?.value?.takeIf { it.deleted == null } }
        mutableState.value = BackTodayState(sky, cards)
    }
}

/**
 * Back today: up to three old finds under Pinned, each covered in fallen leaves until it is opened. A tap blows the
 * leaves away from the finger and opens the find as they fly; a find already opened today shows as it is.
 *
 * @param state The band's finds.
 * @param onUncover Called as a covered find's leaves start to blow away.
 * @param onOpenCard Opens a find.
 * @param modifier Modifier for the band.
 * @param onLongPressCard Called when an uncovered find is held, or null.
 */
@Composable
fun BackTodayBand(
    state: BackTodayState,
    onUncover: (Card) -> Unit,
    onOpenCard: (Card) -> Unit,
    modifier: Modifier = Modifier,
    onLongPressCard: ((Card) -> Unit)? = null,
) {
    if (state.cards.isEmpty()) return
    val colors = LibraryStyle.colors
    val uncovered = state.uncovered
    Column(modifier.fillMaxWidth().padding(bottom = 22.dp).testTag("back-today")) {
        Text("Back today", Modifier.padding(start = 20.dp, bottom = 10.dp), style = LibraryStyle.label, color = colors.muted)
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            for (index in 0 until BAND_SLOTS) {
                val card = state.cards.getOrNull(index)
                if (card == null) {
                    Spacer(Modifier.weight(1f))
                    continue
                }
                key(card.id) {
                    BackTodayFind(card, covered = card.id !in uncovered, onUncover, onOpenCard, onLongPressCard, Modifier.weight(1f).aspectRatio(0.8f))
                }
            }
        }
    }
}

private const val BAND_SLOTS = 3

// One find: its tile, under leaves until it is uncovered.
@Composable
private fun BackTodayFind(
    card: Card,
    covered: Boolean,
    onUncover: (Card) -> Unit,
    onOpenCard: (Card) -> Unit,
    onLongPressCard: ((Card) -> Unit)?,
    modifier: Modifier,
) {
    val scope = rememberCoroutineScope()
    val blown = remember(card.id) { Animatable(if (covered) 0f else 1f) }
    var wind by remember(card.id) { mutableStateOf(Offset(0.5f, 0.5f)) }
    var blowing by remember(card.id) { mutableStateOf(false) }
    val latestUncover by rememberUpdatedState(onUncover)
    val latestOpen by rememberUpdatedState(onOpenCard)
    // The leaves stay while they fly, though the find counts as uncovered from the tap.
    val leaves = covered || (blowing && blown.value < 1f)
    fun blowFrom(at: Offset) {
        if (blowing) return
        blowing = true
        wind = at
        latestUncover(card)
        scope.launch {
            launch { blown.animateTo(1f, tween(BLOW_MS, easing = LinearEasing)) }
            delay(OPEN_AFTER_MS)
            latestOpen(card)
        }
    }
    Box(modifier) {
        GalleryTile(card, onClick = { onOpenCard(card) }, enabled = !leaves,
            modifier = Modifier.fillMaxSize().then(if (leaves) Modifier.clearAndSetSemantics {} else Modifier),
            onLongClick = onLongPressCard?.let { { it(card) } })
        if (leaves) {
            Box(
                Modifier.fillMaxSize()
                    .graphicsLayer()
                    .leafCover(card.id, TILE_CORNER, blown = { blown.value }, from = { wind })
                    .pointerInput(card.id) {
                        detectTapGestures { at -> blowFrom(Offset(at.x / size.width, at.y / size.height)) }
                    }
                    .clearAndSetSemantics {
                        role = Role.Button
                        contentDescription = "A find from your past, under fallen leaves"
                        onClick("Uncover") { blowFrom(Offset(0.5f, 0.5f)); true }
                    },
            )
        }
    }
}

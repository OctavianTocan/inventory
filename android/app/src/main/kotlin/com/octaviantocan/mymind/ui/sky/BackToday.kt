package com.octaviantocan.mymind.ui.sky

import android.graphics.RuntimeShader
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
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.onPlaced
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.AppJson
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.LibraryMemory
import com.octaviantocan.mymind.data.Sky
import com.octaviantocan.mymind.data.SkyStar
import com.octaviantocan.mymind.ui.library.GalleryTile
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.builtins.serializer
import java.time.Instant
import java.time.LocalDate

/**
 * Back today as the band shows it.
 *
 * @property sky The day's sky, or null before one is known for today.
 * @property cards Today's finds that the library still holds, in the server's order.
 * @property finds The sky's finds that the library still holds, by id, once the sky has asked for them.
 * @property seen The finds whose stars the person has seen light in the sky, or null before that is read.
 * @property arriving Finds opened from Back today whose star has not reached the star button yet.
 */
data class BackTodayState(
    val sky: Sky? = null,
    val cards: List<Card> = emptyList(),
    val finds: Map<String, Card> = emptyMap(),
    val seen: Set<String>? = null,
    val arriving: Set<String> = emptySet(),
) {
    /** The finds opened from Back today or the sky; they show as they are, no longer a hologram. */
    val uncovered: Set<String> get() = sky?.stars?.mapTo(HashSet()) { it.id }.orEmpty()

    /** True when the sky has anything to show: a constellation forming or finished, or a find opened again. */
    val hasSky: Boolean get() = sky != null && (sky.active != null || sky.done.isNotEmpty() || sky.stars.isNotEmpty())

    /** How many finds wait to light their stars the next time the person looks up, for the star button's badge. */
    val waiting: Int get() = if (seen == null) 0 else uncovered.count { it !in seen && it !in arriving }
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
                // A visit still waiting to be sent stays uncovered, and its star lit, until the server has it.
                val waiting = mutableState.value.sky?.takeIf { it.day == day }?.stars.orEmpty().filter { star -> answer.value.stars.none { it.id == star.id } }
                val sky = withVisits(answer.value, waiting, Instant.now().toString())
                show(sky)
                memory?.saveSky(AppJson.encodeToString(Sky.serializer(), sky))
            }
        }
    }

    /**
     * Asks the server again once any read running has ended, as after a constellation is finished: the server then forms
     * the next one.
     */
    fun reload() {
        val running = loading
        scope.launch {
            running?.join()
            load()
        }
    }

    /** Asks again once the date has changed since the band last showed, as when the app comes back after midnight. */
    fun loadIfNewDay() {
        if (mutableState.value.sky?.day != today()) load()
    }

    /** Reads which stars the person has seen light; on the first run every star already there counts as seen. */
    private suspend fun readSeen(sky: Sky) {
        if (mutableState.value.seen != null) return
        val kept = memory?.skySeen()?.let { json -> runCatching { AppJson.decodeFromString(SEEN, json) }.getOrNull() }
        val seen = kept?.toSet() ?: sky.stars.mapTo(HashSet()) { it.id }.also { if (memory != null) save(it) }
        mutableState.update { it.copy(seen = seen) }
    }

    private suspend fun save(seen: Set<String>) {
        memory?.saveSkySeen(AppJson.encodeToString(SEEN, seen.toList()))
    }

    /**
     * Records that a find's star lit in front of the person, so it no longer waits on the star button.
     *
     * @param id The find.
     */
    fun markSeen(id: String) {
        val seen = mutableState.value.seen ?: return
        if (id in seen) return
        val next = seen + id
        mutableState.update { it.copy(seen = next) }
        scope.launch { save(next) }
    }

    /**
     * The star of a find opened from Back today reached the star button; it now waits there to light.
     *
     * @param id The find.
     */
    fun landed(id: String) {
        mutableState.update { it.copy(arriving = it.arriving - id) }
    }

    /** Reads the cards of every find on the sky, for the sky's stars to name and open. */
    fun loadFinds() {
        val sky = mutableState.value.sky ?: return
        val ids = (listOfNotNull(sky.active) + sky.done).flatMap { it.cards } + sky.stars.map { it.id }
        scope.launch {
            val finds = ids.distinct().mapNotNull { id -> (library.card(id) as? ApiResult.Ok)?.value?.takeIf { it.deleted == null } }
            mutableState.update { it.copy(finds = it.finds + finds.associateBy(Card::id)) }
        }
    }

    /** Reads the finds' cards again, as after the phone's copy changed: a find sent to the trash leaves the band. */
    fun reread() {
        val sky = mutableState.value.sky ?: return
        scope.launch { show(sky) }
    }

    /**
     * Uncovers a find and records the visit, which lights its star. One opened from Back today flies to the star button
     * once it closes ([arriving]); one opened from its own star lights there.
     *
     * @param card A find in the band or the sky.
     * @param arriving True for a find opened from Back today.
     */
    fun uncover(card: Card, arriving: Boolean = true) {
        val sky = mutableState.value.sky ?: return
        if (card.id in mutableState.value.uncovered) return
        val now = Instant.now().toString()
        val visited = withVisits(sky, listOf(SkyStar(card.id, now)), now)
        mutableState.update { it.copy(sky = visited, arriving = if (arriving) it.arriving + card.id else it.arriving) }
        scope.launch {
            library.visit(card.id)
            memory?.saveSky(AppJson.encodeToString(Sky.serializer(), visited))
        }
    }

    private suspend fun show(sky: Sky?) {
        if (sky == null) {
            mutableState.update { BackTodayState(seen = it.seen, arriving = it.arriving) }
            return
        }
        val cards = sky.back.mapNotNull { id -> (library.card(id) as? ApiResult.Ok)?.value?.takeIf { it.deleted == null } }
        mutableState.update { it.copy(sky = sky, cards = cards) }
        readSeen(sky)
    }
}

private val SEEN = ListSerializer(String.serializer())

/**
 * Back today: up to three old finds under Pinned, each a flickering blue hologram until it is opened (the prototype's
 * Hologram, picked by Octavian on 6 November 2026). A tap restores it from the finger outward with a burst of glints, then
 * opens it; a find already opened today shows as it is.
 *
 * @param state The band's finds.
 * @param onUncover Called as a find's restore starts, with its centre in the window's pixels.
 * @param onOpenCard Opens a find.
 * @param modifier Modifier for the band.
 * @param onLongPressCard Called when an uncovered find is held, or null.
 * @param onShake Called as a restore clears, for the page's shake.
 */
@Composable
fun BackTodayBand(
    state: BackTodayState,
    onUncover: (Card, Offset) -> Unit,
    onOpenCard: (Card) -> Unit,
    modifier: Modifier = Modifier,
    onLongPressCard: ((Card) -> Unit)? = null,
    onShake: () -> Unit = {},
) {
    if (state.cards.isEmpty()) return
    val colors = LibraryStyle.colors
    val uncovered = state.uncovered
    val shader = remember { if (SkyShaders.supported) SkyShaders.hologram() else null }
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
                    BackTodayFind(card, covered = card.id !in uncovered, shader, onUncover, onOpenCard, onLongPressCard, onShake,
                        Modifier.weight(1f).aspectRatio(0.8f))
                }
            }
        }
    }
}

private const val BAND_SLOTS = 3

// One find: its tile, a hologram until it is restored.
@Composable
private fun BackTodayFind(
    card: Card,
    covered: Boolean,
    shader: RuntimeShader?,
    onUncover: (Card, Offset) -> Unit,
    onOpenCard: (Card) -> Unit,
    onLongPressCard: ((Card) -> Unit)?,
    onShake: () -> Unit,
    modifier: Modifier,
) {
    val scope = rememberCoroutineScope()
    val hologram = remember(card.id) { Hologram(hologramStage(card)) }
    val latestUncover by rememberUpdatedState(onUncover)
    val latestOpen by rememberUpdatedState(onOpenCard)
    val latestShake by rememberUpdatedState(onShake)
    // Where the tile is, kept without state so scrolling the gallery never recomposes it; read only on a tap.
    val placed = remember(card.id) { arrayOfNulls<LayoutCoordinates>(1) }
    // It stays a hologram while it clears and its shine crosses, though it counts as uncovered from the tap.
    val worn = covered || hologram.restoring
    val density = LocalDensity.current.density
    fun restoreFrom(at: Offset) {
        if (hologram.restoring) return
        val tile = placed[0]?.takeIf { it.isAttached }
        latestUncover(card, tile?.localToRoot(Offset(tile.size.width / 2f, tile.size.height / 2f)) ?: Offset.Zero)
        val width = (tile?.size?.width ?: 0) / density
        val height = (tile?.size?.height ?: 0) / density
        scope.launch {
            hologram.restoreFrom(at, width, height, scope, onShake = { latestShake() })
            latestOpen(card)
        }
    }
    if (worn) HologramClock(hologram)
    Box(modifier.onPlaced { placed[0] = it }.graphicsLayer {
        val s = hologram.lift.value
        scaleX = s
        scaleY = s
    }) {
        GalleryTile(card, onClick = { onOpenCard(card) }, enabled = !worn,
            modifier = Modifier.fillMaxSize().then(if (worn) Modifier.hologram(hologram, shader).clearAndSetSemantics {} else Modifier),
            onLongClick = onLongPressCard?.let { { it(card) } })
        if (worn) {
            Box(
                Modifier.fillMaxSize()
                    .pointerInput(card.id) {
                        detectTapGestures { at -> restoreFrom(Offset(at.x / size.width, at.y / size.height)) }
                    }
                    .clearAndSetSemantics {
                        role = Role.Button
                        contentDescription = "A find from your past, flickering as a hologram"
                        onClick("Restore") { restoreFrom(Offset(0.5f, 0.45f)); true }
                    },
            )
        }
        HologramGlints(hologram, Modifier.fillMaxSize())
    }
}

/**
 * [sky] as the server will have it once [visits] reach it: they light their stars in the active constellation, and one
 * whose last star they light is finished, as the server finishes it, so it never shows again as the one forming.
 *
 * @param sky The sky.
 * @param visits Finds opened that the server may not have yet.
 * @param now When, for a constellation they finish.
 * @return The sky with them.
 */
internal fun withVisits(sky: Sky, visits: List<SkyStar>, now: String): Sky {
    val stars = sky.stars + visits
    val active = sky.active ?: return sky.copy(stars = stars)
    val lit = (active.lit + visits.map { it.id }.filter { it in active.cards }).distinct()
    val lighted = active.copy(lit = lit)
    if (active.cards.isEmpty() || !active.cards.all { it in lit }) return sky.copy(stars = stars, active = lighted)
    return sky.copy(stars = stars, active = null, done = sky.done + lighted.copy(completed = now))
}

package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.FEED_MEMORY_CARDS
import com.octaviantocan.mymind.data.FeedMemory
import com.octaviantocan.mymind.ui.describe
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * What a paged list of cards shows.
 *
 * @property cards Cards loaded so far, in the server's order.
 * @property loading True while a page is being fetched.
 * @property endReached True once the server has no further page.
 * @property error Text for the last failed fetch, or null.
 * @property refreshing True while pull to refresh reads the first page again.
 */
data class FeedState(
    val cards: List<Card> = emptyList(),
    val loading: Boolean = false,
    val endReached: Boolean = false,
    val error: String? = null,
    val refreshing: Boolean = false,
) {
    /** True before the first page arrived, while it loads or after it failed. */
    val empty: Boolean get() = cards.isEmpty() && !endReached
}

/**
 * A list of cards read page by page from the server.
 *
 * With a [memory], the first load shows the cards kept from last time at once and then checks them against the server's
 * first page: when that page starts the list as kept, the list stays and paging goes on after it; otherwise the list
 * becomes the fresh page. Whatever the list then shows is kept for next time.
 *
 * @param scope Scope the fetches run in.
 * @param memory Where the list's first cards are kept between launches, or null to keep nothing.
 * @param load Reads the page after a cursor; null asks for the first page.
 */
class CardFeed(
    private val scope: CoroutineScope,
    private val memory: FeedMemory? = null,
    private val load: suspend (cursor: String?) -> ApiResult<CardPage>,
) {
    private val mutableState = MutableStateFlow(FeedState())
    private var cursor: String? = null
    private var job: Job? = null
    private var refreshJob: Job? = null
    private var rememberJob: Job? = null
    private var rereadJob: Job? = null
    private var remembered: List<Card>? = null
    private var restored = false
    private var unconfirmed = false

    /** The cards and the loading state. */
    val state: StateFlow<FeedState> = mutableState

    /** Fetches the next page, or retries the last one after a failure. Does nothing while a fetch runs or at the end. */
    fun loadMore() {
        val current = mutableState.value
        if (current.loading || current.endReached) return
        mutableState.value = current.copy(loading = true, error = null)
        job = scope.launch {
            if (!restored) {
                restored = true
                val kept = memory?.read().orEmpty()
                if (kept.isNotEmpty() && mutableState.value.cards.isEmpty()) {
                    unconfirmed = true
                    remembered = kept
                    mutableState.update { it.copy(cards = kept.distinctBy { card -> card.id }) }
                }
            }
            when (val result = load(cursor)) {
                is ApiResult.Ok -> {
                    val page = result.value
                    cursor = page.next
                    if (unconfirmed) {
                        unconfirmed = false
                        val shown = mutableState.value.cards
                        mutableState.value = if (startsWith(shown, page)) {
                            mutableState.value.copy(loading = false, endReached = page.next == null)
                        } else {
                            FeedState(cards = page.cards.distinctBy { it.id }, endReached = page.next == null)
                        }
                    } else {
                        mutableState.update { state ->
                            val known = state.cards.mapTo(HashSet()) { it.id }
                            state.copy(
                                cards = state.cards + page.cards.filter { it.id !in known },
                                loading = false,
                                endReached = page.next == null,
                            )
                        }
                    }
                    remember()
                }
                is ApiResult.Failed -> mutableState.update { it.copy(loading = false, error = describe(result.failure)) }
            }
        }
    }

    /** Drops what is loaded and reads the first page again. */
    fun reload() {
        job?.cancel()
        refreshJob?.cancel()
        rereadJob?.cancel()
        cursor = null
        unconfirmed = false
        mutableState.value = FeedState()
        loadMore()
    }

    /**
     * Reads the first page again for pull to refresh, keeping the cards on show until it arrives. When the list
     * already starts with that page, nothing changes; otherwise the list becomes the fresh page. A failed read leaves
     * a loaded list as it is and turns an empty one into the error. Does nothing while a refresh runs.
     *
     * @param prepare Runs first while the refresh shows, such as a sync of the phone's copy; when it returns true, every
     *   page the list shows is read again, as [reread] does, instead of the first page alone.
     */
    fun refresh(prepare: (suspend () -> Boolean)? = null) {
        if (mutableState.value.refreshing) return
        if (mutableState.value.cards.isEmpty()) return reload()
        mutableState.update { it.copy(refreshing = true) }
        refreshJob = scope.launch {
            if (prepare?.invoke() == true) {
                if (!rereadPages()) mutableState.update { it.copy(refreshing = false) }
                return@launch
            }
            when (val result = load(null)) {
                is ApiResult.Ok -> {
                    val fresh = result.value
                    if (startsWith(mutableState.value.cards, fresh) && unconfirmed) {
                        job?.cancel()
                        unconfirmed = false
                        cursor = fresh.next
                        mutableState.update { it.copy(refreshing = false, loading = false, error = null, endReached = fresh.next == null) }
                    } else if (startsWith(mutableState.value.cards, fresh)) {
                        mutableState.update { it.copy(refreshing = false) }
                    } else {
                        job?.cancel()
                        cursor = fresh.next
                        unconfirmed = false
                        mutableState.value = FeedState(cards = fresh.cards.distinctBy { it.id }, endReached = fresh.next == null)
                        remember()
                    }
                }
                is ApiResult.Failed -> mutableState.update { it.copy(refreshing = false) }
            }
        }
    }

    /**
     * Reads every page the list shows again and puts the result in its place when it differs, without emptying the list
     * first; for lists read from the phone's copy, where pages cost nothing. Does nothing while a refresh runs, and a
     * failed read leaves the list as it is.
     */
    fun reread() {
        if (mutableState.value.refreshing) return
        rereadJob?.cancel()
        rereadJob = scope.launch { rereadPages() }
    }

    /**
     * Puts a new card at the front of the list without asking the server.
     *
     * @param card The card; one the list holds already moves to the front.
     */
    fun prepend(card: Card) {
        mutableState.update { state -> state.copy(cards = listOf(card) + state.cards.filterNot { it.id == card.id }) }
        remember()
    }

    /**
     * Takes a card out of the list without asking the server.
     *
     * @param cardId Card to drop.
     */
    fun remove(cardId: String) {
        mutableState.update { state -> state.copy(cards = state.cards.filterNot { it.id == cardId }) }
        remember()
    }

    /**
     * Swaps in a changed copy of a card that is in the list.
     *
     * @param card The changed card; a card the list does not hold is ignored.
     */
    fun replace(card: Card) {
        mutableState.update { state -> state.copy(cards = state.cards.map { if (it.id == card.id) card else it }) }
        remember()
    }

    // Reads from the first page until it has as many cards as the list shows, then shows that; false when a read failed.
    private suspend fun rereadPages(): Boolean {
        val wanted = mutableState.value.cards.size.coerceAtLeast(1)
        val read = mutableListOf<Card>()
        var next: String? = null
        do {
            when (val result = load(next)) {
                is ApiResult.Ok -> {
                    read += result.value.cards
                    next = result.value.next
                }
                is ApiResult.Failed -> return false
            }
        } while (next != null && read.size < wanted)
        job?.cancel()
        cursor = next
        unconfirmed = false
        val cards = read.distinctBy { it.id }
        mutableState.value = if (cards == mutableState.value.cards) {
            mutableState.value.copy(loading = false, refreshing = false, error = null, endReached = next == null)
        } else {
            FeedState(cards = cards, endReached = next == null)
        }
        remember()
        return true
    }

    // Keeps the list's first cards for next time, when they changed; a newer list replaces a write still waiting.
    private fun remember() {
        val memory = memory ?: return
        val first = mutableState.value.cards.take(FEED_MEMORY_CARDS)
        if (first == remembered) return
        remembered = first
        rememberJob?.cancel()
        rememberJob = scope.launch { memory.write(first) }
    }

    // True when the list already starts with the first page, and holds no more than it when the page is the last.
    private fun startsWith(shown: List<Card>, first: CardPage): Boolean =
        shown.take(first.cards.size) == first.cards && (first.next != null || shown.size == first.cards.size)
}

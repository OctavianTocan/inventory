package com.octaviantocan.mymind.ui.library

import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
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
 * @param scope Scope the fetches run in.
 * @param load Reads the page after a cursor; null asks for the first page.
 */
class CardFeed(private val scope: CoroutineScope, private val load: suspend (cursor: String?) -> ApiResult<CardPage>) {
    private val mutableState = MutableStateFlow(FeedState())
    private var cursor: String? = null
    private var job: Job? = null
    private var refreshJob: Job? = null

    /** The cards and the loading state. */
    val state: StateFlow<FeedState> = mutableState

    /** Fetches the next page, or retries the last one after a failure. Does nothing while a fetch runs or at the end. */
    fun loadMore() {
        val current = mutableState.value
        if (current.loading || current.endReached) return
        mutableState.value = current.copy(loading = true, error = null)
        job = scope.launch {
            when (val result = load(cursor)) {
                is ApiResult.Ok -> {
                    cursor = result.value.next
                    mutableState.update { state ->
                        val known = state.cards.mapTo(HashSet()) { it.id }
                        state.copy(
                            cards = state.cards + result.value.cards.filter { it.id !in known },
                            loading = false,
                            endReached = result.value.next == null,
                        )
                    }
                }
                is ApiResult.Failed -> mutableState.update { it.copy(loading = false, error = describe(result.failure)) }
            }
        }
    }

    /** Drops what is loaded and reads the first page again. */
    fun reload() {
        job?.cancel()
        refreshJob?.cancel()
        cursor = null
        mutableState.value = FeedState()
        loadMore()
    }

    /**
     * Reads the first page again for pull to refresh, keeping the cards on show until it arrives. When the list
     * already starts with that page, nothing changes; otherwise the list becomes the fresh page. A failed read leaves
     * a loaded list as it is and turns an empty one into the error. Does nothing while a refresh runs.
     */
    fun refresh() {
        if (mutableState.value.refreshing) return
        if (mutableState.value.cards.isEmpty()) return reload()
        mutableState.update { it.copy(refreshing = true) }
        refreshJob = scope.launch {
            when (val result = load(null)) {
                is ApiResult.Ok -> {
                    val fresh = result.value
                    val shown = mutableState.value.cards
                    val unchanged = shown.take(fresh.cards.size) == fresh.cards && (fresh.next != null || shown.size == fresh.cards.size)
                    if (unchanged) {
                        mutableState.update { it.copy(refreshing = false) }
                    } else {
                        job?.cancel()
                        cursor = fresh.next
                        mutableState.value = FeedState(cards = fresh.cards.distinctBy { it.id }, endReached = fresh.next == null)
                    }
                }
                is ApiResult.Failed -> mutableState.update { it.copy(refreshing = false) }
            }
        }
    }

    /**
     * Puts a new card at the front of the list without asking the server.
     *
     * @param card The card; one the list holds already moves to the front.
     */
    fun prepend(card: Card) {
        mutableState.update { state -> state.copy(cards = listOf(card) + state.cards.filterNot { it.id == card.id }) }
    }

    /**
     * Takes a card out of the list without asking the server.
     *
     * @param cardId Card to drop.
     */
    fun remove(cardId: String) {
        mutableState.update { state -> state.copy(cards = state.cards.filterNot { it.id == cardId }) }
    }

    /**
     * Swaps in a changed copy of a card that is in the list.
     *
     * @param card The changed card; a card the list does not hold is ignored.
     */
    fun replace(card: Card) {
        mutableState.update { state -> state.copy(cards = state.cards.map { if (it.id == card.id) card else it }) }
    }
}

package com.octaviantocan.mymind.ui.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.data.AiRanking
import com.octaviantocan.mymind.data.AiUnderstood
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.CardPage
import com.octaviantocan.mymind.data.Library
import com.octaviantocan.mymind.data.LibraryMemory
import com.octaviantocan.mymind.data.LibrarySync
import com.octaviantocan.mymind.data.RecentSearches
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.data.SyncOutcome
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.data.UploadFile
import com.octaviantocan.mymind.data.map
import com.octaviantocan.mymind.update.UploadActivity
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.ui.menu.CardActions
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuItem
import com.octaviantocan.mymind.ui.menu.CardMenuOutcome
import com.octaviantocan.mymind.ui.menu.CardSubmenu
import com.octaviantocan.mymind.ui.menu.FileSaver
import com.octaviantocan.mymind.ui.sky.BackTodayModel
import com.octaviantocan.mymind.ui.theme.PageScheme
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import java.io.File

/**
 * What the Spaces tab shows.
 *
 * @property spaces Spaces in the user's order.
 * @property covers First card of each space by space id, once read.
 * @property latest The first cards of each space by space id, up to [SPACE_LATEST_CARDS], for its mosaic and peek.
 * @property loading True while the list is being fetched.
 * @property error Text for the last failed fetch, or null.
 * @property loaded True once the list was read, even when it is empty.
 * @property refreshing True while pull to refresh reads the list again.
 */
data class SpacesState(
    val spaces: List<Space> = emptyList(),
    val covers: Map<String, Card> = emptyMap(),
    val latest: Map<String, List<Card>> = emptyMap(),
    val loading: Boolean = false,
    val error: String? = null,
    val loaded: Boolean = false,
    val refreshing: Boolean = false,
)

/**
 * Tags the Add tags page offers.
 *
 * @property all Every tag of the library.
 * @property recent Tags added last.
 */
data class TagChoices(val all: List<String> = emptyList(), val recent: List<String> = emptyList())

/**
 * A card-sheet creation keeps running when its form leaves composition.
 *
 * @property pending True while the space is being made and the card added to it.
 * @property result The new space's id, or why it could not be made.
 * @property card The card after it joined the new space, or null when none was added.
 */
data class CardSpaceCreationState(
    val pending: Boolean = false,
    val result: ApiResult<String>? = null,
    val card: Card? = null,
)

/**
 * The library behind the three tabs, search, the card menu and the upload queue.
 *
 * @param library Reads and writes of the account's library.
 * @param saver Where Download puts a card's file.
 * @param io Dispatcher picked files are read on.
 * @param memory What the gallery keeps on the phone, so it opens on last time's cards and page scheme; null keeps nothing.
 * @param sync Keeps the phone's copy of the library up to date: at start, on [syncNow] and on pull to refresh; null when
 *   there is no copy.
 * @param changes Signals each change to the phone's copy while the screens read from it, which they then show without
 *   a sync of their own asking for it; null rereads the lists after each sync [syncNow] runs instead.
 * @param ai Ranks a pressed search, or null for none.
 * @param recentSearches Where pressed searches are kept on the phone, or null to keep none.
 */
class LibraryViewModel(
    private val library: Library,
    saver: FileSaver = FileSaver { _, _, _ -> ApiResult.Failed(ApiFailure.Server(0)) },
    io: CoroutineDispatcher = Dispatchers.IO,
    uploadActivity: UploadActivity? = null,
    private val memory: LibraryMemory? = null,
    private val sync: LibrarySync? = null,
    private val changes: Flow<Unit>? = null,
    ai: AiRanking? = null,
    recentSearches: RecentSearches? = null,
) : ViewModel() {
    private val mutableRedrawn = MutableSharedFlow<Unit>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)
    private val mutableScheme = MutableStateFlow(PageScheme.System)
    private val mutableFeatures = MutableStateFlow(emptySet<String>())
    private val mutableSpaces = MutableStateFlow(SpacesState())
    private val mutableTagChoices = MutableStateFlow(TagChoices())
    private val mutableCardSpaceCreation = MutableStateFlow(CardSpaceCreationState())
    private val spaceFeeds = mutableMapOf<String, CardFeed>()
    private val actions = CardActions(library, saver)
    private val mutableDeleting = MutableStateFlow<Card?>(null)
    private val mutableDeleteFailures = MutableSharedFlow<ApiFailure>(extraBufferCapacity = 1)
    private var deleteJob: Job? = null
    private var deletedFrom: List<Pair<CardFeed, Int>> = emptyList()

    /** The Everything tab's cards, newest first. */
    val everything = CardFeed(viewModelScope, memory?.feed("everything")) { shown(library.cards(it)) }

    /** Cards pinned, in pin order. */
    val pinned = CardFeed(viewModelScope, memory?.feed("top-of-mind")) { shown(library.pinned(it)) }

    /** Back today: the day's old finds under Pinned. */
    val backToday = BackTodayModel(viewModelScope, library, memory)

    /** The card waiting out its Undo before it goes to the trash, or null. */
    val deleting: StateFlow<Card?> = mutableDeleting

    /** Deletes the server refused once their Undo ran out; the card is back in its lists. */
    val deleteFailures: SharedFlow<ApiFailure> = mutableDeleteFailures

    /** Files being uploaded from the + button. */
    val uploads = UploadQueue(viewModelScope, library, onUploaded = ::showUploaded, io = io, activity = uploadActivity)

    /** Appearance of the library pages, from the account's setting. */
    val scheme: StateFlow<PageScheme> = mutableScheme

    /** Features the account's plan unlocks, from the web page config; empty until read. */
    val features: StateFlow<Set<String>> = mutableFeatures

    /** The Spaces tab. */
    val spaces: StateFlow<SpacesState> = mutableSpaces

    /** Tags the Add tags page offers; read by [loadTagChoices]. */
    val tagChoices: StateFlow<TagChoices> = mutableTagChoices

    /** Pending creation and its last result, shared by every card-sheet form instance. */
    val cardSpaceCreation: StateFlow<CardSpaceCreationState> = mutableCardSpaceCreation

    /** Search text, filter, suggestions and results. */
    val search = SearchModel(viewModelScope, library, ai, recentSearches)

    /** One signal each time the lists redraw from the phone's copy, for screens holding a card of their own to read it again. */
    val redrawn: SharedFlow<Unit> = mutableRedrawn

    init {
        everything.loadMore()
        pinned.loadMore()
        backToday.load()
        changes?.let { copyChanges ->
            viewModelScope.launch {
                copyChanges.collect {
                    showCopy()
                    delay(COPY_REDRAW_GAP_MS)
                }
            }
        }
        syncNow()
        var schemeRead = false
        viewModelScope.launch {
            val kept = memory?.scheme()?.let { name -> PageScheme.entries.firstOrNull { it.name == name } }
            if (kept != null && !schemeRead) mutableScheme.value = kept
        }
        viewModelScope.launch {
            val scheme = when (val config = library.pageConfig()) {
                is ApiResult.Ok -> {
                    mutableFeatures.value = config.value.features
                    PageScheme.of(config.value.colorScheme)
                }
                is ApiResult.Failed -> (library.me() as? ApiResult.Ok)?.value?.colorScheme?.let(PageScheme::of)
            }
            if (scheme != null) {
                schemeRead = true
                mutableScheme.value = scheme
                memory?.saveScheme(scheme.name)
            }
        }
    }

    /** Reads the spaces and the first card of each; does nothing while a read runs. */
    fun loadSpaces() {
        if (mutableSpaces.value.loading) return
        mutableSpaces.update { it.copy(loading = true, error = null) }
        viewModelScope.launch {
            when (val answer = library.spaces()) {
                is ApiResult.Ok -> {
                    mutableSpaces.update { it.copy(spaces = answer.value.spaces, loading = false, loaded = true) }
                    answer.value.spaces.forEach { space -> launch { readCover(space.id) } }
                }
                is ApiResult.Failed -> mutableSpaces.update { it.copy(loading = false, error = describe(answer.failure)) }
            }
        }
    }

    /**
     * Pull to refresh on Everything: syncs the phone's copy, then reads the library and Pinned again; every page
     * shown when the sync brought changes, the first page otherwise.
     */
    fun refreshEverything() {
        backToday.load()
        val sync = sync ?: return run {
            everything.refresh()
            pinned.refresh()
        }
        val pulled = viewModelScope.async { withTimeoutOrNull(REFRESH_SYNC_WAIT_MS) { sync.sync() }.changedAnything() }
        everything.refresh { pulled.await() }
        pinned.refresh { pulled.await() }
    }

    /** Syncs the phone's copy, as when the app comes back or the network returns, and shows what changed. */
    fun syncNow() {
        backToday.loadIfNewDay()
        val sync = sync ?: return
        viewModelScope.launch {
            if (sync.sync().changedAnything() && changes == null) showCopy()
        }
    }

    /**
     * Reads a card again, as the screens show it now.
     *
     * @param cardId Card id.
     * @return The card, or null when it cannot be read.
     */
    suspend fun cardNow(cardId: String): Card? = (library.card(cardId) as? ApiResult.Ok)?.value

    /**
     * @param card A card holding a file.
     * @param into Where to write the file.
     * @return True once the library server's copy of the card's file is written to [into]; false without a file or
     *   when it could not be fetched.
     */
    suspend fun fetchFile(card: Card, into: File): Boolean {
        val path = card.blob?.path ?: return false
        return library.media(path) { input -> into.outputStream().use { input.copyTo(it) } } is ApiResult.Ok
    }

    /**
     * @param card A card.
     * @return Up to [limit] cards nearest to it in meaning (Same Vibe), nearest first and without the card itself;
     *   empty when the search fails or finds none.
     */
    suspend fun sameVibe(card: Card, limit: Int = 3): List<Card> =
        (library.search("similar:${card.id}", null) as? ApiResult.Ok)?.value?.cards.orEmpty().filter { it.id != card.id }.take(limit)

    // Reads every list on show again from the phone's copy, keeping each as it is when nothing in it changed.
    private suspend fun showCopy() {
        everything.reread()
        pinned.reread()
        backToday.reread()
        spaceFeeds.values.forEach { it.reread() }
        if (mutableSpaces.value.loaded && !mutableSpaces.value.loading) readSpaces()
        mutableRedrawn.tryEmit(Unit)
    }

    /**
     * Pull to refresh on the Spaces list: syncs the phone's copy, then reads the spaces and their first cards again,
     * keeping them on show.
     */
    fun refreshSpaces() {
        if (mutableSpaces.value.refreshing || mutableSpaces.value.loading) return
        if (!mutableSpaces.value.loaded) return loadSpaces()
        mutableSpaces.update { it.copy(refreshing = true) }
        viewModelScope.launch {
            sync?.let { withTimeoutOrNull(REFRESH_SYNC_WAIT_MS) { it.sync() } }
            readSpaces()
        }
    }

    /**
     * Pull to refresh in an opened space: syncs the phone's copy, then reads its pages and its entry in the Spaces list
     * again.
     *
     * @param spaceId The space.
     */
    fun refreshSpaceFeed(spaceId: String) {
        val sync = sync ?: return run {
            spaceFeed(spaceId).refresh()
            refreshSpace(spaceId)
        }
        val pulled = viewModelScope.async { withTimeoutOrNull(REFRESH_SYNC_WAIT_MS) { sync.sync() }.changedAnything() }
        spaceFeed(spaceId).refresh { pulled.await() }
        viewModelScope.launch {
            pulled.await()
            refreshSpace(spaceId)
        }
    }

    /**
     * Uploads picked files, the + button's job; each new card appears at the top of the library when it is done.
     *
     * @param files Picked files.
     * @param spaceId Space the + was tapped in, or null on Everything.
     */
    fun upload(files: List<UploadFile>, spaceId: String?) = uploads.add(files, spaceId)

    /**
     * @param spaceId Space to list.
     * @return The space's cards, read on first use and kept while this view model lives.
     */
    fun spaceFeed(spaceId: String): CardFeed = spaceFeeds.getOrPut(spaceId) {
        CardFeed(viewModelScope) { shown(library.spaceCards(spaceId, it)) }.also { it.loadMore() }
    }

    /**
     * Carries out a card menu row and updates every list holding the card.
     *
     * @param item Row the user tapped.
     * @param card Card the menu is for.
     * @param spaceId Space the card was opened from, or null.
     * @return What happened, for the caller to finish (clipboard, browser, closing the card).
     */
    suspend fun perform(item: CardMenuItem, card: Card, spaceId: String?): CardMenuOutcome {
        val outcome = actions.perform(item, card, spaceId)
        Track.event("action", screen = if (spaceId != null) "space" else "", card = card.id, kind = card.type, detail = item.action.name)
        when (outcome) {
            is CardMenuOutcome.Deleted -> feeds().forEach { it.remove(outcome.cardId) }
            is CardMenuOutcome.RemovedFromSpace -> {
                spaceFeeds[outcome.spaceId]?.remove(outcome.cardId)
                replaceEverywhere(card.copy(spaces = card.spaces.orEmpty() - outcome.spaceId))
                refreshSpace(outcome.spaceId)
            }
            is CardMenuOutcome.PinChanged -> {
                replaceEverywhere(outcome.card)
                pinned.refresh()
            }
            is CardMenuOutcome.ShowSimilar -> search.similarTo(outcome.card)
            is CardMenuOutcome.OpenSubmenu -> when (outcome.submenu) {
                CardSubmenu.Tags -> loadTagChoices()
                CardSubmenu.Spaces -> if (!mutableSpaces.value.loaded) loadSpaces()
                CardSubmenu.Note, CardSubmenu.CardImage -> Unit
            }
            else -> Unit
        }
        return outcome
    }

    /** Reads every tag and the recent ones for the Add tags page. */
    fun loadTagChoices() {
        viewModelScope.launch {
            val all = (library.tags() as? ApiResult.Ok)?.value
            val recent = (library.recentTags() as? ApiResult.Ok)?.value
            mutableTagChoices.update { TagChoices(all ?: it.all, recent ?: it.recent) }
        }
    }

    /**
     * Adds tags to a card, one request each, and updates every list holding it.
     *
     * @param card Card to tag.
     * @param names Tags to add.
     * @return The card with its tags as the server answered, or the first failure; tags added before it stay.
     */
    suspend fun addTags(card: Card, names: List<String>): ApiResult<Card> {
        var tags = card.tags.orEmpty()
        var failure: ApiResult.Failed? = null
        for (name in names) {
            when (val answer = library.addTag(card.id, name)) {
                is ApiResult.Ok -> if (tags.none { it.id != null && it.id == answer.value.id }) tags = tags + answer.value
                is ApiResult.Failed -> {
                    failure = answer
                    break
                }
            }
        }
        val updated = card.copy(tags = tags)
        if (tags != card.tags.orEmpty()) replaceEverywhere(updated)
        return failure ?: ApiResult.Ok(updated)
    }

    /**
     * Writes the user's note on a card and updates every list holding it.
     *
     * @param card Card the note is on.
     * @param text The whole note; blank takes the note off.
     * @return The card with its new note, or the failure.
     */
    suspend fun setNote(card: Card, text: String): ApiResult<Card> {
        val note = text.trim()
        return when (val answer = library.setNote(card.id, note)) {
            is ApiResult.Ok -> ApiResult.Ok(card.copy(note = note).also(::replaceEverywhere))
            is ApiResult.Failed -> answer
        }
    }

    /**
     * Takes a tag off a card and updates every list holding it.
     *
     * @param card Card the tag is on.
     * @param tag Tag to take off.
     * @return The card without the tag, or the failure.
     */
    suspend fun removeTag(card: Card, tag: Tag): ApiResult<Card> {
        val id = tag.id ?: return ApiResult.Failed(ApiFailure.Server(0))
        return library.removeTag(id).map {
            card.copy(tags = card.tags.orEmpty().filterNot { it.id == id }).also(::replaceEverywhere)
        }
    }

    /**
     * Puts a card into a space, or takes it out when it is in already.
     *
     * @param card Card to move.
     * @param space Space to change.
     * @return The card with its new spaces, or the failure.
     */
    suspend fun toggleSpace(card: Card, space: Space): ApiResult<Card> {
        val member = space.id in card.spaces.orEmpty()
        val answer = if (member) library.removeFromSpace(space.id, card.id) else library.addToSpace(space.id, card.id)
        return answer.map {
            val updated = card.copy(spaces = if (member) card.spaces.orEmpty() - space.id else card.spaces.orEmpty() + space.id)
            replaceEverywhere(updated)
            if (member) spaceFeeds[space.id]?.remove(card.id) else spaceFeeds.remove(space.id)
            refreshSpace(space.id)
            updated
        }
    }

    /**
     * Takes a card out of every list at once and moves it to the trash once [UNDO_DELETE_MS] pass without [undoDelete].
     * A delete still waiting goes to the trash at once when another starts.
     *
     * @param card Card to delete.
     */
    fun deleteLater(card: Card) {
        mutableDeleting.value?.let { waiting ->
            deleteJob?.cancel()
            trash(waiting)
        }
        deletedFrom = feeds().mapNotNull { feed -> feed.state.value.cards.indexOfFirst { it.id == card.id }.takeIf { it >= 0 }?.let { feed to it } }
        deletedFrom.forEach { (feed, _) -> feed.remove(card.id) }
        mutableDeleting.value = card
        deleteJob = viewModelScope.launch {
            delay(UNDO_DELETE_MS)
            trash(card)
        }
    }

    /** Puts the card waiting to be deleted back where it was in each list, and keeps it. */
    fun undoDelete() {
        val card = mutableDeleting.value ?: return
        deleteJob?.cancel()
        mutableDeleting.value = null
        deletedFrom.forEach { (feed, index) -> feed.insert(card, index) }
        deletedFrom = emptyList()
    }

    // Sends a waiting delete; a refusal puts the card back and reports why.
    private fun trash(card: Card) {
        val places = deletedFrom
        deletedFrom = emptyList()
        viewModelScope.launch {
            val outcome = perform(CardMenuItem(CardMenuAction.DeleteCard), card, null)
            if (mutableDeleting.value?.id == card.id) mutableDeleting.value = null
            if (outcome is CardMenuOutcome.Failed) {
                places.forEach { (feed, index) -> feed.insert(card, index) }
                mutableDeleteFailures.tryEmit(outcome.failure)
            }
        }
    }

    // A page without the card waiting out its Undo, so a list read again meanwhile does not bring it back.
    private fun shown(page: ApiResult<CardPage>): ApiResult<CardPage> {
        val gone = mutableDeleting.value?.id ?: return page
        return page.map { it.copy(cards = it.cards.filterNot { card -> card.id == gone }) }
    }

    /**
     * Creates a space and reads the list again.
     *
     * @param name Space name.
     * @param color Space colour as a CSS hex colour, such as `#f96` or `#75ffc0`.
     * @return The new space's id, or the failure.
     */
    suspend fun createSpace(name: String, color: String): ApiResult<String> =
        library.createSpace(name, color).also { if (it is ApiResult.Ok) reloadSpaces() }

    /**
     * Starts one card-sheet creation in the model's scope, then adds the card to the new space. Leaving the form cannot
     * cancel either step or the list reload.
     *
     * @param name Trimmed, nonblank space name.
     * @param color Selected CSS hex color.
     * @param card Card to add to the new space, or null to only make it.
     */
    fun createCardSpace(name: String, color: String, card: Card? = null) {
        if (mutableCardSpaceCreation.value.pending || name.isBlank()) return
        mutableCardSpaceCreation.value = CardSpaceCreationState(pending = true)
        viewModelScope.launch {
            val answer = createSpace(name, color)
            val id = (answer as? ApiResult.Ok)?.value
            val added = if (id != null && card != null) (toggleSpace(card, Space(id, name, color)) as? ApiResult.Ok)?.value else null
            mutableCardSpaceCreation.value = CardSpaceCreationState(result = answer, card = added)
        }
    }

    /** Clears a completed creation once the active form has returned to the selector. */
    fun acknowledgeCardSpaceCreation() {
        if (!mutableCardSpaceCreation.value.pending) mutableCardSpaceCreation.value = CardSpaceCreationState()
    }

    /**
     * Deletes a space; its cards stay in the library.
     *
     * @param space Space to delete.
     * @return Success, or the failure.
     */
    suspend fun deleteSpace(space: Space): ApiResult<Unit> = library.deleteSpace(space.id).also { answer ->
        if (answer is ApiResult.Ok) {
            spaceFeeds.remove(space.id)
            mutableSpaces.update { state ->
                state.copy(spaces = state.spaces.filterNot { it.id == space.id }, covers = state.covers - space.id, latest = state.latest - space.id)
            }
            feeds().forEach { feed ->
                feed.state.value.cards.filter { space.id in it.spaces.orEmpty() }.forEach { feed.replace(it.copy(spaces = it.spaces.orEmpty() - space.id)) }
            }
        }
    }

    private suspend fun showUploaded(cardId: String, spaceId: String?) {
        when (val answer = library.card(cardId)) {
            is ApiResult.Ok -> everything.prepend(answer.value)
            is ApiResult.Failed -> everything.refresh()
        }
        if (spaceId != null) {
            spaceFeeds[spaceId]?.refresh()
            refreshSpace(spaceId)
        }
    }

    // Reads the spaces and their first cards again, keeping the list on show, and ends a pull to refresh.
    private suspend fun readSpaces() = coroutineScope {
        when (val answer = library.spaces()) {
            is ApiResult.Ok -> {
                val fresh = answer.value.spaces
                fresh.map { space -> launch { readCover(space.id) } }.forEach { it.join() }
                mutableSpaces.update { state ->
                    val kept = fresh.mapTo(HashSet()) { it.id }
                    state.copy(spaces = fresh, covers = state.covers.filterKeys { it in kept }, latest = state.latest.filterKeys { it in kept }, refreshing = false)
                }
            }
            is ApiResult.Failed -> mutableSpaces.update { it.copy(refreshing = false) }
        }
    }

    private fun reloadSpaces() {
        mutableSpaces.update { it.copy(loading = false) }
        loadSpaces()
    }

    private suspend fun readCover(spaceId: String) {
        val cards = (library.spaceCards(spaceId, null) as? ApiResult.Ok)?.value?.cards.orEmpty().take(SPACE_LATEST_CARDS)
        val first = cards.firstOrNull()
        mutableSpaces.update { state ->
            state.copy(
                covers = if (first != null) state.covers + (spaceId to first) else state.covers - spaceId,
                latest = if (cards.isNotEmpty()) state.latest + (spaceId to cards) else state.latest - spaceId,
            )
        }
    }

    private fun refreshSpace(spaceId: String) {
        viewModelScope.launch {
            readCover(spaceId)
            (library.spaces() as? ApiResult.Ok)?.value?.spaces?.let { fresh -> mutableSpaces.update { it.copy(spaces = fresh) } }
        }
    }

    private fun replaceEverywhere(card: Card) = feeds().forEach { it.replace(card) }

    private fun feeds() = listOf(everything, pinned, search.results.value) + spaceFeeds.values
}

/**
 * A filter the search is limited to, shown as a chip.
 *
 * @property label Chip text.
 * @property queryText The filter in mymind's query syntax, such as `type:article`, `tag:"road trip"` or
 *   `similar:<card id>`.
 */
data class SearchFilter(val label: String, val queryText: String) {
    /** True for Same Vibe, which the server answers on its own and never together with text. */
    val similar: Boolean get() = queryText.startsWith("similar:")
}

/** Label of the Same Vibe filter, as on the web. */
const val SAME_VIBE_LABEL = "Same Vibe"

/** What Claude Code or Codex is doing with a pressed search, whose instant results stay on show meanwhile. */
sealed interface AiRank {
    /** [engine] is reading what the search means. */
    data class Understanding(val engine: String) : AiRank

    /** [engine] read the search as [understood], or could not read it when null, and is ranking what that finds. */
    data class Ranking(val engine: String, val understood: AiUnderstood? = null) : AiRank

    /**
     * [engine] ranked [count] of the results first.
     *
     * @property understood What it read the search as, or null.
     * @property answer Its answer when the search asks a question, or null.
     * @property sources The found cards the answer draws from, in its order.
     */
    data class Ranked(
        val engine: String,
        val count: Int,
        val understood: AiUnderstood? = null,
        val answer: String? = null,
        val sources: List<Card> = emptyList(),
    ) : AiRank

    /** The ranking failed, in the server's words. */
    data class Failed(val message: String) : AiRank
}

/** What the engine read the search as, while it ranks and after, or null. */
val AiRank.understood: AiUnderstood?
    get() = when (this) {
        is AiRank.Ranking -> understood
        is AiRank.Ranked -> understood
        else -> null
    }

/**
 * Search text, an optional filter, the suggestion chips and the matching cards. Typing searches by words and meaning
 * at once; pressing search also has the account's Claude Code or Codex read what it means, search for that, rank what
 * it finds and answer a question, when one is signed in.
 *
 * @param scope Scope the searches run in.
 * @param library Source of the results and suggestions.
 * @param ai Ranks a pressed search, or null for none.
 * @param recent Keeps the pressed searches on the phone, or null to keep none.
 */
class SearchModel(
    private val scope: CoroutineScope,
    private val library: Library,
    private val ai: AiRanking? = null,
    private val recent: RecentSearches? = null,
) {
    private val mutableQuery = MutableStateFlow("")
    private val mutableFilter = MutableStateFlow<SearchFilter?>(null)
    private val mutableTypes = MutableStateFlow<List<Suggestion>>(emptyList())
    private val mutableSuggestions = MutableStateFlow<List<Suggestion>>(emptyList())
    private val mutableResults = MutableStateFlow(CardFeed(scope) { ApiResult.Ok(CardPage()) })
    private val mutableRank = MutableStateFlow<AiRank?>(null)
    private val mutableEngine = MutableStateFlow<String?>(null)
    private val mutableOnlyRanked = MutableStateFlow(false)
    private var answered: List<Card> = emptyList()
    private var pending: Job? = null
    private var suggesting: Job? = null
    private var ranking: Job? = null

    /** Text in the search field. */
    val query: StateFlow<String> = mutableQuery

    /** Filter the search is limited to, or null. */
    val filter: StateFlow<SearchFilter?> = mutableFilter

    /** The library's card types, most used first, once [loadTypes] read them. */
    val types: StateFlow<List<Suggestion>> = mutableTypes

    /** Tags that match the typed text; empty while the field is empty. */
    val suggestions: StateFlow<List<Suggestion>> = mutableSuggestions

    /** Cards matching the current text and filter; empty while both are blank. */
    val results: StateFlow<CardFeed> = mutableResults

    /** What the AI is doing with a pressed search, or null when it is not at work on these results. */
    val rank: StateFlow<AiRank?> = mutableRank

    /** The signed-in engine's name, such as "Claude Code", once [loadEngine] or a pressed search found one; else null. */
    val engine: StateFlow<String?> = mutableEngine

    /** True while the results show only the cards the engine ranked first. */
    val onlyRanked: StateFlow<Boolean> = mutableOnlyRanked

    /** The searches last pressed on this phone, newest first. */
    val recentSearches: StateFlow<List<String>> = recent?.searches ?: MutableStateFlow(emptyList())

    /** True when there is something to search for. */
    val active: Boolean get() = mutableQuery.value.isNotBlank() || mutableFilter.value != null

    /**
     * Searches at once, then has the account's Claude Code or Codex read what the search means, search for that, rank
     * what it finds and, for a question, answer it, and shows the cards in that order. A failed reading still ranks
     * the search's own matches. Does nothing more without a signed-in engine, for Same Vibe or for an empty search.
     */
    fun submit() {
        val text = searchText()
        val similar = mutableFilter.value?.similar == true
        if (!similar) recent?.add(mutableQuery.value)
        Track.event("search", words = mutableQuery.value, detail = if (similar) "similar" else "")
        restart(0)
        val ai = ai ?: return
        if (text.isEmpty() || similar) return
        val shown = pending
        ranking = scope.launch {
            val engine = ai.engine()
            mutableEngine.value = engine
            if (engine == null) return@launch
            mutableRank.value = AiRank.Understanding(engine)
            mutableRank.value = try {
                val understood = try {
                    ai.understand(text)
                } catch (e: CancellationException) {
                    throw e
                } catch (_: Exception) {
                    null
                }
                mutableRank.value = AiRank.Ranking(engine, understood)
                val answer = ai.search(text, understood)
                // The search started above shows first; the ranked cards then take its place.
                shown?.join()
                val feed = CardFeed(scope) { cursor -> ApiResult.Ok(if (cursor == null) CardPage(answer.cards) else CardPage()) }
                mutableResults.value = feed
                feed.loadMore()
                answered = answer.cards
                val byId = answer.cards.associateBy { it.id }
                AiRank.Ranked(engine, answer.ranked, answer.understood ?: understood, answer.answer?.text,
                    answer.answer?.ids.orEmpty().mapNotNull(byId::get))
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                AiRank.Failed(e.message ?: "$engine could not rank these. Try again.")
            }
        }
    }

    /**
     * Puts [text] in the field and presses search, as for a recent search tapped again.
     *
     * @param text The search.
     */
    fun searchFor(text: String) {
        mutableQuery.value = text
        suggest("")
        submit()
    }

    /** Asks which engine is signed in, so the page can offer it before a search is pressed. */
    fun loadEngine() {
        val ai = ai ?: return
        scope.launch {
            mutableEngine.value = try {
                ai.engine()
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                null
            }
        }
    }

    /**
     * Shows only the cards the engine ranked first, or every match again. Does nothing until the engine has ranked.
     *
     * @param only True for the ranked cards alone.
     */
    fun showOnlyRanked(only: Boolean) {
        val rank = mutableRank.value as? AiRank.Ranked ?: return
        mutableOnlyRanked.value = only
        val cards = if (only) answered.take(rank.count) else answered
        val feed = CardFeed(scope) { cursor -> ApiResult.Ok(if (cursor == null) CardPage(cards) else CardPage()) }
        mutableResults.value = feed
        feed.loadMore()
    }

    /** Forgets the searches kept on the phone. */
    fun forgetSearches() {
        recent?.clear()
    }

    /** Reads the type suggestions once; later calls keep the first answer. */
    fun loadTypes() {
        if (mutableTypes.value.isNotEmpty()) return
        scope.launch {
            (library.searchSuggestions("") as? ApiResult.Ok)?.value?.filter { it.type == "type" }?.let { mutableTypes.value = it }
        }
    }

    /**
     * Changes the search text, suggests tags after a short pause and searches again after a longer one.
     *
     * @param text New text of the field.
     */
    fun setQuery(text: String) {
        mutableQuery.value = text
        suggest(text)
        restart(DEBOUNCE_MS)
    }

    /**
     * Limits the search to a filter and empties the field, as the web does when a chip is tapped.
     *
     * @param filter Filter to apply, or null to lift it.
     */
    fun setFilter(filter: SearchFilter?) {
        mutableFilter.value = filter
        mutableQuery.value = ""
        suggest("")
        restart(0)
    }

    /**
     * Applies a tapped suggestion: a type limits the search to that type, a tag to cards with that tag.
     *
     * @param suggestion The suggestion.
     */
    fun choose(suggestion: Suggestion) {
        val filter = if (suggestion.type == "type") {
            SearchFilter(suggestion.text, suggestion.queryText)
        } else {
            val name = suggestion.queryText.removePrefix("#").removePrefix("tag:").trim('"')
            SearchFilter(name.lowercase(), "tag:${quoted(name)}")
        }
        setFilter(if (filter == mutableFilter.value) null else filter)
    }

    /**
     * Shows the cards that look like [card], under the Same Vibe filter.
     *
     * @param card Card whose look is searched for.
     */
    fun similarTo(card: Card) = setFilter(SearchFilter(SAME_VIBE_LABEL, "similar:${card.id}"))

    /** Empties the field and the filter. */
    fun clear() {
        mutableQuery.value = ""
        mutableFilter.value = null
        suggest("")
        restart(0)
    }

    private fun suggest(text: String) {
        suggesting?.cancel()
        val trimmed = text.trim()
        if (trimmed.isEmpty()) {
            mutableSuggestions.value = emptyList()
            return
        }
        suggesting = scope.launch {
            delay(SUGGEST_DELAY_MS)
            val found = if (trimmed.startsWith("#")) {
                (library.tagSuggestions(trimmed.removePrefix("#")) as? ApiResult.Ok)?.value?.map { Suggestion("tag", it, "#$it") }
            } else {
                (library.searchSuggestions(trimmed) as? ApiResult.Ok)?.value
            }
            mutableSuggestions.value = found.orEmpty().distinctBy { it.queryText }.take(SUGGESTION_LIMIT)
        }
    }

    private fun searchText(): String {
        val filter = mutableFilter.value
        return if (filter?.similar == true) {
            filter.queryText
        } else {
            listOfNotNull(filter?.queryText, mutableQuery.value.trim().ifEmpty { null }).joinToString(" ")
        }
    }

    private fun restart(wait: Long) {
        pending?.cancel()
        ranking?.cancel()
        mutableRank.value = null
        mutableOnlyRanked.value = false
        answered = emptyList()
        val text = searchText()
        pending = scope.launch {
            if (wait > 0) delay(wait)
            val feed = CardFeed(scope) { cursor ->
                if (text.isEmpty()) ApiResult.Ok(CardPage()) else library.search(text, cursor)
            }
            mutableResults.value = feed
            feed.loadMore()
        }
    }

    private companion object {
        const val DEBOUNCE_MS = 300L
        const val SUGGEST_DELAY_MS = 120L
        const val SUGGESTION_LIMIT = 20
    }
}

// How long pull to refresh waits for a sync before it reads what the phone has.
private const val REFRESH_SYNC_WAIT_MS = 10_000L

// The pause after the lists redraw from the copy, so a burst of changes, such as a sync's pages, redraws them twice at most.
private const val COPY_REDRAW_GAP_MS = 250L

/** How long a deleted card waits for Undo before it goes to the trash. */
const val UNDO_DELETE_MS = 5_000L

/** How many of a space's first cards the space pages show in its mosaic and when held. */
const val SPACE_LATEST_CARDS = 8

/** True when a sync ended having changed the phone's copy; false for a failure, a missing feed or a timeout. */
private fun SyncOutcome?.changedAnything(): Boolean = (this as? SyncOutcome.Done)?.changed == true

/** A tag name as a query value, in quotes when it holds a space or a quote. */
private fun quoted(name: String): String =
    if (name.any { it.isWhitespace() || it == '"' }) "\"" + name.replace("\"", "") + "\"" else name

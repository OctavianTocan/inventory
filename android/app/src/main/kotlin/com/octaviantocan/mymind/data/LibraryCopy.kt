package com.octaviantocan.mymind.data

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteException
import android.database.sqlite.SQLiteOpenHelper
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import java.time.Instant
import java.util.UUID

/** How many of a list's first cards the phone keeps. */
const val FEED_MEMORY_CARDS = 200

/** Cards per page the phone's copy answers with, as the server's gallery does. */
const val LOCAL_PAGE_CARDS = 50

// Card flag bits: pinned, and the private, draft and stub bits that hide a card from the grid.
private const val PINNED_FLAG = 512
private const val GRID_HIDDEN_FLAGS = 8 or 131_072 or 262_144

/** How the copy's gallery cursors start, so a reader can tell them from the server's. */
internal const val LOCAL_CURSOR = "local:"

/** The cards a list showed, kept between launches so it can show them again before the server answers. */
interface FeedMemory {
    /** @return The cards the list showed last, in order; empty when none are kept or they cannot be read. */
    suspend fun read(): List<Card>

    /** @param cards The list's cards now; the first [FEED_MEMORY_CARDS] replace what was kept. */
    suspend fun write(cards: List<Card>)
}

/** What the library screens keep on the phone between launches. */
interface LibraryMemory {
    /**
     * @param name The list, such as `everything`.
     * @return That list's kept cards.
     */
    fun feed(name: String): FeedMemory

    /** @return The page scheme's name as last saved, or null. */
    suspend fun scheme(): String?

    /** @param name The page scheme's name, kept for the next launch. */
    suspend fun saveScheme(name: String)

    /** @return The last Back today and sky answer as JSON, or null when none is kept. */
    suspend fun sky(): String? = null

    /** @param json The latest Back today and sky answer, kept so the band shows at once and offline. */
    suspend fun saveSky(json: String) {}

    /** @return The finds whose stars the person has seen light in the sky, as a JSON array of ids, or null before any. */
    suspend fun skySeen(): String? = null

    /** @param json The finds whose stars the person has seen light, as a JSON array of ids. */
    suspend fun saveSkySeen(json: String) {}
}

/**
 * The phone's copy of the library, in a private SQLite database: the first screens of each list as last shown, and,
 * once the server's changes feed has been read to the end, every card with a keyword index and the spaces. It also
 * holds the [Outbox]'s queue of changes made while the server could not be reached, and shows them on its cards.
 *
 * Everything in it belongs to the session that wrote it: under another session's token it reads as empty and is wiped
 * on the next write, and [clear] runs at sign-out. Failures read as empty or null and writes that fail are dropped, so
 * the copy never stops the app.
 *
 * @param context Any context; the database lives in the app's private storage.
 * @param session The session whose token owns the copy.
 * @param io Dispatcher the database work runs on.
 */
class LibraryCopy(context: Context, private val session: Session, private val io: CoroutineDispatcher = Dispatchers.IO) : LibraryMemory {
    private val helper = Helper(context.applicationContext)
    private val lock = Mutex()
    private val changed = MutableSharedFlow<Unit>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)

    /**
     * One signal after each change to what the copy shows: a sync that brought cards or spaces, a change queued on the
     * phone, or a card read again from the server. A collector still busy gets one for all it missed.
     */
    val changes: SharedFlow<Unit> = changed

    override fun feed(name: String): FeedMemory = object : FeedMemory {
        override suspend fun read(): List<Card> = owned(emptyList()) { db ->
            db.query("feed_cards", arrayOf("card"), "feed = ?", arrayOf(name), null, null, "position").use { rows ->
                buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) }
            }
        }

        override suspend fun write(cards: List<Card>) = owned(Unit) { db ->
            db.delete("feed_cards", "feed = ?", arrayOf(name))
            cards.take(FEED_MEMORY_CARDS).forEachIndexed { position, card ->
                db.insertOrThrow("feed_cards", null, ContentValues().apply {
                    put("feed", name)
                    put("position", position)
                    put("card", encode(card))
                })
            }
        }
    }

    override suspend fun scheme(): String? = owned(null) { db -> meta(db, SCHEME) }

    override suspend fun saveScheme(name: String) = owned(Unit) { db -> setMeta(db, SCHEME, name) }

    override suspend fun sky(): String? = owned(null) { db -> meta(db, SKY) }

    override suspend fun saveSky(json: String) = owned(Unit) { db -> setMeta(db, SKY, json) }

    override suspend fun skySeen(): String? = owned(null) { db -> meta(db, SKY_SEEN) }

    override suspend fun saveSkySeen(json: String) = owned(Unit) { db -> setMeta(db, SKY_SEEN, json) }

    /** @return True once the copy holds the whole library, so the gallery can be read from it. */
    suspend fun complete(): Boolean = owned(false) { db -> meta(db, COMPLETE) == "1" }

    /** @return The changes feed position the copy is up to date with, or null before the first answer. */
    suspend fun syncToken(): String? = owned(null) { db -> meta(db, TOKEN) }

    /**
     * Keeps one answer of the changes feed, all of it or nothing.
     *
     * @param page The answer.
     * @return True when it was kept; false when signed out or the database failed.
     */
    suspend fun apply(page: ChangesPage): Boolean {
        var shows = false
        val kept = owned(false) { db ->
            shows = page.reset || page.cards.isNotEmpty() || page.removed.isNotEmpty() ||
                (page.spaces != null && page.spaces != spacesIn(db)) || (!page.more && meta(db, COMPLETE) != "1")
            keepPage(db, page)
        }
        if (kept && shows) changed.tryEmit(Unit)
        return kept
    }

    // Writes one answer of the feed into the copy, with the changes still waiting shown on top.
    private fun keepPage(db: SQLiteDatabase, page: ChangesPage): Boolean {
        if (page.reset) {
            db.delete("cards", null, null)
            db.delete("card_text", null, null)
            db.delete("meta", "key = ?", arrayOf(COMPLETE))
        }
        page.cards.forEach { card -> keep(db, card) }
        page.removed.forEach { id -> forget(db, id) }
        page.spaces?.let { spaces -> setSpaces(db, spaces) }
        showQueued(db, page.cards.mapTo(HashSet()) { it.id })
        setMeta(db, TOKEN, page.next)
        if (!page.more) setMeta(db, COMPLETE, "1")
        return true
    }

    /**
     * @param cursor `next` of the previous page, or null for the first.
     * @return One page of the gallery from the copy, newest bumped first with the grid's filter, or null when it cannot
     *   be read.
     */
    suspend fun page(cursor: String?): CardPage? = owned(null) { db ->
        val after = cursor?.removePrefix(LOCAL_CURSOR)?.split(':', limit = 2)
        val visible = "(flags & $GRID_HIDDEN_FLAGS) = 0"
        val rows = if (after == null || after.size < 2) {
            db.rawQuery("SELECT id, card, bumped FROM cards WHERE $visible ORDER BY bumped DESC, id DESC LIMIT ${LOCAL_PAGE_CARDS + 1}", null)
        } else {
            db.rawQuery(
                "SELECT id, card, bumped FROM cards WHERE $visible AND (bumped < ? OR (bumped = ? AND id < ?)) ORDER BY bumped DESC, id DESC LIMIT ${LOCAL_PAGE_CARDS + 1}",
                arrayOf(after[0], after[0], after[1]),
            )
        }
        val found = rows.use { buildList { while (it.moveToNext()) add(Triple(it.getString(0), it.getString(1), it.getLong(2))) } }
        val shown = found.take(LOCAL_PAGE_CARDS)
        val next = if (found.size > LOCAL_PAGE_CARDS) shown.last().let { (id, _, bumped) -> "$LOCAL_CURSOR$bumped:$id" } else null
        CardPage(shown.map { decode(it.second) }, next)
    }

    /** @return Pinned from the copy, in pin order, or null when it cannot be read. */
    suspend fun pinned(): CardPage? = owned(null) { db -> CardPage(pinnedIn(db)) }

    /** @return Every card the gallery shows, for counting finds, or null when the copy cannot be read. */
    suspend fun gallery(): List<Card>? = owned(null) { db ->
        db.rawQuery("SELECT card FROM cards WHERE (flags & $GRID_HIDDEN_FLAGS) = 0", null)
            .use { rows -> buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) } }
    }

    /**
     * @param newest How many of the gallery's newest cards to include.
     * @return The cards whose pictures the phone keeps for use offline: Pinned, then the gallery's [newest] cards,
     *   each once; null when the copy cannot be read.
     */
    suspend fun pictureCards(newest: Int): List<Card>? = owned(null) { db ->
        val latest = db.rawQuery(
            "SELECT card FROM cards WHERE (flags & $GRID_HIDDEN_FLAGS) = 0 ORDER BY bumped DESC, id DESC LIMIT ?",
            arrayOf(newest.toString()),
        ).use { rows -> buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) } }
        (pinnedIn(db) + latest).distinctBy { it.id }
    }

    /**
     * Keyword search over the copy: every word of the query starts a word of the card's title, text, link or tags.
     *
     * @param query Search text.
     * @return Matching cards, newest bumped first, or null when it cannot be read.
     */
    suspend fun search(query: String): CardPage? = owned(null) { db ->
        val words = query.lowercase().split(Regex("[^\\p{L}\\p{N}]+")).filter { it.isNotEmpty() }.take(MAX_SEARCH_WORDS)
        if (words.isEmpty()) return@owned CardPage()
        db.rawQuery(
            "SELECT c.card FROM card_text t JOIN cards c ON c.slot = t.docid WHERE card_text MATCH ? AND (c.flags & $GRID_HIDDEN_FLAGS) = 0 " +
                "ORDER BY c.bumped DESC, c.id DESC LIMIT $LOCAL_PAGE_CARDS",
            arrayOf(words.joinToString(" ") { "$it*" }),
        ).use { rows -> CardPage(buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) }) }
    }

    /**
     * @param id Card id.
     * @return The card as the copy holds it, or null.
     */
    suspend fun card(id: String): Card? = owned(null) { db -> cardIn(db, id) }

    /** @return The spaces as the last full answer listed them, with the space changes still queued, or null. */
    suspend fun spaces(): List<Space>? = owned(null) { db -> spacesIn(db) }

    /**
     * @param spaceId Space id.
     * @return The space's cards from the copy in the space's order, or null when it cannot be read. Cards that do not
     *   carry their place in the space ([Card.spaceRanks]) come after the others, newest bumped first.
     */
    suspend fun spaceCards(spaceId: String): CardPage? = owned(null) { db ->
        db.rawQuery(
            "SELECT card FROM cards WHERE spaces LIKE ? AND (flags & $GRID_HIDDEN_FLAGS) = 0 ORDER BY bumped DESC, id DESC",
            arrayOf("%,$spaceId,%"),
        ).use { rows ->
            val cards = buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) }.filter { spaceId in it.spaces.orEmpty() }
            CardPage(cards.sortedBy { it.spaceRanks?.get(spaceId) ?: Double.MAX_VALUE })
        }
    }

    /**
     * Queues a change for the server and shows it on the card at once.
     *
     * @param edit The change; a tag removal without a card is matched to the card that has the tag.
     * @return True when it was queued; false when signed out or the database failed.
     */
    suspend fun queue(edit: Edit): Boolean = owned(false) { db ->
        val kept = if (edit is Edit.RemoveTag && edit.cardId == null) edit.copy(cardId = cardWithTag(db, edit.tagId)?.id) else edit
        db.insertOrThrow("outbox", null, ContentValues().apply { put("edit", AppJson.encodeToString(Edit.serializer(), kept)) })
        show(db, kept)
        true
    }.also { if (it) changed.tryEmit(Unit) }

    /** @return The change that waits longest, with its place in the queue, or null when none waits or it cannot be read. */
    suspend fun firstQueued(): Pair<Long, Edit>? = owned(null) { db -> queuedIn(db).firstOrNull() }

    /**
     * @param limit Most changes to return.
     * @return The changes that wait longest, oldest first, each with its place in the queue; empty when none waits or
     *   the queue cannot be read.
     */
    suspend fun queued(limit: Int): List<Pair<Long, Edit>> = owned(emptyList()) { db -> queuedIn(db).take(limit) }

    /**
     * @param seq A change's place in the queue.
     * @return True once the change is out of the queue; false when the database failed.
     */
    suspend fun unqueue(seq: Long): Boolean = owned(false) { db ->
        db.delete("outbox", "seq = ?", arrayOf(seq.toString()))
        true
    }

    /**
     * @param seq A place in the queue.
     * @return True once the change at [seq] and every change before it are out of the queue; false when the database
     *   failed.
     */
    suspend fun unqueueThrough(seq: Long): Boolean = owned(false) { db ->
        db.delete("outbox", "seq <= ?", arrayOf(seq.toString()))
        true
    }

    /** @return True once the server has answered a push for this copy; false before, or when it cannot be read. */
    suspend fun pushWorks(): Boolean = owned(false) { db -> meta(db, PUSH) == "1" }

    /** Notes that the server answered a push for this copy; the note is wiped with the queue. */
    suspend fun markPushWorks() = owned(Unit) { db -> setMeta(db, PUSH, "1") }

    /**
     * @return The id this copy goes by with the server's push, made on first use and wiped with the queue, so a new
     *   queue starts a new count with the server; null when signed out or the database failed.
     */
    suspend fun clientId(): String? = owned(null) { db -> meta(db, CLIENT) ?: UUID.randomUUID().toString().also { setMeta(db, CLIENT, it) } }

    /**
     * Takes a tag added while offline out of the queue and off its card, before it was sent.
     *
     * @param localId The tag's id on the phone.
     * @return True when its addition was still queued and is now gone.
     */
    suspend fun unqueueTag(localId: String): Boolean = owned(false) { db ->
        val (seq, edit) = queuedIn(db).firstOrNull { (_, edit) -> edit is Edit.AddTag && edit.localId == localId } ?: return@owned false
        db.delete("outbox", "seq = ?", arrayOf(seq.toString()))
        edit.cardId?.let { id -> cardIn(db, id) }?.let { card -> keep(db, card.copy(tags = card.tags.orEmpty().filterNot { it.id == localId })) }
        true
    }.also { if (it) changed.tryEmit(Unit) }

    /**
     * Puts the server's version of one card in the copy, with the changes still queued for it on top.
     *
     * @param id Card id.
     * @param card The card as the server has it; null or trashed takes it out of the copy.
     */
    suspend fun replace(id: String, card: Card?) {
        val kept = owned(false) { db ->
            if (card == null || card.deleted != null) forget(db, id) else keep(db, card)
            showQueued(db, setOf(id))
            true
        }
        if (kept) changed.tryEmit(Unit)
    }

    /** Forgets everything the copy holds. */
    suspend fun clear() {
        lock.withLock {
            withContext(io) {
                runCatching { helper.writableDatabase.run { TABLES.forEach { delete(it, null, null) } } }
            }
        }
    }

    // Runs `work` in one transaction once the copy belongs to the current session; `fallback` when signed out or on failure.
    private suspend fun <T> owned(fallback: T, work: (SQLiteDatabase) -> T): T = lock.withLock {
        withContext(io) {
            val owner = session.token?.let(::accountProfileSessionKey) ?: return@withContext fallback
            runCatching {
                val db = helper.writableDatabase
                db.beginTransaction()
                try {
                    if (meta(db, OWNER) != owner) {
                        TABLES.forEach { db.delete(it, null, null) }
                        setMeta(db, OWNER, owner)
                    }
                    work(db).also { db.setTransactionSuccessful() }
                } finally {
                    db.endTransaction()
                }
            }.getOrDefault(fallback)
        }
    }

    private fun pinnedIn(db: SQLiteDatabase): List<Card> = db.rawQuery(
        "SELECT card FROM cards WHERE (flags & $PINNED_FLAG) != 0 AND (flags & $GRID_HIDDEN_FLAGS) = 0 ORDER BY COALESCE(pin, 2147483647), bumped DESC, id DESC",
        null,
    ).use { rows -> buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) } }

    private fun cardIn(db: SQLiteDatabase, id: String): Card? =
        db.rawQuery("SELECT card FROM cards WHERE id = ?", arrayOf(id)).use { if (it.moveToFirst()) decode(it.getString(0)) else null }

    // The card holding a tag; the text match narrows the rows, decoding confirms it.
    private fun cardWithTag(db: SQLiteDatabase, tagId: String): Card? =
        db.rawQuery("SELECT card FROM cards WHERE card LIKE ?", arrayOf("%$tagId%")).use { rows ->
            generateSequence { if (rows.moveToNext()) decode(rows.getString(0)) else null }.firstOrNull { card -> card.tags.orEmpty().any { it.id == tagId } }
        }

    // Changes still waiting, oldest first; a change this version cannot read is skipped.
    private fun queuedIn(db: SQLiteDatabase): List<Pair<Long, Edit>> =
        db.rawQuery("SELECT seq, edit FROM outbox ORDER BY seq", null).use { rows ->
            buildList {
                while (rows.moveToNext()) {
                    val seq = rows.getLong(0)
                    runCatching { AppJson.decodeFromString(Edit.serializer(), rows.getString(1)) }.getOrNull()?.let { add(seq to it) }
                }
            }
        }

    // Shows one change: on its card, when the copy holds the card, or on the spaces and their cards. A card joining a
    // space goes to its end, as the server puts it.
    private fun show(db: SQLiteDatabase, edit: Edit) {
        when (edit) {
            is Edit.CreateSpace -> spacesIn(db)?.let { spaces ->
                val last = spaces.maxOfOrNull { it.position ?: 0L } ?: 0L
                setSpaces(db, spaces.filterNot { it.id == edit.spaceId } + Space(edit.spaceId, edit.name, edit.color, last + 1, smart = false, count = 0))
            }
            is Edit.DeleteSpace -> {
                spacesIn(db)?.let { spaces -> setSpaces(db, spaces.filterNot { it.id == edit.spaceId }) }
                db.rawQuery("SELECT card FROM cards WHERE spaces LIKE ?", arrayOf("%,${edit.spaceId},%")).use { rows ->
                    buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) }
                }.forEach { card -> card.edited(edit)?.let { keep(db, it) } }
            }
            is Edit.CreateCard -> if (cardIn(db, edit.cardId) == null) keep(db, edit.placeholder())
            is Edit.Visit -> {}
            is Edit.MoveInSpace -> showMove(db, edit)
            else -> showOnCard(db, edit)
        }
    }

    private fun showOnCard(db: SQLiteDatabase, edit: Edit) {
        val id = edit.cardId ?: return
        val card = cardIn(db, id) ?: return
        val edited = card.edited(edit) ?: return forget(db, id)
        val joins = edit is Edit.AddToSpace && edit.spaceId !in card.spaces.orEmpty()
        keep(db, if (joins) edited.copy(spaceRanks = edited.spaceRanks.orEmpty() + (edit.spaceId to nextRank(db, edit.spaceId))) else edited)
    }

    // Moves a card within its space as the server does: the space's cards in their order, the card taken out and put
    // back after the one it follows, then every card numbered again from 0. Nothing moves when either card is not there.
    private fun showMove(db: SQLiteDatabase, edit: Edit.MoveInSpace) {
        val members = db.rawQuery("SELECT card FROM cards WHERE spaces LIKE ?", arrayOf("%,${edit.spaceId},%")).use { rows ->
            buildList { while (rows.moveToNext()) add(decode(rows.getString(0))) }
        }.filter { edit.spaceId in it.spaces.orEmpty() }.sortedBy { it.spaceRanks?.get(edit.spaceId) ?: Double.MAX_VALUE }.toMutableList()
        val from = members.indexOfFirst { it.id == edit.cardId }
        if (from < 0) return
        val moved = members.removeAt(from)
        val to = edit.afterId?.let { after -> members.indexOfFirst { it.id == after }.takeIf { it >= 0 }?.plus(1) ?: return } ?: 0
        members.add(to, moved)
        members.forEachIndexed { rank, card ->
            if (card.spaceRanks?.get(edit.spaceId) != rank.toDouble()) {
                keep(db, card.copy(spaceRanks = card.spaceRanks.orEmpty() + (edit.spaceId to rank.toDouble())))
            }
        }
    }

    // The place after the last card of a space, by the places the copy's cards carry.
    private fun nextRank(db: SQLiteDatabase, spaceId: String): Double =
        db.rawQuery("SELECT card FROM cards WHERE spaces LIKE ?", arrayOf("%,$spaceId,%")).use { rows ->
            generateSequence { if (rows.moveToNext()) decode(rows.getString(0)) else null }.mapNotNull { it.spaceRanks?.get(spaceId) }.maxOrNull()
        }?.plus(1) ?: 0.0

    // Shows the changes still waiting on cards the server's answer just replaced, and the changes that make or delete
    // things again, since the answer may have started the copy over or brought the cards of a deleted space.
    private fun showQueued(db: SQLiteDatabase, ids: Set<String>) {
        queuedIn(db).forEach { (_, edit) ->
            if (edit.cardId in ids || edit is Edit.CreateSpace || edit is Edit.DeleteSpace || edit is Edit.CreateCard ||
                edit is Edit.MoveInSpace) show(db, edit)
        }
    }

    private fun spacesIn(db: SQLiteDatabase): List<Space>? = meta(db, SPACES)?.let { AppJson.decodeFromString(ListSerializer(Space.serializer()), it) }

    private fun setSpaces(db: SQLiteDatabase, spaces: List<Space>) {
        setMeta(db, SPACES, AppJson.encodeToString(ListSerializer(Space.serializer()), spaces))
    }

    // Writes a card and its keyword text, keeping its row number so the text stays joined to it.
    private fun keep(db: SQLiteDatabase, card: Card) {
        val values = ContentValues().apply {
            put("card", encode(card))
            put("bumped", nanosOf(card.bumped))
            put("flags", card.flags)
            put("pin", card.pinPosition)
            put("spaces", card.spaces.orEmpty().joinToString(",", prefix = ",", postfix = ","))
        }
        val row = db.rawQuery("SELECT slot FROM cards WHERE id = ?", arrayOf(card.id)).use { if (it.moveToFirst()) it.getLong(0) else null }
        val id = if (row != null) {
            db.update("cards", values, "slot = ?", arrayOf(row.toString()))
            row
        } else {
            db.insertOrThrow("cards", null, values.apply { put("id", card.id) })
        }
        db.delete("card_text", "docid = ?", arrayOf(id.toString()))
        db.insertOrThrow("card_text", null, ContentValues().apply {
            put("docid", id)
            put("title", listOfNotNull(card.title, card.reddit?.title).joinToString(" "))
            put("body", bodyText(card))
            put("tags", card.tags.orEmpty().mapNotNull { it.name }.joinToString(" "))
        })
    }

    private fun forget(db: SQLiteDatabase, id: String) {
        db.rawQuery("SELECT slot FROM cards WHERE id = ?", arrayOf(id)).use { if (it.moveToFirst()) it.getLong(0) else null }?.let { row ->
            db.delete("card_text", "docid = ?", arrayOf(row.toString()))
            db.delete("cards", "slot = ?", arrayOf(row.toString()))
        }
    }

    private fun meta(db: SQLiteDatabase, key: String): String? =
        db.query("meta", arrayOf("value"), "key = ?", arrayOf(key), null, null, null).use { if (it.moveToFirst()) it.getString(0) else null }

    private fun setMeta(db: SQLiteDatabase, key: String, value: String) {
        db.insertWithOnConflict("meta", null, ContentValues().apply { put("key", key); put("value", value) }, SQLiteDatabase.CONFLICT_REPLACE)
    }

    private class Helper(context: Context) : SQLiteOpenHelper(context, "library-copy.db", null, 3) {
        override fun onCreate(db: SQLiteDatabase) {
            db.execSQL("CREATE TABLE feed_cards (feed TEXT NOT NULL, position INTEGER NOT NULL, card TEXT NOT NULL, PRIMARY KEY (feed, position))")
            db.execSQL("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            db.execSQL("CREATE TABLE cards (slot INTEGER PRIMARY KEY, id TEXT NOT NULL UNIQUE, card TEXT NOT NULL, bumped INTEGER NOT NULL, flags INTEGER NOT NULL, pin INTEGER, spaces TEXT NOT NULL)")
            db.execSQL("CREATE INDEX cards_gallery ON cards (bumped DESC, id DESC)")
            db.execSQL("CREATE TABLE outbox (seq INTEGER PRIMARY KEY AUTOINCREMENT, edit TEXT NOT NULL)")
            // unicode61 folds case and accents beyond ASCII; the simple tokenizer is the fallback where it is missing.
            try {
                db.execSQL("CREATE VIRTUAL TABLE card_text USING fts4(title, body, tags, tokenize=unicode61)")
            } catch (_: SQLiteException) {
                db.execSQL("CREATE VIRTUAL TABLE card_text USING fts4(title, body, tags)")
            }
        }

        // The copy is a cache: a new layout starts empty and fills from the server. Version 3 added the outbox; a later
        // layout must carry its waiting changes over, since the server does not have them.
        override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
            TABLES.forEach { db.execSQL("DROP TABLE IF EXISTS $it") }
            onCreate(db)
        }
    }

    private companion object {
        const val OWNER = "owner"
        const val SCHEME = "scheme"
        const val SKY = "sky"
        const val SKY_SEEN = "sky_seen"
        const val TOKEN = "sync_token"
        const val COMPLETE = "sync_complete"
        const val SPACES = "spaces"
        const val CLIENT = "client_id"
        const val PUSH = "push_works"
        const val MAX_SEARCH_WORDS = 8
        val TABLES = listOf("feed_cards", "meta", "cards", "card_text", "outbox")
    }
}

private fun encode(card: Card): String = AppJson.encodeToString(Card.serializer(), card)

private fun decode(json: String): Card = AppJson.decodeFromString(Card.serializer(), json)

// Nanoseconds since the epoch of an ISO time, so cards sort as the server's gallery does; 0 when it cannot be read.
private fun nanosOf(iso: String): Long = runCatching { Instant.parse(iso).let { it.epochSecond * 1_000_000_000 + it.nano } }.getOrDefault(0)

// The words of a card besides its title and tags: what it says, where it is from and its saved text.
private fun bodyText(card: Card): String = buildList {
    addAll(listOfNotNull(card.description, card.note, card.url, card.domain, card.siteName, card.author, card.product?.brand))
    card.post?.bodyHtml?.let { add(it.replace(Regex("<[^>]+>"), " ")) }
    card.post?.author?.let(::add)
    card.article?.let { addAll(listOfNotNull(it.title, it.preview)) }
    card.reddit?.comments?.forEach { addAll(listOfNotNull(it.author, it.body)) }
    card.prose?.let { collectText(it, this) }
}.joinToString(" ")

private fun collectText(node: JsonElement, into: MutableList<String>) {
    when (node) {
        is JsonObject -> node.forEach { (key, value) ->
            if (key == "text" && value is JsonPrimitive && value.isString) into.add(value.content) else collectText(value, into)
        }
        is JsonArray -> node.forEach { collectText(it, into) }
        else -> Unit
    }
}

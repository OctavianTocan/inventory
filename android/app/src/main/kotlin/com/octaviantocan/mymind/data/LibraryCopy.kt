package com.octaviantocan.mymind.data

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext

/** How many of a list's first cards the phone keeps. */
const val FEED_MEMORY_CARDS = 200

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
}

/**
 * The phone's copy of the library's first screens, in a private SQLite database. Everything in it belongs to the
 * session that wrote it: under another session's token it reads as empty and is wiped on the next write, and
 * [clear] runs at sign-out. Failures read as empty and writes that fail are dropped, so the copy never stops the app.
 *
 * @param context Any context; the database lives in the app's private storage.
 * @param session The session whose token owns the copy.
 * @param io Dispatcher the database work runs on.
 */
class LibraryCopy(context: Context, private val session: Session, private val io: CoroutineDispatcher = Dispatchers.IO) : LibraryMemory {
    private val helper = Helper(context.applicationContext)
    private val lock = Mutex()

    override fun feed(name: String): FeedMemory = object : FeedMemory {
        override suspend fun read(): List<Card> = owned(emptyList()) { db ->
            db.query("feed_cards", arrayOf("card"), "feed = ?", arrayOf(name), null, null, "position").use { rows ->
                buildList { while (rows.moveToNext()) add(AppJson.decodeFromString(Card.serializer(), rows.getString(0))) }
            }
        }

        override suspend fun write(cards: List<Card>) = owned(Unit) { db ->
            db.delete("feed_cards", "feed = ?", arrayOf(name))
            cards.take(FEED_MEMORY_CARDS).forEachIndexed { position, card ->
                db.insertOrThrow("feed_cards", null, ContentValues().apply {
                    put("feed", name)
                    put("position", position)
                    put("card", AppJson.encodeToString(Card.serializer(), card))
                })
            }
        }
    }

    override suspend fun scheme(): String? = owned(null) { db -> meta(db, SCHEME) }

    override suspend fun saveScheme(name: String) = owned(Unit) { db -> setMeta(db, SCHEME, name) }

    /** Forgets everything the copy holds. */
    suspend fun clear() {
        lock.withLock {
            withContext(io) {
                runCatching {
                    helper.writableDatabase.run {
                        delete("feed_cards", null, null)
                        delete("meta", null, null)
                    }
                }
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
                        db.delete("feed_cards", null, null)
                        db.delete("meta", null, null)
                        setMeta(db, OWNER, owner)
                    }
                    work(db).also { db.setTransactionSuccessful() }
                } finally {
                    db.endTransaction()
                }
            }.getOrDefault(fallback)
        }
    }

    private fun meta(db: SQLiteDatabase, key: String): String? =
        db.query("meta", arrayOf("value"), "key = ?", arrayOf(key), null, null, null).use { if (it.moveToFirst()) it.getString(0) else null }

    private fun setMeta(db: SQLiteDatabase, key: String, value: String) {
        db.insertWithOnConflict("meta", null, ContentValues().apply { put("key", key); put("value", value) }, SQLiteDatabase.CONFLICT_REPLACE)
    }

    private class Helper(context: Context) : SQLiteOpenHelper(context, "library-copy.db", null, 1) {
        override fun onCreate(db: SQLiteDatabase) {
            db.execSQL("CREATE TABLE feed_cards (feed TEXT NOT NULL, position INTEGER NOT NULL, card TEXT NOT NULL, PRIMARY KEY (feed, position))")
            db.execSQL("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        }

        // The copy is a cache: a new layout starts empty and fills from the server.
        override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
            db.execSQL("DROP TABLE IF EXISTS feed_cards")
            db.execSQL("DROP TABLE IF EXISTS meta")
            onCreate(db)
        }
    }

    private companion object {
        const val OWNER = "owner"
        const val SCHEME = "scheme"
    }
}

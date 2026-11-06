package com.octaviantocan.mymind.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import java.io.File
import java.util.UUID

/**
 * One thing done on the phone, as the server's `POST /app/v1/events` takes it (`docs/analytics.md`). Empty text and
 * zero numbers mean "not given".
 */
@Serializable
data class AppEvent(
    val name: String,
    val screen: String = "",
    val card: String = "",
    val kind: String = "",
    val words: String = "",
    val detail: String = "",
    val count: Double = 0.0,
    val results: Double = 0.0,
    val ms: Double = 0.0,
    val at: Long = 0,
)

/** One batch of [AppEvent]s sent together, under one [batch] id. */
@Serializable
data class EventBatch(val batch: String, val version: String, val events: List<AppEvent>)

/** The server's answer to a batch: how many it kept, and [stop] while it records nothing. */
@Serializable
data class EventsAnswer(val kept: Int = 0, val stop: Boolean = false)

/**
 * The phone's own usage events, waiting to be sent. They are kept apart from the [Outbox] and its changes: a capped
 * queue in its own file, oldest dropped first, sent in batches of [BATCH] and wiped when the app signs out. Counted
 * gestures (pinches, swipes) become one event per kind each time [fold] runs.
 *
 * @param file Where waiting events are kept between launches.
 * @param clock The time now, in milliseconds since 1970.
 */
class Analytics(private val file: File, private val clock: () -> Long = System::currentTimeMillis) {
    private val lock = Any()
    private val waiting = ArrayDeque<AppEvent>(read())
    private val counts = LinkedHashMap<Triple<String, String, String>, Int>()

    @Volatile
    private var stopped = false

    /** How many events wait to be sent. */
    val size: Int get() = synchronized(lock) { waiting.size }

    /**
     * Keeps [event], stamped with the time now, dropping the oldest past [KEPT].
     *
     * @param event What happened.
     */
    fun record(event: AppEvent) {
        if (stopped) return
        synchronized(lock) {
            waiting.addLast(if (event.at == 0L) event.copy(at = clock()) else event)
            while (waiting.size > KEPT) waiting.removeFirst()
        }
    }

    /**
     * Counts one more [name] on [screen], sent as one event with its count at the next [fold].
     *
     * @param name What happened, for example `pinch`.
     * @param screen Where.
     * @param detail More about it, for example the direction.
     */
    fun count(name: String, screen: String = "", detail: String = "") {
        if (stopped) return
        synchronized(lock) {
            val key = Triple(name, screen, detail)
            counts[key] = (counts[key] ?: 0) + 1
        }
    }

    /** Turns the gestures counted since the last fold into one event each and writes the queue to its file. */
    fun fold() {
        val now = clock()
        synchronized(lock) {
            counts.forEach { (key, n) -> waiting.addLast(AppEvent(key.first, screen = key.second, detail = key.third, count = n.toDouble(), at = now)) }
            counts.clear()
            while (waiting.size > KEPT) waiting.removeFirst()
            write(waiting.toList())
        }
    }

    /**
     * Folds the counts, then sends what waits in batches of [BATCH] through [post]. A batch the server answered is
     * dropped; a failed one stays for the next try. When the server says to stop, everything waiting is dropped and
     * nothing more is kept until the app starts again.
     *
     * @param version The app version sent with each batch.
     * @param post Sends one batch.
     * @return Whether everything that waited was sent.
     */
    suspend fun send(version: String, post: suspend (EventBatch) -> ApiResult<EventsAnswer>): Boolean {
        fold()
        while (true) {
            val events = synchronized(lock) { waiting.take(BATCH) }
            if (events.isEmpty()) return true
            when (val answer = post(EventBatch(UUID.randomUUID().toString(), version, events))) {
                is ApiResult.Ok -> synchronized(lock) {
                    if (answer.value.stop) {
                        stopped = true
                        waiting.clear()
                    } else {
                        repeat(events.size) { waiting.removeFirstOrNull() }
                    }
                    write(waiting.toList())
                }
                is ApiResult.Failed -> return false
            }
        }
    }

    /** Forgets every waiting event and count, as when the app signs out. */
    fun clear() {
        synchronized(lock) {
            waiting.clear()
            counts.clear()
            file.delete()
        }
    }

    private fun read(): List<AppEvent> = runCatching {
        if (file.exists()) AppJson.decodeFromString(LIST, file.readText()) else emptyList()
    }.getOrDefault(emptyList()).takeLast(KEPT)

    private fun write(events: List<AppEvent>) {
        runCatching {
            if (events.isEmpty()) {
                file.delete()
            } else {
                val next = File(file.path + ".next")
                next.writeText(AppJson.encodeToString(LIST, events))
                next.renameTo(file)
            }
        }
    }

    companion object {
        /** Most events kept waiting; the oldest go first. */
        const val KEPT = 2000

        /** Most events in one batch, the server's limit. */
        const val BATCH = 100

        private val LIST = ListSerializer(AppEvent.serializer())
    }
}

/**
 * Where screens record usage events from. It does nothing until the app installs its [Analytics], so tests and
 * previews record nothing.
 */
object Track {
    @Volatile
    var sink: Analytics? = null

    /**
     * Records one event.
     *
     * @param name What happened (`start`, `screen`, `open`, `search`, `action`, `share`, `error`).
     * @param screen Where it happened.
     * @param card The card it concerns.
     * @param kind That card's kind.
     * @param words The words searched for.
     * @param detail More about it.
     * @param results How many results it had.
     */
    fun event(
        name: String,
        screen: String = "",
        card: String = "",
        kind: String = "",
        words: String = "",
        detail: String = "",
        results: Int = 0,
    ) {
        sink?.record(AppEvent(name, screen, card, kind, words, detail, results = results.toDouble()))
    }

    /**
     * Counts one gesture, sent with the others of its kind about once a minute.
     *
     * @param name The gesture (`pinch`, `swipe`).
     * @param screen Where.
     * @param detail More about it.
     */
    fun count(name: String, screen: String = "", detail: String = "") {
        sink?.count(name, screen, detail)
    }
}

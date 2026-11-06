package com.octaviantocan.mymind.ui.library

import androidx.compose.runtime.Stable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.octaviantocan.mymind.data.Card

/**
 * The finds a person has gathered into their backpack to act on together: tag, put in a space. It holds cards, not
 * places, so the choice survives a pinch, a scroll or the gallery loading more, and it carries from one screen to the
 * next until it is emptied.
 */
@Stable
class CardChoice {
    private val chosen = mutableStateListOf<Card>()
    private var sweepBase: List<Card> = emptyList()
    private var sweepFrom: Int = -1

    /** True while a hold-and-slide is gathering finds. */
    var sweeping by mutableStateOf(false)
        private set

    /** The chosen cards, in the order they were gathered. */
    val cards: List<Card> get() = chosen

    /** How many cards are chosen. */
    val count: Int get() = chosen.size

    /** True while anything is chosen, when a tap on a find adds or drops it instead of opening it. */
    val active by derivedStateOf { chosen.isNotEmpty() }

    /**
     * Whether a card is chosen.
     *
     * @param id The card's id.
     * @return True when it is in the backpack.
     */
    fun contains(id: String): Boolean = chosen.any { it.id == id }

    /**
     * Adds a card, or drops it when it is chosen already.
     *
     * @param card The tapped card.
     */
    fun toggle(card: Card) {
        val at = chosen.indexOfFirst { it.id == card.id }
        if (at >= 0) chosen.removeAt(at) else chosen.add(card)
    }

    /**
     * Starts a hold-and-slide from the card at [index] of [cards], the gallery's cards in their order.
     *
     * @param cards The gallery's cards.
     * @param index Where the finger came down.
     */
    fun beginSweep(cards: List<Card>, index: Int) {
        if (index !in cards.indices) return
        sweepBase = chosen.filter { it.id != cards[index].id }
        sweepFrom = index
        sweeping = true
        sweepTo(cards, index)
    }

    /**
     * Chooses every card from where the sweep began to the card at [index], in the gallery's reading order, as a photo
     * gallery does: rows fill as the finger goes down, and sliding back drops them again.
     *
     * @param cards The gallery's cards, in the order the sweep began with.
     * @param index The card under the finger now.
     */
    fun sweepTo(cards: List<Card>, index: Int) {
        if (!sweeping || index !in cards.indices || sweepFrom !in cards.indices) return
        val run = sweepRun(sweepFrom, index).map { cards[it] }
        val runIds = run.mapTo(HashSet()) { it.id }
        val next = sweepBase.filter { it.id !in runIds } + run
        if (next.map { it.id } != chosen.map { it.id }) {
            chosen.clear()
            chosen.addAll(next)
        }
    }

    /** Ends a hold-and-slide; what it gathered stays chosen. */
    fun endSweep() {
        sweeping = false
        sweepBase = emptyList()
        sweepFrom = -1
    }

    /**
     * Puts the latest copies of chosen cards in their place, as after tagging, and drops cards no longer in the library.
     *
     * @param updated Cards as they are now; a chosen card missing from [gone] and from here keeps its copy.
     * @param gone Ids of cards that left the library.
     */
    fun refresh(updated: List<Card> = emptyList(), gone: Set<String> = emptySet()) {
        val byId = updated.associateBy { it.id }
        val next = chosen.filter { it.id !in gone }.map { byId[it.id] ?: it }
        if (next != chosen.toList()) {
            chosen.clear()
            chosen.addAll(next)
        }
    }

    /** Empties the backpack. */
    fun clear() {
        endSweep()
        chosen.clear()
    }
}

/**
 * The indices a sweep covers, from where it began to where the finger is, in either direction.
 *
 * @param from Where the sweep began.
 * @param to Where the finger is.
 * @return Every index between them, both ends included, from [from] towards [to].
 */
internal fun sweepRun(from: Int, to: Int): List<Int> = if (to >= from) (from..to).toList() else (from downTo to).toList()

/** How much of a tag the chosen cards have. */
enum class TagShare { None, Some, All }

/**
 * How many of [cards] carry each tag, by name without regard to case.
 *
 * @param cards The chosen cards.
 * @return Each tag's lower-case name with the number of cards that have it.
 */
internal fun tagCounts(cards: List<Card>): Map<String, Int> {
    val counts = LinkedHashMap<String, Int>()
    cards.forEach { card ->
        card.tags.orEmpty().mapNotNull { it.name?.trim()?.lowercase()?.takeIf(String::isNotEmpty) }.toSet()
            .forEach { counts[it] = (counts[it] ?: 0) + 1 }
    }
    return counts
}

/**
 * Whether none, some or all of [total] cards have a tag that [have] of them carry.
 *
 * @param have Cards with the tag.
 * @param total Cards chosen.
 * @return The share, for the tag's mark.
 */
internal fun tagShare(have: Int, total: Int): TagShare = when {
    have <= 0 -> TagShare.None
    have >= total -> TagShare.All
    else -> TagShare.Some
}

package com.octaviantocan.mymind.data

import java.io.InputStream

/** The library reads and writes the screens use. [Repository] talks to the server; tests use a fake. */
interface Library {
    /** @return The signed-in account, or the failure. */
    suspend fun me(): ApiResult<User>

    /**
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of the library, newest first.
     */
    suspend fun cards(cursor: String?): ApiResult<CardPage>

    /** @return The spaces in the user's order. */
    suspend fun spaces(): ApiResult<SpaceList>

    /**
     * @param spaceId Space to read.
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of the space's cards.
     */
    suspend fun spaceCards(spaceId: String, cursor: String?): ApiResult<CardPage>

    /**
     * @param query Search text.
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of matching cards.
     */
    suspend fun search(query: String, cursor: String?): ApiResult<CardPage>

    /** @return The account settings and feature switches of the web page config. */
    suspend fun pageConfig(): ApiResult<PageConfig>

    /**
     * @param query Typed search text, or empty for the library's card types.
     * @return Card types and tags that match, best first.
     */
    suspend fun searchSuggestions(query: String): ApiResult<List<Suggestion>>

    /**
     * @param query Text a tag name contains.
     * @return Names of matching tags, most used first.
     */
    suspend fun tagSuggestions(query: String): ApiResult<List<String>>

    /** @return Names of every tag the user added, in the server's order. */
    suspend fun tags(): ApiResult<List<String>>

    /** @return Names of the tags added last, newest first. */
    suspend fun recentTags(): ApiResult<List<String>>

    /**
     * @param cardId Card to tag.
     * @param name Tag name.
     * @return The tag as the card now has it.
     */
    suspend fun addTag(cardId: String, name: String): ApiResult<Tag>

    /**
     * @param tagId Id of the card's tag.
     * @return Success, or the failure.
     */
    suspend fun removeTag(tagId: String): ApiResult<Unit>

    /**
     * @param spaceId Space the card joins.
     * @param cardId Card to put into the space.
     * @return Success, or the failure.
     */
    suspend fun addToSpace(spaceId: String, cardId: String): ApiResult<Unit>

    /**
     * @param name Space name.
     * @param color Space colour as a CSS hex colour, such as `#f96` or `#75ffc0`.
     * @return The new space's id.
     */
    suspend fun createSpace(name: String, color: String): ApiResult<String>

    /**
     * @param spaceId Space to delete; its cards stay in the library.
     * @return Success, or the failure.
     */
    suspend fun deleteSpace(spaceId: String): ApiResult<Unit>

    /**
     * @param path The card's `blob.path`.
     * @param sink Receives the file's bytes off the main thread.
     * @return Success once [sink] returned, or the failure.
     */
    suspend fun media(path: String, sink: (InputStream) -> Unit): ApiResult<Unit>

    /**
     * @param cardId Card to change.
     * @param pinned True pins the card to Top of Mind, false takes it off.
     * @return Success, or the failure.
     */
    suspend fun setPinned(cardId: String, pinned: Boolean): ApiResult<Unit>

    /**
     * @param cardId Card to move to the trash.
     * @return Success, or the failure.
     */
    suspend fun trash(cardId: String): ApiResult<Unit>

    /**
     * @param spaceId Space the card leaves.
     * @param cardId Card to take out of the space.
     * @return Success, or the failure.
     */
    suspend fun removeFromSpace(spaceId: String, cardId: String): ApiResult<Unit>

    /** @return Success, or the failure, after telling the server a Serendipity deck was finished. */
    suspend fun completeSerendipity(): ApiResult<Unit>
}

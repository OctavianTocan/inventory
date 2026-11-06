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
     * @param spaceId Space to rename; its colour, place and cards stay.
     * @param name The new name.
     * @return Success, or the failure.
     */
    suspend fun renameSpace(spaceId: String, name: String): ApiResult<Unit>

    /**
     * @param path The card's `blob.path`.
     * @param sink Receives the file's bytes off the main thread.
     * @return Success once [sink] returned, or the failure.
     */
    suspend fun media(path: String, sink: (InputStream) -> Unit): ApiResult<Unit>

    /**
     * @param cardId Card to change.
     * @param pinned True pins the card, false takes it off.
     * @return Success, or the failure.
     */
    suspend fun setPinned(cardId: String, pinned: Boolean): ApiResult<Unit>

    /**
     * @param cardId Card whose picture turns.
     * @param rotation Degrees clockwise: 0, 90, 180 or 270.
     * @return Success, or the failure.
     */
    suspend fun setRotation(cardId: String, rotation: Int): ApiResult<Unit>

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

    /**
     * @param cursor `next` of the previous page, or null for the first page.
     * @return One page of the cards pinned, in pin order.
     */
    suspend fun pinned(cursor: String?): ApiResult<CardPage>

    /**
     * @param cardId Card to read.
     * @return The card as the server has it now.
     */
    suspend fun card(cardId: String): ApiResult<Card>

    /**
     * @param count Number of files, 1 to 100.
     * @return One upload authorization per file.
     */
    suspend fun authorizeUploads(count: Int): ApiResult<List<UploadAuthorization>>

    /**
     * Sends one piece of a file.
     *
     * @param authorization The file's authorization.
     * @param uploadId Id from the first piece's answer, or null for the first piece.
     * @param file The file.
     * @param start Offset of the piece's first byte.
     * @param length Bytes in the piece, at most 32 MiB.
     * @param source Stream at [start]; exactly [length] bytes are read from it and it is left open.
     * @return The server's answer, or the failure.
     */
    suspend fun uploadChunk(
        authorization: UploadAuthorization,
        uploadId: String?,
        file: UploadFile,
        start: Long,
        length: Long,
        source: InputStream,
    ): ApiResult<UploadChunkAnswer>

    /**
     * @param uploadId Blob id of a finished upload.
     * @param spaceId Space the new card joins, or null.
     * @return The new Media card's id.
     */
    suspend fun createMedia(uploadId: String, spaceId: String?): ApiResult<String>

    /**
     * @param content A link or text to save.
     * @param source Where the card came from.
     * @param spaceId Space the new card joins, or null.
     * @return The card's id; a link that is already saved answers the existing card's id.
     */
    suspend fun createFromContent(content: String, source: String, spaceId: String?): ApiResult<String>

    /**
     * Saves a link or text as [createFromContent] does, with words to go with it.
     *
     * @param content A link or text to save.
     * @param source Where the card came from.
     * @param spaceId Space the new card joins, or null.
     * @param note Words that become the card's note unless it already has one, as a link saved before does; or null.
     * @return The card's id; a link that is already saved answers the existing card's id.
     */
    suspend fun createFromContent(content: String, source: String, spaceId: String?, note: String?): ApiResult<String> {
        val saved = createFromContent(content, source, spaceId)
        if (saved is ApiResult.Ok && note != null) {
            val card = (card(saved.value) as? ApiResult.Ok)?.value
            if (card != null && card.note.isNullOrBlank()) setNote(saved.value, note)
        }
        return saved
    }

    /**
     * @param cardId Card id.
     * @param text Note text.
     * @return Success, or the failure.
     */
    suspend fun setNote(cardId: String, text: String): ApiResult<Unit>

    /**
     * @param day The phone's date, `YYYY-MM-DD`.
     * @return Back today and the sky for that day; a library without them fails with a server 404.
     */
    suspend fun sky(day: String): ApiResult<Sky> = ApiResult.Failed(ApiFailure.Server(404))

    /**
     * Records that a find was opened from Back today or the sky, which lights its star.
     *
     * @param cardId The find.
     * @return Success once recorded or queued; a library that keeps no sky does nothing.
     */
    suspend fun visit(cardId: String): ApiResult<Unit> = ApiResult.Ok(Unit)
}

package com.octaviantocan.mymind.data

import android.os.Build
import java.io.InputStream

/** Sign-in and library operations used by the screens. */
class Repository(private val api: MymindApi, private val session: Session, private val profiles: AccountProfileStore? = null) : Library {

    suspend fun signInWithGoogle(
        profile: () -> AccountProfile? = { null },
        requestIdToken: suspend (nonce: String) -> String?,
    ): SignInOutcome {
        val nonce = when (val answer = api.nonce()) {
            is ApiResult.Ok -> answer.value.nonce
            is ApiResult.Failed -> return SignInOutcome.Failed(answer.failure)
        }
        val idToken = requestIdToken(nonce) ?: return SignInOutcome.Cancelled
        return finish(api.signInWithGoogle(idToken, nonce, deviceLabel()), profile)
    }

    suspend fun signInWithPassword(username: String, password: String): SignInOutcome =
        finish(api.signInWithPassword(username, password, deviceLabel()))

    suspend fun signOut() {
        try {
            api.signOut()
        } finally {
            runCatching { profiles?.clear() }
            session.clear()
        }
    }

    override suspend fun me(): ApiResult<User> {
        val requestedSession = session.token
        val answer = api.me()
        if (session.token != requestedSession) {
            if (session.token == null) runCatching { profiles?.clear() }
            return ApiResult.Failed(ApiFailure.Unauthorized)
        }
        return when (answer) {
            is ApiResult.Ok -> {
                val profile = requestedSession?.let { token ->
                    runCatching { profiles?.read(accountProfileSessionKey(token))?.forVerifiedUser(answer.value) }.getOrNull()
                }
                ApiResult.Ok(answer.value.copy(photoUrl = profile?.photoUrl, displayName = profile?.displayName))
            }
            is ApiResult.Failed -> {
                if (answer.failure == ApiFailure.Unauthorized || answer.failure == ApiFailure.InvalidToken) runCatching { profiles?.clear() }
                answer
            }
        }
    }

    /** @return The AI apps connected to the account. */
    suspend fun agents(): ApiResult<AgentList> = api.agents()

    /** Signs the connected AI app [id] out. */
    suspend fun removeAgent(id: String): ApiResult<Unit> = api.removeAgent(id)

    /**
     * Reads no network, so the account shows at once and offline.
     *
     * @return The Google photo, name and email kept for this session since its sign-in, or null when none is kept.
     */
    fun savedAccount(): AccountProfile? {
        val token = session.token ?: return null
        return runCatching { profiles?.read(accountProfileSessionKey(token))?.forDisplay() }.getOrNull()
    }

    override suspend fun cards(cursor: String?): ApiResult<CardPage> = api.cards(PAGE_SIZE, cursor)

    override suspend fun spaces(): ApiResult<SpaceList> = api.spaces()

    override suspend fun spaceCards(spaceId: String, cursor: String?): ApiResult<CardPage> =
        api.spaceCards(spaceId, PAGE_SIZE, cursor)

    override suspend fun search(query: String, cursor: String?): ApiResult<CardPage> = api.search(query, PAGE_SIZE, cursor)

    override suspend fun pageConfig(): ApiResult<PageConfig> = api.pageConfig()

    override suspend fun sky(day: String): ApiResult<Sky> = api.sky(day)

    /**
     * @param since `next` of the previous answer, or null to read the whole library.
     * @return One answer of the changes feed.
     */
    suspend fun changes(since: String?): ApiResult<ChangesPage> = api.changes(since, CHANGES_SIZE)

    /**
     * @param request The phone's id and its waiting changes, oldest first.
     * @return What the server handled, or the failure.
     */
    suspend fun push(request: PushRequest): ApiResult<PushAnswer> = api.push(request)

    /** Sends one batch of the phone's own usage events. */
    suspend fun events(batch: EventBatch): ApiResult<EventsAnswer> = api.events(batch)

    override suspend fun searchSuggestions(query: String): ApiResult<List<Suggestion>> = api.searchSuggestions(query)

    override suspend fun tagSuggestions(query: String): ApiResult<List<String>> = api.tagSuggestions(query).map { tags -> tags.map { it.name } }

    override suspend fun tags(): ApiResult<List<String>> = api.tags().map { list -> list.tags.map { it.name } }

    override suspend fun recentTags(): ApiResult<List<String>> = api.recentTags().map { tags -> tags.map { it.name } }

    override suspend fun addTag(cardId: String, name: String): ApiResult<Tag> = api.addTag(cardId, name)

    override suspend fun removeTag(tagId: String): ApiResult<Unit> = api.removeTag(tagId)

    override suspend fun addToSpace(spaceId: String, cardId: String): ApiResult<Unit> = api.addToSpace(spaceId, cardId)

    override suspend fun createSpace(name: String, color: String): ApiResult<String> = api.createSpace(name, color)

    override suspend fun deleteSpace(spaceId: String): ApiResult<Unit> = api.deleteSpace(spaceId)

    override suspend fun media(path: String, sink: (InputStream) -> Unit): ApiResult<Unit> = api.media(path, sink)

    override suspend fun setPinned(cardId: String, pinned: Boolean): ApiResult<Unit> = api.setPinned(cardId, pinned)

    override suspend fun trash(cardId: String): ApiResult<Unit> = api.trash(cardId)

    override suspend fun removeFromSpace(spaceId: String, cardId: String): ApiResult<Unit> =
        api.removeFromSpace(spaceId, cardId)

    override suspend fun pinned(cursor: String?): ApiResult<CardPage> = api.pinned(PAGE_SIZE, cursor)

    override suspend fun card(cardId: String): ApiResult<Card> = api.card(cardId)

    override suspend fun authorizeUploads(count: Int): ApiResult<List<UploadAuthorization>> = api.authorizeUploads(count)

    override suspend fun uploadChunk(
        authorization: UploadAuthorization,
        uploadId: String?,
        file: UploadFile,
        start: Long,
        length: Long,
        source: InputStream,
    ): ApiResult<UploadChunkAnswer> = api.uploadChunk(authorization, uploadId, file.name, file.mime, file.size, start, length, source)

    override suspend fun createMedia(uploadId: String, spaceId: String?): ApiResult<String> = api.createMedia(uploadId, spaceId)

    override suspend fun createFromContent(content: String, source: String, spaceId: String?): ApiResult<String> =
        api.createFromContent(content, source, spaceId)

    override suspend fun setNote(cardId: String, text: String): ApiResult<Unit> = api.setNote(cardId, text)

    private suspend fun finish(answer: ApiResult<TokenAnswer>, profile: () -> AccountProfile? = { null }): SignInOutcome = when (answer) {
        is ApiResult.Ok -> {
            // Cosmetic metadata is accepted only after server verification, and ready before the signed-in UI appears.
            runCatching {
                profiles?.write(accountProfileSessionKey(answer.value.token), profile()?.forVerifiedUser(answer.value.user))
            }
            session.signIn(answer.value.token)
            SignInOutcome.Success
        }
        is ApiResult.Failed -> SignInOutcome.Failed(answer.failure)
    }

    private fun deviceLabel() = "${Build.MANUFACTURER} ${Build.MODEL}".trim()

    private companion object {
        const val PAGE_SIZE = 50
        const val CHANGES_SIZE = 100
    }
}

sealed interface SignInOutcome {
    data object Success : SignInOutcome
    data object Cancelled : SignInOutcome
    data class Failed(val failure: ApiFailure) : SignInOutcome
}

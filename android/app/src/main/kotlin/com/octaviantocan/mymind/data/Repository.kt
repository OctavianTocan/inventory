package com.octaviantocan.mymind.data

import android.os.Build
import java.io.InputStream

/** Sign-in and library operations used by the screens. */
class Repository(private val api: MymindApi, private val session: Session) : Library {

    suspend fun signInWithGoogle(requestIdToken: suspend (nonce: String) -> String?): SignInOutcome {
        val nonce = when (val answer = api.nonce()) {
            is ApiResult.Ok -> answer.value.nonce
            is ApiResult.Failed -> return SignInOutcome.Failed(answer.failure)
        }
        val idToken = requestIdToken(nonce) ?: return SignInOutcome.Cancelled
        return finish(api.signInWithGoogle(idToken, nonce, deviceLabel()))
    }

    suspend fun signInWithPassword(username: String, password: String): SignInOutcome =
        finish(api.signInWithPassword(username, password, deviceLabel()))

    suspend fun signOut() {
        api.signOut()
        session.clear()
    }

    override suspend fun me(): ApiResult<User> = api.me()

    override suspend fun cards(cursor: String?): ApiResult<CardPage> = api.cards(PAGE_SIZE, cursor)

    override suspend fun spaces(): ApiResult<SpaceList> = api.spaces()

    override suspend fun spaceCards(spaceId: String, cursor: String?): ApiResult<CardPage> =
        api.spaceCards(spaceId, PAGE_SIZE, cursor)

    override suspend fun search(query: String, cursor: String?): ApiResult<CardPage> = api.search(query, PAGE_SIZE, cursor)

    override suspend fun pageConfig(): ApiResult<PageConfig> = api.pageConfig()

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

    override suspend fun completeSerendipity(): ApiResult<Unit> = api.completeSerendipity()

    private suspend fun finish(answer: ApiResult<TokenAnswer>): SignInOutcome = when (answer) {
        is ApiResult.Ok -> {
            session.signIn(answer.value.token)
            SignInOutcome.Success
        }
        is ApiResult.Failed -> SignInOutcome.Failed(answer.failure)
    }

    private fun deviceLabel() = "${Build.MANUFACTURER} ${Build.MODEL}".trim()

    private companion object {
        const val PAGE_SIZE = 50
    }
}

sealed interface SignInOutcome {
    data object Success : SignInOutcome
    data object Cancelled : SignInOutcome
    data class Failed(val failure: ApiFailure) : SignInOutcome
}

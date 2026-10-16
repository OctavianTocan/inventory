package com.octaviantocan.mymind.data

import android.os.Build

/** Sign-in and library operations used by the screens. */
class Repository(private val api: MymindApi, private val session: Session) {

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

    suspend fun cards(cursor: String?): ApiResult<CardPage> = api.cards(PAGE_SIZE, cursor)

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

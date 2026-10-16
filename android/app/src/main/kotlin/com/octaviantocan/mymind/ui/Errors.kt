package com.octaviantocan.mymind.ui

import com.octaviantocan.mymind.data.ApiFailure

/** Plain-words description of [failure] for the sign-in screen. */
fun describe(failure: ApiFailure): String = when (failure) {
    ApiFailure.NotAllowed -> "This Google account is not allowed to use this library."
    ApiFailure.Invalid -> "Wrong username or password."
    ApiFailure.Locked -> "Too many attempts. Try again later."
    ApiFailure.InvalidToken -> "Google sign-in could not be verified. Try again."
    ApiFailure.Unauthorized -> "Your session ended. Sign in again."
    ApiFailure.Network -> "Cannot reach the server. Check your connection."
    is ApiFailure.Server -> "The server returned an error (${failure.code})."
}

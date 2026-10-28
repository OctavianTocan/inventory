package com.octaviantocan.mymind.data

import android.content.Context
import java.net.URI
import java.security.MessageDigest
import java.util.Locale

/** Display metadata supplied by Google; it never establishes an authenticated account. */
data class AccountProfile(val email: String, val photoUrl: String? = null, val displayName: String? = null)

/** A cosmetic cache bound to an accepted session's digest, never to the bearer token itself. */
interface AccountProfileStore {
    fun read(sessionKey: String): AccountProfile?
    fun write(sessionKey: String, profile: AccountProfile?)
    fun clear()
}

/** One private, replaceable profile. A different session cannot read the previous profile. */
class PrivateAccountProfileStore(context: Context) : AccountProfileStore {
    private val preferences = context.applicationContext.getSharedPreferences("account-profile", Context.MODE_PRIVATE)

    override fun read(sessionKey: String): AccountProfile? {
        if (preferences.getString("session", null) != sessionKey) return null
        val email = preferences.getString("email", null)?.takeIf { it.isNotBlank() } ?: return null
        return AccountProfile(email, preferences.getString("photo", null), preferences.getString("name", null))
    }

    override fun write(sessionKey: String, profile: AccountProfile?) {
        val edit = preferences.edit().clear()
        if (profile != null) {
            edit.putString("session", sessionKey).putString("email", profile.email)
                .putString("photo", profile.photoUrl).putString("name", profile.displayName)
        }
        edit.apply()
    }

    override fun clear() { preferences.edit().clear().apply() }
}

/** The digest only prevents stale cosmetic metadata from crossing sessions; it is never an authentication key. */
internal fun accountProfileSessionKey(token: String): String = MessageDigest.getInstance("SHA-256")
    .digest(token.toByteArray(Charsets.UTF_8))
    .joinToString("") { (it.toInt() and 0xff).toString(16).padStart(2, '0') }

/** Only a server-accepted Google account may use matching credential metadata. */
internal fun AccountProfile.forVerifiedUser(user: User?): AccountProfile? {
    if (user == null) return null
    val verifiedEmail = user.email?.trim()?.takeIf { it.isNotBlank() } ?: return null
    if (user.kind?.equals("google", ignoreCase = true) != true || !email.trim().equals(verifiedEmail, ignoreCase = true)) return null
    return copy(email = verifiedEmail.lowercase(Locale.ROOT)).forDisplay()
}

/** The profile as the screens may show it: only a safe Google photo, at [ACCOUNT_PHOTO_PX], and a plain name. */
internal fun AccountProfile.forDisplay(): AccountProfile = copy(
    photoUrl = safeGooglePhoto(photoUrl)?.replace(GOOGLE_PHOTO_SIZE, "=s$ACCOUNT_PHOTO_PX-c"),
    displayName = displayName?.filterNot { it.isISOControl() }?.trim()?.take(120)?.takeIf { it.isNotEmpty() },
)

/** Pixels of the account photo asked of Google: the 52 dp card photo at the densest screens. */
internal const val ACCOUNT_PHOTO_PX = 256

// Google's sign-in hands out a 96-pixel photo (`…=s96-c`), blurred once the account button and card scale it up.
private val GOOGLE_PHOTO_SIZE = Regex("=s\\d+(-c)?$")

/** Load accepted URLs with a public image loader, never the API client that attaches the session bearer. */
private fun safeGooglePhoto(value: String?): String? {
    val url = value?.trim()?.takeIf { it.isNotEmpty() && it.length <= 2048 } ?: return null
    val uri = runCatching { URI(url) }.getOrNull() ?: return null
    val host = uri.host?.lowercase(Locale.ROOT) ?: return null
    return url.takeIf {
        uri.scheme.equals("https", ignoreCase = true) && uri.userInfo == null && (uri.port == -1 || uri.port == 443) &&
            (host == "googleusercontent.com" || host.endsWith(".googleusercontent.com"))
    }
}

package com.octaviantocan.mymind

import com.octaviantocan.mymind.data.AccountProfile
import com.octaviantocan.mymind.data.AccountProfileStore
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.AuthInterceptor
import com.octaviantocan.mymind.data.MymindApi
import com.octaviantocan.mymind.data.PrivateAccountProfileStore
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.Session
import com.octaviantocan.mymind.data.SessionState
import com.octaviantocan.mymind.data.SignInOutcome
import com.octaviantocan.mymind.data.TokenStore
import com.octaviantocan.mymind.data.User
import com.octaviantocan.mymind.data.accountProfileSessionKey
import com.octaviantocan.mymind.data.forVerifiedUser
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment
import org.robolectric.annotation.Config
import java.io.IOException

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class AccountProfileTest {
    private val server = MockWebServer()
    private val profiles = MemoryProfiles()
    private val tokens = ProfileTokens()
    private lateinit var session: Session
    private lateinit var repository: Repository
    private val candidate = AccountProfile("person@example.com", "https://lh3.googleusercontent.com/example", "A Person")

    @Before
    fun setUp() {
        server.start()
        session = Session(tokens)
        val client = OkHttpClient.Builder().addInterceptor(AuthInterceptor(session, server.url("/").toString())).build()
        repository = Repository(MymindApi(client, server.url("/").toString()), session, profiles)
    }

    @After
    fun tearDown() = server.shutdown()

    private fun answer(body: String, code: Int = 200) = server.enqueue(MockResponse().setResponseCode(code).setBody(body))

    private fun googleAnswers(token: String = "accepted-session-a", email: String = "person@example.com") {
        answer("""{"nonce":"test-nonce"}""")
        answer("""{"token":"$token","user":{"email":"$email","kind":"google"}}""")
    }

    private suspend fun signIn(profile: AccountProfile? = candidate): SignInOutcome =
        repository.signInWithGoogle(profile = { profile }) { "test-google-credential" }

    private suspend fun me(email: String = "person@example.com", kind: String = "google"): User {
        answer("""{"email":"$email","kind":"$kind"}""")
        return (repository.me() as ApiResult.Ok).value
    }

    @Test
    fun acceptedGoogleMetadataIsStoredBeforeTheSignedInStateAndSurvivesRestore() = runBlocking {
        googleAnswers(email = "Person@Example.com")
        tokens.onWrite = { token ->
            assertEquals(candidate, profiles.read(accountProfileSessionKey(token)))
            assertNotEquals(SessionState.SignedIn, session.state.value)
        }
        assertEquals(SignInOutcome.Success, signIn())
        assertEquals(SessionState.SignedIn, session.state.value)
        assertEquals(candidate.photoUrl, me().photoUrl)

        val restored = Session(tokens)
        restored.restore()
        val repo = Repository(MymindApi(OkHttpClient(), server.url("/").toString()), restored, profiles)
        answer("""{"email":"PERSON@example.com","kind":"google"}""")
        assertEquals(candidate.displayName, (repo.me() as ApiResult.Ok).value.displayName)
    }

    @Test
    fun refusedGoogleCredentialsNeverReadOrPersistDisplayMetadata() = runBlocking {
        answer("""{"nonce":"test-nonce"}""")
        answer("""{"error":"not-allowed"}""", 403)
        var readProfile = false
        val outcome = repository.signInWithGoogle(profile = { readProfile = true; candidate }) { "refused-test-credential" }
        assertTrue(outcome is SignInOutcome.Failed)
        assertFalse(readProfile)
        assertEquals(0, profiles.writes)
        assertNull(session.token)
    }

    @Test
    fun metadataForAnotherEmailIsDiscardedAfterSuccessfulLogin() = runBlocking {
        googleAnswers(email = "other@example.com")
        assertEquals(SignInOutcome.Success, signIn())
        assertNull(profiles.record)
        assertNull(me(email = "other@example.com").photoUrl)
    }

    @Test
    fun onlyHttpsGooglePhotosWithoutEmbeddedCredentialsAreAccepted() {
        val user = User(email = candidate.email, kind = "google")
        val rejected = listOf(
            "http://lh3.googleusercontent.com/example",
            "https://example.com/photo",
            "https://googleusercontent.com.example.com/photo",
            "https://user:pass@lh3.googleusercontent.com/photo",
            "https://lh3.googleusercontent.com:444/photo",
            "javascript:photo",
            "https://lh3.googleusercontent.com/a photo",
            "",
        )
        rejected.forEach { url -> assertNull(url, candidate.copy(photoUrl = url).forVerifiedUser(user)?.photoUrl) }
        assertEquals(candidate, candidate.forVerifiedUser(user))
        assertNull(candidate.forVerifiedUser(user.copy(kind = "password")))
        assertNull(candidate.forVerifiedUser(user.copy(email = null)))
    }

    @Test
    fun aPasswordLoginCannotInheritTheSameEmailsGooglePhoto() = runBlocking {
        googleAnswers()
        signIn()
        answer("""{"token":"accepted-password-session","user":{"email":"person@example.com","kind":"password"}}""")
        assertEquals(SignInOutcome.Success, repository.signInWithPassword("person@example.com", "test-password"))
        assertNull(profiles.record)
        assertNull(me(kind = "password").photoUrl)
    }

    @Test
    fun aCosmeticCacheWriteFailureDoesNotBlockLoginOrReuseAnOlderPhoto() = runBlocking {
        profiles.write(accountProfileSessionKey("old-session"), candidate)
        profiles.failWrite = true
        googleAnswers()
        assertEquals(SignInOutcome.Success, signIn(candidate.copy(photoUrl = null)))
        assertEquals(SessionState.SignedIn, session.state.value)
        assertNull(me().photoUrl)
    }

    @Test
    fun aMatchingSessionDigestStillCannotShowAnotherAccountsMetadata() = runBlocking {
        googleAnswers()
        signIn()
        profiles.write(accountProfileSessionKey(requireNotNull(session.token)), candidate.copy(email = "another@example.com"))
        val user = me()
        assertNull(user.photoUrl)
        assertNull(user.displayName)
    }

    @Test
    fun signOutClearsMetadataAndSessionEvenIfTheServerReturnsAnError() = runBlocking {
        googleAnswers()
        signIn()
        answer("", 500)
        repository.signOut()
        assertNull(profiles.record)
        assertNull(session.token)
        assertEquals(SessionState.SignedOut, session.state.value)
    }

    @Test
    fun rejectedMeDoesNotReturnAnOlderProfileAndClearsItsCache() = runBlocking {
        googleAnswers()
        signIn()
        answer("""{"error":"unauthorized"}""", 401)
        assertEquals(ApiResult.Failed(ApiFailure.Unauthorized), repository.me())
        assertNull(profiles.record)
        assertNull(session.token)
    }

    @Test
    fun privatePreferencesKeepOnlyOneSessionBoundProfileAndCanBeCleared() {
        val context = RuntimeEnvironment.getApplication()
        val store = PrivateAccountProfileStore(context)
        store.clear()
        val key = accountProfileSessionKey("test-session")
        assertEquals(64, key.length)
        assertFalse(key.contains("test-session"))
        store.write(key, candidate)
        assertEquals(candidate, PrivateAccountProfileStore(context).read(key))
        assertNull(store.read(accountProfileSessionKey("different-session")))
        store.write(key, null)
        assertNull(store.read(key))
        store.write(key, candidate)
        store.clear()
        assertNull(store.read(key))
    }
}

private class ProfileTokens : TokenStore {
    var token: String? = null
    var onWrite: (String) -> Unit = {}
    override suspend fun read() = token
    override suspend fun write(token: String) { onWrite(token); this.token = token }
    override suspend fun clear() { token = null }
}

private class MemoryProfiles : AccountProfileStore {
    var record: Pair<String, AccountProfile>? = null
    var writes = 0
    var failWrite = false
    override fun read(sessionKey: String) = record?.takeIf { it.first == sessionKey }?.second
    override fun write(sessionKey: String, profile: AccountProfile?) {
        writes++
        if (failWrite) throw IOException("Test cache failure")
        record = profile?.let { sessionKey to it }
    }
    override fun clear() { record = null }
}

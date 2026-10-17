package com.octaviantocan.mymind.update

import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okio.Buffer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import org.junit.runner.RunWith
import java.io.File
import java.io.IOException
import java.security.MessageDigest

@RunWith(AndroidJUnit4::class)
class UpdaterTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val server = MockWebServer()
    private lateinit var prefs: UpdatePrefs
    private lateinit var directory: File
    private val installer = RecordingInstaller()
    private var now = 10_000_000L
    private val apk = ByteArray(5000) { (it % 251).toByte() }

    private class RecordingInstaller(var failure: IOException? = null) : ApkInstaller {
        val installed = mutableListOf<File>()

        override suspend fun install(file: File) {
            failure?.let { throw it }
            installed += file
        }
    }

    @Before
    fun setUp() {
        server.start()
        prefs = UpdatePrefs(ApplicationProvider.getApplicationContext<Context>().getSharedPreferences("updater-test", Context.MODE_PRIVATE).also { it.edit().clear().commit() })
        directory = File(folder.newFolder(), "updates")
    }

    @After
    fun tearDown() = server.shutdown()

    private fun updater(installed: Int = 2) =
        Updater(OkHttpClient(), server.url("/").toString(), installed, directory, installer, prefs, { now }, Dispatchers.Unconfined)

    private fun sha(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }

    private fun json(code: Int = 3, bytes: Long = apk.size.toLong(), sha: String = sha(apk)) =
        """{"versionName":"0.2.1","versionCode":$code,"url":"/download/mymind-0.2.1.apk","bytes":$bytes,"sha256":"$sha","notes":["One","Two"]}"""

    private fun enqueueRelease(json: String = json(), body: ByteArray = apk) {
        server.enqueue(MockResponse().setBody(json))
        server.enqueue(MockResponse().setBody(Buffer().write(body)))
    }

    @Test
    fun newerReleaseIsDownloadedVerifiedAndReady() = runBlocking {
        enqueueRelease()
        val state = updater().check()
        assertTrue(state is UpdateState.Ready)
        val file = (state as UpdateState.Ready).file
        assertEquals("mymind-3.apk", file.name)
        assertTrue(file.readBytes().contentEquals(apk))
        assertFalse(File(directory, "mymind-3.apk.part").exists())
        assertEquals(listOf("One", "Two"), state.release.notes)
    }

    @Test
    fun sameOrOlderReleaseIsUpToDate() = runBlocking {
        server.enqueue(MockResponse().setBody(json(code = 2)))
        assertEquals(UpdateState.UpToDate, updater(installed = 2).check())
        assertEquals(1, server.requestCount)
    }

    @Test
    fun missingReleaseIsUpToDate() = runBlocking {
        server.enqueue(MockResponse().setResponseCode(404))
        assertEquals(UpdateState.UpToDate, updater().check())
    }

    @Test
    fun sizeMismatchFailsAndLeavesNoFile() = runBlocking {
        enqueueRelease(json(bytes = apk.size + 1L))
        val state = updater().check()
        assertTrue(state is UpdateState.Failed)
        assertEquals("The downloaded package failed its size check", (state as UpdateState.Failed).reason)
        assertEquals(emptyList<String>(), directory.list()?.toList().orEmpty())
    }

    @Test
    fun checksumMismatchFailsAndLeavesNoFile() = runBlocking {
        enqueueRelease(json(sha = "0".repeat(64)))
        val state = updater().check()
        assertEquals("The downloaded package failed its checksum check", (state as UpdateState.Failed).reason)
        assertEquals(emptyList<String>(), directory.list()?.toList().orEmpty())
    }

    @Test
    fun failedDownloadIsRetriedAtNextCheck() = runBlocking {
        val updater = updater()
        enqueueRelease(json(sha = "0".repeat(64)))
        assertTrue(updater.check() is UpdateState.Failed)
        enqueueRelease()
        assertTrue(updater.check() is UpdateState.Ready)
    }

    @Test
    fun olderFilesAreDeleted() = runBlocking {
        directory.mkdirs()
        File(directory, "mymind-2.apk").writeText("old")
        enqueueRelease()
        updater().check()
        assertEquals(listOf("mymind-3.apk"), directory.list()?.toList())
    }

    @Test
    fun signedOutServerIsReportedAsFailure() = runBlocking {
        server.enqueue(MockResponse().setResponseCode(401))
        val state = updater().check()
        assertEquals("Signed out of the downloads server", (state as UpdateState.Failed).reason)
    }

    @Test
    fun checkIfStaleWaitsAnHour() = runBlocking {
        val updater = updater()
        server.enqueue(MockResponse().setResponseCode(404))
        updater.checkIfStale()
        now += 59 * 60 * 1000L
        updater.checkIfStale()
        assertEquals(1, server.requestCount)
        now += 2 * 60 * 1000L
        server.enqueue(MockResponse().setResponseCode(404))
        updater.checkIfStale()
        assertEquals(2, server.requestCount)
    }

    @Test
    fun installHandsTheVerifiedFileToTheInstaller() = runBlocking {
        enqueueRelease()
        val updater = updater()
        updater.install()
        assertTrue(installer.installed.isEmpty())
        updater.check()
        updater.install()
        assertEquals(listOf("mymind-3.apk"), installer.installed.map { it.name })
        assertTrue(updater.state.value is UpdateState.Installing)
    }

    @Test
    fun installerFailureIsKeptWithItsReason() = runBlocking {
        enqueueRelease()
        installer.failure = IOException("disk full")
        val updater = updater()
        updater.check()
        updater.install()
        assertEquals("Could not start the install: disk full", (updater.state.value as UpdateState.Failed).reason)
    }

    @Test
    fun pendingUserActionOpensThePromptInTheForeground() = runBlocking {
        enqueueRelease()
        val updater = updater().also { it.foreground = true }
        updater.check()
        updater.install()
        val action = updater.onInstallStatus(PackageInstaller.STATUS_PENDING_USER_ACTION, null, Intent("prompt"))
        assertTrue(action is StatusAction.OpenPrompt)
    }

    @Test
    fun pendingUserActionNotifiesInTheBackground() = runBlocking {
        enqueueRelease()
        val updater = updater().also { it.foreground = false }
        updater.check()
        updater.install()
        val action = updater.onInstallStatus(PackageInstaller.STATUS_PENDING_USER_ACTION, null, Intent("prompt"))
        assertTrue(action is StatusAction.NotifyPrompt)
    }

    @Test
    fun pendingUserActionWithoutPromptFails() = runBlocking {
        val updater = updater()
        assertEquals(StatusAction.None, updater.onInstallStatus(PackageInstaller.STATUS_PENDING_USER_ACTION, null, null))
        assertTrue(updater.state.value is UpdateState.Failed)
    }

    @Test
    fun failureStatusIsKeptWithWording() = runBlocking {
        enqueueRelease()
        val updater = updater()
        updater.check()
        updater.install()
        updater.onInstallStatus(PackageInstaller.STATUS_FAILURE_STORAGE, "x", null)
        assertEquals("Not enough storage to install", (updater.state.value as UpdateState.Failed).reason)
        assertEquals("3", (updater.state.value as UpdateState.Failed).release?.versionCode?.toString())
    }

    @Test
    fun successStatusChangesNothing() = runBlocking {
        val updater = updater()
        assertEquals(StatusAction.None, updater.onInstallStatus(PackageInstaller.STATUS_SUCCESS, null, null))
        assertEquals(UpdateState.None, updater.state.value)
    }

    @Test
    fun versionComparisonUsesTheCode() {
        assertTrue(isNewer(3, 2))
        assertFalse(isNewer(2, 2))
        assertFalse(isNewer(1, 2))
        assertTrue(isNewer(10, 9))
    }
}

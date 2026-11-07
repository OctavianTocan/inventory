package com.octaviantocan.mymind.update

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
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
import java.security.MessageDigest

@RunWith(AndroidJUnit4::class)
class UpdateInstallerTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val server = MockWebServer()
    private val apk = ByteArray(2000) { (it % 251).toByte() }
    private val installed = mutableListOf<File>()
    private var allowed = true
    private val uploads = UploadActivity()
    private lateinit var updater: Updater
    private lateinit var installer: UpdateInstaller

    private val installerFake = object : ApkInstaller {
        override suspend fun install(file: File) {
            installed += file
        }
    }

    @Before
    fun setUp() {
        server.start()
        val prefs = UpdatePrefs(ApplicationProvider.getApplicationContext<Context>().getSharedPreferences("update-installer-test", Context.MODE_PRIVATE).also { it.edit().clear().commit() })
        updater = Updater(OkHttpClient(), server.url("/").toString(), 2, File(folder.newFolder(), "updates"), installerFake, prefs, { 0L }, Dispatchers.Unconfined)
        installer = UpdateInstaller(updater, { allowed }, uploads)
    }

    @After
    fun tearDown() = server.shutdown()

    private fun sha() = MessageDigest.getInstance("SHA-256").digest(apk).joinToString("") { "%02x".format(it) }

    private fun enqueueRelease() {
        server.enqueue(MockResponse().setBody("""{"versionName":"0.3.1","versionCode":3,"url":"/download/a.apk","bytes":${apk.size},"sha256":"${sha()}","notes":["One"]}"""))
        server.enqueue(MockResponse().setBody(Buffer().write(apk)))
    }

    @Test
    fun aReadyReleaseInstallsAtOnceEvenOnScreen() = runBlocking {
        enqueueRelease()
        updater.check()
        updater.foreground = true
        assertEquals(UpdateStart.Started, installer.update())
        assertEquals(1, installed.size)
        assertTrue(updater.state.value is UpdateState.Installing)
    }

    @Test
    fun aReleaseNotDownloadedYetIsDownloadedThenInstalled() = runBlocking {
        enqueueRelease()
        assertEquals(UpdateStart.Started, installer.update())
        assertEquals(1, installed.size)
    }

    @Test
    fun notAllowedDownloadsAndInstallsNothing() = runBlocking {
        allowed = false
        assertEquals(UpdateStart.NeedsAllow, installer.update())
        assertEquals(0, server.requestCount)
        assertTrue(installed.isEmpty())
    }

    @Test
    fun aFailedDownloadInstallsNothing() = runBlocking {
        server.enqueue(MockResponse().setResponseCode(500))
        assertEquals(UpdateStart.NothingToInstall, installer.update())
        assertTrue(installed.isEmpty())
    }

    @Test
    fun anUploadInFlightHoldsTheInstallUntilItEnds() = runBlocking {
        enqueueRelease()
        updater.check()
        uploads.begin()
        val run = async { installer.update() }
        repeat(5) { yield() }
        assertFalse(run.isCompleted)
        assertTrue(installed.isEmpty())
        uploads.end()
        assertEquals(UpdateStart.Started, run.await())
        assertEquals(1, installed.size)
    }

    @Test
    fun startRunsOutsideTheCallerAndOnlyOnceAtATime() = runBlocking {
        enqueueRelease()
        updater.check()
        uploads.begin()
        var runs = 0
        val done = kotlinx.coroutines.CompletableDeferred<UpdateStart>()
        val background = UpdateInstaller(updater, { allowed }, uploads, onRun = { runs++ })
        assertTrue(background.start { done.complete(it) })
        assertTrue(background.running.value)
        assertFalse(background.start())
        assertEquals(1, runs)
        uploads.end()
        assertEquals(UpdateStart.Started, done.await())
        assertFalse(background.running.value)
        assertEquals(1, installed.size)
    }

    @Test
    fun theNotificationFollowsTheRun() {
        val release = ReleaseInfo("0.3.1", 3, "/a.apk", 10, "x")
        assertEquals(UpdateProgress.Downloading(42), updateProgress(UpdateState.Downloading(release, 0.425f), uploading = false))
        assertEquals(UpdateProgress.WaitingForUpload, updateProgress(UpdateState.Ready(release, File("a")), uploading = true))
        assertEquals(UpdateProgress.Installing, updateProgress(UpdateState.Installing(release), uploading = false))
        assertEquals(UpdateProgress.Preparing, updateProgress(UpdateState.Checking, uploading = false))
    }
}

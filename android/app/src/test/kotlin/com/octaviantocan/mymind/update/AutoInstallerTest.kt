package com.octaviantocan.mymind.update

import android.content.Context
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
import java.security.MessageDigest

@RunWith(AndroidJUnit4::class)
class AutoInstallerTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val server = MockWebServer()
    private val apk = ByteArray(2000) { (it % 251).toByte() }
    private val installed = mutableListOf<File>()
    private val asked = mutableListOf<String>()
    private var allowed = true
    private val uploads = UploadActivity()
    private lateinit var updater: Updater
    private lateinit var installer: AutoInstaller

    private val installerFake = object : ApkInstaller {
        override suspend fun install(file: File) {
            installed += file
        }
    }

    @Before
    fun setUp() {
        server.start()
        val prefs = UpdatePrefs(ApplicationProvider.getApplicationContext<Context>().getSharedPreferences("auto-install-test", Context.MODE_PRIVATE).also { it.edit().clear().commit() })
        updater = Updater(OkHttpClient(), server.url("/").toString(), 2, File(folder.newFolder(), "updates"), installerFake, prefs, { 0L }, Dispatchers.Unconfined)
        installer = AutoInstaller(updater, { allowed }, uploads) { asked += it.versionName }
    }

    @After
    fun tearDown() = server.shutdown()

    private fun sha() = MessageDigest.getInstance("SHA-256").digest(apk).joinToString("") { "%02x".format(it) }

    private suspend fun ready() {
        server.enqueue(MockResponse().setBody("""{"versionName":"0.3.1","versionCode":3,"url":"/download/a.apk","bytes":${apk.size},"sha256":"${sha()}","notes":["One"]}"""))
        server.enqueue(MockResponse().setBody(Buffer().write(apk)))
        assertTrue(updater.check() is UpdateState.Ready)
    }

    @Test
    fun offScreenAndAllowedInstalls() = runBlocking {
        ready()
        updater.foreground = false
        assertEquals(AutoInstallResult.Started, installer.installIfIdle())
        assertEquals(1, installed.size)
        assertTrue(updater.state.value is UpdateState.Installing)
    }

    @Test
    fun onScreenNeverInstalls() = runBlocking {
        ready()
        updater.foreground = true
        assertEquals(AutoInstallResult.OnScreen, installer.installIfIdle())
        assertTrue(installed.isEmpty())
        assertTrue(asked.isEmpty())
    }

    @Test
    fun anUploadHoldsTheInstallBackUntilItEnds() = runBlocking {
        ready()
        uploads.begin()
        assertEquals(AutoInstallResult.Uploading, installer.installIfIdle())
        assertTrue(installed.isEmpty())
        uploads.end()
        assertEquals(AutoInstallResult.Started, installer.installIfIdle())
        assertEquals(1, installed.size)
    }

    @Test
    fun withoutThePermissionTheUserIsAskedAndNothingInstalls() = runBlocking {
        ready()
        allowed = false
        assertEquals(AutoInstallResult.NeedsAllow, installer.installIfIdle())
        assertEquals(listOf("0.3.1"), asked)
        assertTrue(installed.isEmpty())
    }

    @Test
    fun withoutAReadyReleaseNothingHappens() = runBlocking {
        assertEquals(AutoInstallResult.NothingToInstall, installer.installIfIdle())
        assertTrue(installed.isEmpty())
    }

    @Test
    fun aTapInstallsOnlyWhenAllowedAndIdle() = runBlocking {
        ready()
        uploads.begin()
        assertFalse(installer.installNow())
        uploads.end()
        allowed = false
        assertFalse(installer.installNow())
        allowed = true
        assertTrue(installer.installNow())
        assertEquals(1, installed.size)
    }

    @Test
    fun theTrackerFollowsTheScreenAndReportsLeaving() {
        var left = 0
        val tracker = ForegroundTracker(updater) { left++ }
        val owner = object : androidx.lifecycle.LifecycleOwner {
            override val lifecycle = androidx.lifecycle.LifecycleRegistry.createUnsafe(this)
        }
        tracker.onStart(owner)
        assertTrue(updater.foreground)
        tracker.onStop(owner)
        assertFalse(updater.foreground)
        assertEquals(1, left)
    }
}

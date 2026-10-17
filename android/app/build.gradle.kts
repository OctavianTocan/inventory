import com.octaviantocan.mymind.changelog.Changelog
import java.nio.file.Files

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.roborazzi)
}

val mymindResourceFiles = listOf(
    "raw/everything_light.json",
    "raw/everything_dark.json",
    "raw/spaces_light.json",
    "raw/spaces_dark.json",
    "raw/serendipity_light.json",
    "raw/serendipity_dark.json",
    "raw/load_spin.json",
    "font/avenir_roman.otf",
    "font/louize_regular.otf",
    "font/nunito.ttf",
    "drawable-xxxhdpi/splash_bg.png",
    "drawable-xxxhdpi/mymind_splash_icon.png",
    "drawable-nodpi/offline_notice.png",
    "drawable/offline_bg.jpg",
    "drawable/offline_bg_dark.png",
    "drawable/svg_google.xml",
    "drawable/ic_caret_down.xml",
    "drawable/ic_horizontal_menu.xml",
    "drawable/settings_background_light.png",
    "drawable/settings_background_dark.png",
    "drawable/icon_pile_light.png",
    "drawable/icon_pile_dark.png",
    "drawable/ic_hand.xml",
    "drawable/help.xml",
    "drawable/sign_out.xml",
    "drawable/mymind_badge.png",
    "drawable/addtagsicon.xml",
    "drawable/addtospaceicon.xml",
    "drawable/copytoclipboardicon.xml",
    "drawable/visitsourceicon.xml",
    "drawable/downloadicon.xml",
    "drawable/samevibeicon.xml",
    "drawable/deleteicon.xml",
    "drawable/removefromspaceicon.xml",
    "drawable/topofmindicon.xml",
    "drawable/removetopofmindicon.xml",
    "drawable/readermodeicon.xml",
    "drawable/upgrade_dark.png",
    "drawable/upgrade_light.png",
    "drawable/close.xml",
    "drawable/spaces_checkmark.xml",
    "drawable/ic_chevron_down_light.xml",
    "drawable/ic_spaces.xml",
    "drawable/baseline_photo_library_24.xml",
    "drawable/baseline_camera_alt_24.xml",
    "drawable/ic_upload_progress.png",
    "drawable/ic_image.xml",
    "drawable/ic_video.xml",
    "drawable/ic_pdf.xml",
    "drawable/ic_text.xml",
    "drawable/ic_delete.xml",
)

val robolectricJvmArgs = listOf(
    "--add-exports=java.base/jdk.internal.access=ALL-UNNAMED",
) + listOf(
    "java.lang", "java.lang.reflect", "java.io", "java.net", "java.nio", "java.util",
    "java.util.concurrent", "java.util.concurrent.atomic", "java.text", "sun.nio.ch", "sun.nio.fs",
).map { "--add-opens=java.base/$it=ALL-UNNAMED" }

abstract class CopyMymindResources : DefaultTask() {
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.NONE)
    abstract val sources: ConfigurableFileCollection

    @get:OutputDirectory
    abstract val output: DirectoryProperty

    @TaskAction
    fun copy() {
        val missing = sources.files.filter { !it.isFile }
        if (missing.isNotEmpty()) {
            throw GradleException(
                "mymind resources missing in android/decoded: ${missing.joinToString { it.name }}. " +
                    "Decode the original app into android/decoded first (docs/android-plan.md).",
            )
        }
        val target = output.get().asFile.toPath()
        target.toFile().deleteRecursively()
        sources.files.forEach { file ->
            val folder = target.resolve(file.parentFile.name)
            Files.createDirectories(folder)
            Files.copy(file.toPath(), folder.resolve(file.name))
        }
    }
}

abstract class CopyChangelog : DefaultTask() {
    @get:InputFile
    @get:PathSensitive(PathSensitivity.NONE)
    abstract val source: RegularFileProperty

    @get:OutputDirectory
    abstract val output: DirectoryProperty

    @TaskAction
    fun copy() {
        val target = output.get().asFile
        target.deleteRecursively()
        target.mkdirs()
        source.get().asFile.copyTo(target.resolve("CHANGELOG.md"))
    }
}

val changelogFile: File = providers.gradleProperty("mymindChangelog").orNull
    ?.let { File(it).let { file -> if (file.isAbsolute) file else rootProject.file(file) } }
    ?: rootProject.file("CHANGELOG.md")
val changelogEntries = changelogFile.let { file ->
    if (!file.isFile) throw GradleException("Changelog missing: ${file.path}. Every version needs an entry in android/CHANGELOG.md.")
    Changelog.parse(file.readText(), file.name)
}
val latestEntry = changelogEntries.first()

val releaseTasks = setOf("assembleRelease", "bundleRelease", "packageRelease")
gradle.taskGraph.whenReady {
    if (allTasks.any { it.project == project && it.name in releaseTasks } && System.getenv("ANDROID_RELEASE_PASSWORD").isNullOrEmpty()) {
        throw GradleException(
            "Release builds need ANDROID_RELEASE_PASSWORD (and MYMIND_RELEASE_STORE and MYMIND_RELEASE_ALIAS when they differ " +
                "from ~/.config/mymind/android-release.jks and mymind) in the environment.",
        )
    }
}

val copyChangelog = tasks.register<CopyChangelog>("copyChangelog") {
    source.set(changelogFile)
    output.set(layout.buildDirectory.dir("generated/changelog-assets"))
}

val copyMymindResources = tasks.register<CopyMymindResources>("copyMymindResources") {
    sources.from(mymindResourceFiles.map { rootProject.file("decoded/res/$it") })
    output.set(layout.projectDirectory.dir("generated/mymind-res"))
}

androidComponents {
    onVariants { variant ->
        variant.sources.res?.addGeneratedSourceDirectory(copyMymindResources, CopyMymindResources::output)
        variant.sources.assets?.addGeneratedSourceDirectory(copyChangelog, CopyChangelog::output)
        variant.sources.kotlin?.addStaticSourceDirectory(rootProject.file("buildSrc/src/main/kotlin/com/octaviantocan/mymind/changelog").path)
    }
}

roborazzi {
    outputDir.set(layout.projectDirectory.dir("screenshots"))
}

fun webClientId(): String {
    providers.gradleProperty("mymindGoogleWebClientId").orNull?.let { return it }
    val env = rootProject.file("../.env")
    if (!env.isFile) return ""
    return env.readLines()
        .firstOrNull { it.startsWith("GOOGLE_CLIENT_ID=") }
        ?.substringAfter('=')?.trim()?.trim('"', '\'')
        .orEmpty()
}

val mymindServer: String = providers.gradleProperty("mymindServer").getOrElse("https://mymind.octaviantocan.com")
val mymindDownloads: String = providers.gradleProperty("mymindDownloads").getOrElse(mymindServer)

fun quoted(value: String) = "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

android {
    namespace = "com.octaviantocan.mymind"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.octaviantocan.mymind"
        minSdk = 32
        targetSdk = 37
        versionCode = latestEntry.versionCode
        versionName = latestEntry.versionName
        buildConfigField("String", "SERVER_URL", quoted(mymindServer))
        buildConfigField("String", "DOWNLOADS_URL", quoted(mymindDownloads))
        buildConfigField("String", "GOOGLE_WEB_CLIENT_ID", quoted(webClientId()))
    }

    signingConfigs {
        create("release") {
            val home = System.getProperty("user.home")
            storeFile = File(System.getenv("MYMIND_RELEASE_STORE") ?: "$home/.config/mymind/android-release.jks")
            storeType = "PKCS12"
            keyAlias = System.getenv("MYMIND_RELEASE_ALIAS") ?: "mymind"
            storePassword = System.getenv("ANDROID_RELEASE_PASSWORD")
            keyPassword = System.getenv("ANDROID_RELEASE_PASSWORD")
        }
    }

    buildTypes {
        debug {
            manifestPlaceholders["appLabel"] = "mymind dev"
            manifestPlaceholders["networkSecurityConfig"] = "@xml/network_loopback"
        }
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("release")
            manifestPlaceholders["appLabel"] = "mymind"
            // Cleartext to the emulator's host only for a release pointed at a local server, as in the end-to-end run.
            val local = listOf(mymindServer, mymindDownloads).any { it.startsWith("http://") }
            manifestPlaceholders["networkSecurityConfig"] = if (local) "@xml/network_loopback" else "@xml/network_strict"
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    testOptions {
        unitTests.isReturnDefaultValues = true
        unitTests.isIncludeAndroidResources = true
        unitTests.all {
            it.maxHeapSize = "2g"
            it.jvmArgs(robolectricJvmArgs)
        }
    }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.datastore.preferences)
    implementation(libs.androidx.work.runtime.ktx)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.material)
    implementation(libs.androidx.compose.material3)
    implementation(libs.lottie.compose)
    implementation(libs.androidx.credentials)
    implementation(libs.androidx.credentials.play.services.auth)
    implementation(libs.googleid)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.okhttp)
    implementation(libs.coil.compose)
    implementation(libs.coil.network.okhttp)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.okhttp.mockwebserver)
    testImplementation(libs.robolectric)
    testImplementation(libs.roborazzi)
    testImplementation(libs.roborazzi.compose)
    testImplementation(libs.androidx.test.core)
    testImplementation(libs.androidx.test.ext.junit)
    testImplementation(platform(libs.androidx.compose.bom))
    testImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}

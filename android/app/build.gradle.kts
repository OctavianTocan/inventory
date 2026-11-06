import com.octaviantocan.mymind.changelog.Changelog

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.roborazzi)
}

val robolectricJvmArgs = listOf(
    "--add-exports=java.base/jdk.internal.access=ALL-UNNAMED",
) + listOf(
    "java.lang", "java.lang.reflect", "java.io", "java.net", "java.nio", "java.util",
    "java.util.concurrent", "java.util.concurrent.atomic", "java.text", "sun.nio.ch", "sun.nio.fs",
).map { "--add-opens=java.base/$it=ALL-UNNAMED" }

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

androidComponents {
    onVariants { variant ->
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
            manifestPlaceholders["appLabel"] = "Inventory dev"
            manifestPlaceholders["networkSecurityConfig"] = "@xml/network_loopback"
        }
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            signingConfig = signingConfigs.getByName("release")
            manifestPlaceholders["appLabel"] = "Inventory"
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
    implementation(project(":ai-kit"))
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.webkit)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.lifecycle.process)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.datastore.preferences)
    implementation(libs.androidx.work.runtime.ktx)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.material)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.credentials)
    implementation(libs.androidx.credentials.play.services.auth)
    // Credentials' biometric puts fragment 1.2.5 on the compile classpath, and release lint then refuses
    // registerForActivityResult; the APK already ships 1.5.7, so this only makes compile and runtime agree.
    implementation(libs.androidx.fragment)
    implementation(libs.googleid)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.okhttp)
    implementation(libs.coil.compose)
    implementation(libs.coil.network.okhttp)
    implementation(libs.media3.exoplayer)
    implementation(libs.media3.datasource.okhttp)
    implementation(libs.media3.ui.compose)

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

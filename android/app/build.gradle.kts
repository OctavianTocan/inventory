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

val copyMymindResources = tasks.register<CopyMymindResources>("copyMymindResources") {
    sources.from(mymindResourceFiles.map { rootProject.file("decoded/res/$it") })
    output.set(layout.projectDirectory.dir("generated/mymind-res"))
}

androidComponents {
    onVariants { variant ->
        variant.sources.res?.addGeneratedSourceDirectory(copyMymindResources, CopyMymindResources::output)
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

fun quoted(value: String) = "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

android {
    namespace = "com.octaviantocan.mymind"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.octaviantocan.mymind"
        minSdk = 32
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"
        buildConfigField("String", "SERVER_URL", quoted(providers.gradleProperty("mymindServer").getOrElse("https://mymind.octaviantocan.com")))
        buildConfigField("String", "GOOGLE_WEB_CLIENT_ID", quoted(webClientId()))
    }

    buildTypes {
        debug {
            manifestPlaceholders["appLabel"] = "mymind dev"
        }
        release {
            isMinifyEnabled = false
            manifestPlaceholders["appLabel"] = "mymind"
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

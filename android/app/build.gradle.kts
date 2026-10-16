import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
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
    implementation(libs.androidx.compose.material3)
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
}

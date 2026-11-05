pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "mymind-android"
include(":app")
// The app half of the AI kit shared with Pully (Claude Code and Codex sign-in, models, efforts), from the
// vendor/ai-kit submodule (OctavianTocan/ai-kit).
include(":ai-kit")
project(":ai-kit").projectDir = file("../vendor/ai-kit/kotlin")

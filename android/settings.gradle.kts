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
// vendor/pulls.review submodule.
include(":ai-kit")
project(":ai-kit").projectDir = file("../vendor/pulls.review/ai-kit/kotlin")

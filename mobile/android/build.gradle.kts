// Kotlin is compiled by AGP's built-in Kotlin support (AGP 9+); no Kotlin
// Gradle plugin is applied anywhere in this build. Every module pins its
// own compileSdk/buildToolsVersion and Java 11 bytecode level.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
}

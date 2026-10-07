// Shim module. `sparkling-storage`'s build script depends on
// `project(":sparkling-method")` (a leftover from the Sparkling monorepo),
// while the SDK ships that library on Maven. This empty Android library
// satisfies the project reference and re-exports the published artifact, so
// every module resolves the one and only copy of sparkling-method.
plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "app.imkitchen.android.sparklingmethod"
    compileSdk = 34

    defaultConfig {
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
}

dependencies {
    api("com.tiktok.sparkling:sparkling-method:2.0.1")
}

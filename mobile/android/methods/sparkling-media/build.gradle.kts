// Build script for the published `sparkling-media` npm package (sources in
// mobile/node_modules/sparkling-media/android). Its own android/build.gradle.kts
// targets AGP 7 / Kotlin 1.8 and cannot be evaluated by this build; see
// settings.gradle.kts. Keep the dependency list in sync with the package.
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
}

val packageSources = rootProject.layout.projectDirectory.dir("../node_modules/sparkling-media/android/src/main/java")

android {
    namespace = "com.tiktok.sparkling.method.media"
    compileSdk = 37
    buildToolsVersion = "37.0.0"

    defaultConfig {
        minSdk = 21
    }

    sourceSets {
        getByName("main").apply {
            java.setSrcDirs(listOf(packageSources))
            kotlin.setSrcDirs(listOf(packageSources))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_11
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    api(project(":sparkling-method"))
}

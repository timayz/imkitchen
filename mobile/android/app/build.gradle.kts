import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
}

android {
    namespace = "app.imkitchen.android"
    compileSdk = 37
    buildToolsVersion = "37.0.0"

    defaultConfig {
        applicationId = "app.imkitchen.android"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        ndk {
            abiFilters.addAll(listOf("armeabi-v7a", "arm64-v8a"))
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    buildFeatures {
        // AGP 8+ no longer generates BuildConfig by default; SparklingApplication reads BuildConfig.DEBUG.
        buildConfig = true
    }

    // Default integrate assets from dist; switch to native assets when env is set
    val useNativeAssets =
        System.getenv("SPARKLING_USE_NATIVE_ASSETS")?.equals("true", ignoreCase = true) ?: false
    sourceSets {
        getByName("main").apply {
            if (useNativeAssets) {
                // Use native assets directory (used in --copy mode)
                assets.setSrcDirs(listOf("src/main/assets"))
            } else {
                // Default: use dist directly; no copy required
                assets.setSrcDirs(listOf("../../dist"))
            }
        }
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
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)

    implementation("com.tiktok.sparkling:sparkling:2.0.1")
    implementation("com.tiktok.sparkling:sparkling-method:2.0.1")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")

    implementation(libs.fresco)
    implementation(libs.fresco.animated.gif)
    implementation(libs.fresco.animated.webp)
    implementation(libs.fresco.webp.support)
    implementation(libs.fresco.animated.base)

    // BEGIN SPARKLING AUTOLINK
    listOf(
        project(":sparkling-keep-awake"),
        project(":sparkling-db"),
        project(":sparkling-navigation"),
        project(":sparkling-media"),
        project(":sparkling-storage")
    ).forEach { dep -> add("implementation", dep) }
    // END SPARKLING AUTOLINK
}

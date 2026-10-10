import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
}

// A Gradle property wins over the env var, so a CI secret can be overridden locally.
fun prop(name: String, env: String): String? =
    project.findProperty(name) as String? ?: System.getenv(env)

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

    // Signing is manual: the keystore lives outside the repo and is named by
    // `imkitchen.keystore` in ~/.gradle/gradle.properties, or by the matching
    // IMKITCHEN_* env vars in CI. With none of them set the release build stays
    // unsigned rather than falling back to a key nobody chose.
    val keystore = prop("imkitchen.keystore", "IMKITCHEN_KEYSTORE")
    signingConfigs {
        if (keystore != null) {
            create("release") {
                storeFile = file(keystore)
                storePassword = prop("imkitchen.keystore.password", "IMKITCHEN_KEYSTORE_PASSWORD")
                keyAlias = prop("imkitchen.key.alias", "IMKITCHEN_KEY_ALIAS")
                keyPassword = prop("imkitchen.key.password", "IMKITCHEN_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.findByName("release")
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

    // Sparkling 2.0.1 only declares Lynx 3.6.0 transitively; pin the engine
    // explicitly so the whole org.lynxsdk.lynx graph resolves to the version
    // in gradle/libs.versions.toml (Gradle picks the highest requested).
    implementation(libs.lynx)
    implementation(libs.lynx.jssdk)
    implementation(libs.lynx.trace)
    implementation(libs.primjs)
    implementation(libs.lynx.service.image)
    implementation(libs.lynx.service.log)
    implementation(libs.lynx.service.http)
    implementation(libs.lynx.service.devtool)
    implementation(libs.lynx.devtool)

    implementation(libs.fresco)
    implementation(libs.fresco.animated.gif)
    implementation(libs.fresco.animated.webp)
    implementation(libs.fresco.webp.support)
    implementation(libs.fresco.animated.base)

    // BEGIN SPARKLING AUTOLINK
    listOf(
        project(":sparkling-db"),
        project(":sparkling-keep-awake"),
        project(":sparkling-media"),
        project(":sparkling-navigation"),
        project(":sparkling-storage"),
        project(":sparkling-timer-alarm")
    ).forEach { dep -> add("implementation", dep) }
    // END SPARKLING AUTOLINK
}

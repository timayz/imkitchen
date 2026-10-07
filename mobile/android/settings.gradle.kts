pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
}
}

rootProject.name = "imkitchen"
include(":app")

// Shim for the `project(":sparkling-method")` reference inside the autolinked
// method packages; see sparkling-method/build.gradle.kts.
include(":sparkling-method")
project(":sparkling-method").projectDir = file("sparkling-method")

// The published method packages still ship AGP 7 / Kotlin 1.8 build scripts
// (`kotlinOptions`, compileSdk 34, the Kotlin Gradle plugin that AGP 9's DSL
// rejects). Keep their sources in node_modules but build them with the
// scripts under methods/<name>/, which read the sources from there.
// `sparkling autolink` re-appends its block at the end of this file on every
// run, so the override is applied once the whole script has been evaluated.
gradle.settingsEvaluated {
    listOf("sparkling-media", "sparkling-navigation", "sparkling-storage").forEach { name ->
        project(":$name").projectDir = file("methods/$name")
    }
}

// BEGIN SPARKLING AUTOLINK
val sparklingAutolinkProjects = listOf<Pair<String, java.io.File>>(
  "sparkling-db" to file("../node_modules/sparkling-db/android"),
  "sparkling-keep-awake" to file("../node_modules/sparkling-keep-awake/android"),
  "sparkling-media" to file("../node_modules/sparkling-media/android"),
  "sparkling-navigation" to file("../node_modules/sparkling-navigation/android"),
  "sparkling-storage" to file("../node_modules/sparkling-storage/android")
)
sparklingAutolinkProjects.forEach { (name, dir) ->
    include(":$name")
    project(":$name").projectDir = dir
}
// END SPARKLING AUTOLINK

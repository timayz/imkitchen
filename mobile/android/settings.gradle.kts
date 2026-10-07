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

// BEGIN SPARKLING AUTOLINK
val sparklingAutolinkProjects = listOf<Pair<String, java.io.File>>(
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

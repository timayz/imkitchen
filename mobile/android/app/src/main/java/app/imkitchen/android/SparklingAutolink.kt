package app.imkitchen.android

data class SparklingAutolinkModule(val name: String, val androidPackage: String?, val className: String?)

object SparklingAutolink {
    val modules = listOf(
        SparklingAutolinkModule(name = "sparkling-keep-awake", androidPackage = "app.imkitchen.keepawake", className = "KeepAwakeSetEnabledMethod"),
        SparklingAutolinkModule(name = "sparkling-db", androidPackage = "app.imkitchen.db", className = "DbGetMethod"),
        SparklingAutolinkModule(name = "sparkling-navigation", androidPackage = "com.tiktok.sparkling.methods.router", className = "RouterMethod"),
        SparklingAutolinkModule(name = "sparkling-media", androidPackage = "", className = ""),
        SparklingAutolinkModule(name = "sparkling-storage", androidPackage = "com.tiktok.sparkling.methods.storage", className = "StorageMethod")
    )
}

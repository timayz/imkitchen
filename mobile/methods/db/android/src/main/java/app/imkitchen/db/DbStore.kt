package app.imkitchen.db

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.Executors

/**
 * One SQLite table of JSON documents, keyed by `(collection, key)`. The
 * value is opaque to the native side: the TypeScript layer owns the schema
 * of what it stores (cached API responses, the offline write queue).
 *
 * Every operation runs on a single background thread, so calls from the
 * bridge are serialized (a `put` followed by a `get` sees the write) and
 * SQLite never blocks the bridge thread.
 */
object DbStore {
    private const val NAME = "imkitchen.db"
    private const val VERSION = 1
    private const val TABLE = "documents"

    private val executor = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "imkitchen-db")
    }

    @Volatile
    private var helper: Helper? = null

    private class Helper(context: Context) :
        SQLiteOpenHelper(context.applicationContext, NAME, null, VERSION) {

        override fun onCreate(db: SQLiteDatabase) {
            db.execSQL(
                """
                CREATE TABLE IF NOT EXISTS $TABLE (
                  collection TEXT NOT NULL,
                  key        TEXT NOT NULL,
                  value      TEXT NOT NULL,
                  updated_at INTEGER NOT NULL,
                  PRIMARY KEY (collection, key)
                ) WITHOUT ROWID
                """.trimIndent()
            )
        }

        override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
            // Documents are a cache: a schema change may simply start over.
            db.execSQL("DROP TABLE IF EXISTS $TABLE")
            onCreate(db)
        }
    }

    private fun db(context: Context): SQLiteDatabase {
        val existing = helper
        if (existing != null) return existing.writableDatabase
        return synchronized(this) {
            val created = helper ?: Helper(context).also {
                it.setWriteAheadLoggingEnabled(true)
                helper = it
            }
            created.writableDatabase
        }
    }

    /** Runs `block` on the store's thread. */
    fun execute(block: () -> Unit) {
        executor.execute(block)
    }

    fun get(context: Context, collection: String, key: String): String? {
        db(context).query(
            TABLE, arrayOf("value"), "collection = ? AND key = ?",
            arrayOf(collection, key), null, null, null, "1"
        ).use { cursor ->
            return if (cursor.moveToFirst()) cursor.getString(0) else null
        }
    }

    fun put(context: Context, collection: String, key: String, value: String) {
        val row = ContentValues(4).apply {
            put("collection", collection)
            put("key", key)
            put("value", value)
            put("updated_at", System.currentTimeMillis())
        }
        db(context).insertWithOnConflict(TABLE, null, row, SQLiteDatabase.CONFLICT_REPLACE)
    }

    fun remove(context: Context, collection: String, key: String) {
        db(context).delete(TABLE, "collection = ? AND key = ?", arrayOf(collection, key))
    }

    /** `[{key, value, updated_at}]` ordered by key, as a JSON string. */
    fun list(context: Context, collection: String, prefix: String?): String {
        val (where, args) = if (prefix.isNullOrEmpty()) {
            "collection = ?" to arrayOf(collection)
        } else {
            "collection = ? AND key >= ? AND key < ?" to arrayOf(collection, prefix, prefix + "￿")
        }
        val rows = JSONArray()
        db(context).query(
            TABLE, arrayOf("key", "value", "updated_at"), where, args, null, null, "key ASC"
        ).use { cursor ->
            while (cursor.moveToNext()) {
                rows.put(
                    JSONObject()
                        .put("key", cursor.getString(0))
                        .put("value", cursor.getString(1))
                        .put("updated_at", cursor.getLong(2))
                )
            }
        }
        return rows.toString()
    }

    fun clear(context: Context, collection: String?) {
        if (collection.isNullOrEmpty()) {
            db(context).delete(TABLE, null, null)
        } else {
            db(context).delete(TABLE, "collection = ?", arrayOf(collection))
        }
    }
}

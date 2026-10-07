import Foundation
import SQLite3

/// One SQLite table of JSON documents, keyed by `(collection, key)`. The
/// value is opaque to the native side: the TypeScript layer owns the schema
/// of what it stores (cached API responses, the offline write queue).
///
/// Mirrors `DbStore.kt` on Android: every operation runs on a single serial
/// queue, so calls from the bridge are serialized (a `put` followed by a
/// `get` sees the write) and SQLite never blocks the bridge thread.
final class DbStore {
    static let shared = DbStore()

    struct Failure: Error, CustomStringConvertible {
        let description: String
    }

    private static let table = "documents"
    private static let transient = unsafeBitCast(-1, to: sqlite3_destructor_type.self)

    private let queue = DispatchQueue(label: "app.imkitchen.db")
    private var handle: OpaquePointer?

    private init() {}

    /// Runs `block` on the store's queue.
    func execute(_ block: @escaping () -> Void) {
        queue.async(execute: block)
    }

    func get(collection: String, key: String) throws -> String? {
        let db = try open()
        var value: String?
        try query(
            db,
            "SELECT value FROM \(Self.table) WHERE collection = ? AND key = ? LIMIT 1",
            [collection, key]
        ) { statement in
            value = Self.text(statement, 0)
        }
        return value
    }

    func put(collection: String, key: String, value: String) throws {
        let db = try open()
        let now = Int64(Date().timeIntervalSince1970 * 1000)
        try query(
            db,
            "INSERT OR REPLACE INTO \(Self.table) (collection, key, value, updated_at) VALUES (?, ?, ?, ?)",
            [collection, key, value, now]
        )
    }

    func remove(collection: String, key: String) throws {
        let db = try open()
        try query(db, "DELETE FROM \(Self.table) WHERE collection = ? AND key = ?", [collection, key])
    }

    /// `[{key, value, updated_at}]` ordered by key, as a JSON string.
    func list(collection: String, prefix: String?) throws -> String {
        let db = try open()
        let sql: String
        let args: [Any]
        if let prefix, !prefix.isEmpty {
            sql = "SELECT key, value, updated_at FROM \(Self.table) WHERE collection = ? AND key >= ? AND key < ? ORDER BY key ASC"
            args = [collection, prefix, prefix + "\u{FFFF}"]
        } else {
            sql = "SELECT key, value, updated_at FROM \(Self.table) WHERE collection = ? ORDER BY key ASC"
            args = [collection]
        }
        var rows: [[String: Any]] = []
        try query(db, sql, args) { statement in
            rows.append([
                "key": Self.text(statement, 0) ?? "",
                "value": Self.text(statement, 1) ?? "",
                "updated_at": sqlite3_column_int64(statement, 2),
            ])
        }
        let data = try JSONSerialization.data(withJSONObject: rows)
        return String(decoding: data, as: UTF8.self)
    }

    func clear(collection: String?) throws {
        let db = try open()
        if let collection, !collection.isEmpty {
            try query(db, "DELETE FROM \(Self.table) WHERE collection = ?", [collection])
        } else {
            try query(db, "DELETE FROM \(Self.table)", [])
        }
    }

    // MARK: - SQLite plumbing

    private func open() throws -> OpaquePointer {
        if let handle { return handle }
        let directory = try FileManager.default.url(
            for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true
        )
        let path = directory.appendingPathComponent("imkitchen.db").path
        var db: OpaquePointer?
        let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_FULLMUTEX
        guard sqlite3_open_v2(path, &db, flags, nil) == SQLITE_OK, let db else {
            let message = db.map { String(cString: sqlite3_errmsg($0)) } ?? "unknown error"
            sqlite3_close(db)
            throw Failure(description: "open \(path): \(message)")
        }
        do {
            try query(db, "PRAGMA journal_mode=WAL", [])
            try query(
                db,
                """
                CREATE TABLE IF NOT EXISTS \(Self.table) (
                  collection TEXT NOT NULL,
                  key        TEXT NOT NULL,
                  value      TEXT NOT NULL,
                  updated_at INTEGER NOT NULL,
                  PRIMARY KEY (collection, key)
                ) WITHOUT ROWID
                """,
                []
            )
        } catch {
            sqlite3_close(db)
            throw error
        }
        handle = db
        return db
    }

    /// Prepares `sql`, binds `args` (String or Int64) and steps through every
    /// row, handing each to `onRow`.
    private func query(
        _ db: OpaquePointer,
        _ sql: String,
        _ args: [Any],
        onRow: (OpaquePointer) -> Void = { _ in }
    ) throws {
        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &statement, nil) == SQLITE_OK, let statement else {
            throw Failure(description: "prepare: \(String(cString: sqlite3_errmsg(db)))")
        }
        defer { sqlite3_finalize(statement) }

        for (index, arg) in args.enumerated() {
            let position = Int32(index + 1)
            let status: Int32
            switch arg {
            case let text as String:
                status = sqlite3_bind_text(statement, position, text, -1, Self.transient)
            case let number as Int64:
                status = sqlite3_bind_int64(statement, position, number)
            default:
                throw Failure(description: "bind: unsupported argument \(type(of: arg))")
            }
            guard status == SQLITE_OK else {
                throw Failure(description: "bind: \(String(cString: sqlite3_errmsg(db)))")
            }
        }

        while true {
            switch sqlite3_step(statement) {
            case SQLITE_ROW:
                onRow(statement)
            case SQLITE_DONE:
                return
            default:
                throw Failure(description: "step: \(String(cString: sqlite3_errmsg(db)))")
            }
        }
    }

    private static func text(_ statement: OpaquePointer, _ column: Int32) -> String? {
        guard let pointer = sqlite3_column_text(statement, column) else { return nil }
        return String(cString: pointer)
    }
}

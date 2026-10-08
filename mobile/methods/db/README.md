# sparkling-db

A SQLite document store behind the Sparkling bridge: the mobile app's offline
cache and write queue live here (`src/lib/db.ts`, `src/lib/cache.ts`,
`src/lib/offline/`). One table, `documents (collection, key, value, updated_at)`,
whose `value` is an opaque JSON string: TypeScript owns what is stored.

Hand-written like `methods/keep-awake` (the `sparkling-method-cli` codegen
emits invalid TypeScript/Swift).

Methods (`Db.<name>`), every result field a string or boolean so nothing
crosses the bridge as a disposable `PiperData`:

| Method      | Params                       | Result                                      |
| ----------- | ---------------------------- | ------------------------------------------- |
| `Db.get`    | `collection`, `key`          | `value` (null when missing)                 |
| `Db.put`    | `collection`, `key`, `value` | `ok`                                        |
| `Db.remove` | `collection`, `key`          | `ok`                                        |
| `Db.list`   | `collection`, `prefix?`      | `json`: `[{key, value, updated_at}]` by key |
| `Db.clear`  | `collection?`                | `ok` (no collection: everything)            |

- Android: `DbStore.kt` (raw `SQLiteOpenHelper`, WAL, one background thread
  so calls are serialized and never block the bridge). The five `Db*Method`
  classes are registered in `SparklingApplication.kt`.
- iOS: `ios/Sources/DbStore.swift` (raw `sqlite3`, WAL, one serial queue,
  same table and semantics; the file lives in Application Support). The five
  `Db*Method` classes in `DbMethods.swift` subclass `PipeMethod` directly and
  are picked up by `MethodRegistry.autoRegisterGlobalMethods()`; the pod is
  `ios/Sparkling-Db.podspec`. Built by the `Mobile` workflow only (no Mac).
  When the bridge answers "unregistered", `src/lib/db.ts` falls back to
  network-only.

Linked into the app with `npm install ./methods/db` + `npm run autolink`.

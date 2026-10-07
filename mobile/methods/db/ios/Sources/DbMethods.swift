import Foundation
import SparklingMethod

// The five `Db.*` methods (see ../../README.md), one class each. Every one
// subclasses `PipeMethod` directly because
// `MethodRegistry.autoRegisterGlobalMethods` only picks up direct subclasses;
// the shared plumbing lives in `DbCall` instead of a base class.

@objc(DbGetMethod)
public final class DbGetMethod: PipeMethod {
    public override var methodName: String { "Db.get" }
    public override class func methodName() -> String { "Db.get" }
    @objc public override var paramsModelClass: AnyClass { DbKeyParamModel.self }
    @objc public override var resultModelClass: AnyClass { DbValueResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? DbKeyParamModel else {
            return DbCall.invalid("Invalid parameter model type", completionHandler)
        }
        guard let collection = params.collection, let key = params.key, !collection.isEmpty, !key.isEmpty else {
            return DbCall.invalid("Db.get: empty collection/key", completionHandler)
        }
        DbCall.run("Db.get", completionHandler) {
            let result = DbValueResultModel()
            result.value = try DbStore.shared.get(collection: collection, key: key)
            return result
        }
    }
}

@objc(DbPutMethod)
public final class DbPutMethod: PipeMethod {
    public override var methodName: String { "Db.put" }
    public override class func methodName() -> String { "Db.put" }
    @objc public override var paramsModelClass: AnyClass { DbPutParamModel.self }
    @objc public override var resultModelClass: AnyClass { DbOkResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? DbPutParamModel else {
            return DbCall.invalid("Invalid parameter model type", completionHandler)
        }
        guard let collection = params.collection, let key = params.key, let value = params.value,
              !collection.isEmpty, !key.isEmpty, !value.isEmpty
        else {
            return DbCall.invalid("Db.put: empty collection/key/value", completionHandler)
        }
        DbCall.run("Db.put", completionHandler) {
            try DbStore.shared.put(collection: collection, key: key, value: value)
            return DbOkResultModel.ok()
        }
    }
}

@objc(DbRemoveMethod)
public final class DbRemoveMethod: PipeMethod {
    public override var methodName: String { "Db.remove" }
    public override class func methodName() -> String { "Db.remove" }
    @objc public override var paramsModelClass: AnyClass { DbKeyParamModel.self }
    @objc public override var resultModelClass: AnyClass { DbOkResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? DbKeyParamModel else {
            return DbCall.invalid("Invalid parameter model type", completionHandler)
        }
        guard let collection = params.collection, let key = params.key, !collection.isEmpty, !key.isEmpty else {
            return DbCall.invalid("Db.remove: empty collection/key", completionHandler)
        }
        DbCall.run("Db.remove", completionHandler) {
            try DbStore.shared.remove(collection: collection, key: key)
            return DbOkResultModel.ok()
        }
    }
}

@objc(DbListMethod)
public final class DbListMethod: PipeMethod {
    public override var methodName: String { "Db.list" }
    public override class func methodName() -> String { "Db.list" }
    @objc public override var paramsModelClass: AnyClass { DbListParamModel.self }
    @objc public override var resultModelClass: AnyClass { DbJsonResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? DbListParamModel else {
            return DbCall.invalid("Invalid parameter model type", completionHandler)
        }
        guard let collection = params.collection, !collection.isEmpty else {
            return DbCall.invalid("Db.list: empty collection", completionHandler)
        }
        let prefix = params.prefix
        DbCall.run("Db.list", completionHandler) {
            let result = DbJsonResultModel()
            result.json = try DbStore.shared.list(collection: collection, prefix: prefix)
            return result
        }
    }
}

@objc(DbClearMethod)
public final class DbClearMethod: PipeMethod {
    public override var methodName: String { "Db.clear" }
    public override class func methodName() -> String { "Db.clear" }
    @objc public override var paramsModelClass: AnyClass { DbClearParamModel.self }
    @objc public override var resultModelClass: AnyClass { DbOkResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? DbClearParamModel else {
            return DbCall.invalid("Invalid parameter model type", completionHandler)
        }
        let collection = params.collection
        DbCall.run("Db.clear", completionHandler) {
            try DbStore.shared.clear(collection: collection)
            return DbOkResultModel.ok()
        }
    }
}

// MARK: - Shared plumbing

enum DbCall {
    static func invalid(_ message: String, _ handler: PipeMethod.CompletionHandlerProtocol) {
        handler.handleCompletion(status: .invalidParameter(message: message), result: nil)
    }

    /// Runs `work` on the store's queue and reports back on the main queue.
    static func run(
        _ name: String,
        _ handler: PipeMethod.CompletionHandlerProtocol,
        _ work: @escaping () throws -> SPKMethodModel
    ) {
        DbStore.shared.execute {
            let status: MethodStatus
            let result: SPKMethodModel?
            do {
                result = try work()
                status = .succeeded()
            } catch {
                result = nil
                status = .failed(message: "\(name) failed: \(error)")
            }
            DispatchQueue.main.async {
                handler.handleCompletion(status: status, result: result)
            }
        }
    }
}

// MARK: - Models

@objc(DbKeyParamModel)
public class DbKeyParamModel: SPKMethodModel {
    @objc public var collection: String?
    @objc public var key: String?

    public override class func requiredKeyPaths() -> Set<String>? { ["collection", "key"] }

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["collection": "collection", "key": "key"]
    }
}

@objc(DbPutParamModel)
public class DbPutParamModel: SPKMethodModel {
    @objc public var collection: String?
    @objc public var key: String?
    @objc public var value: String?

    public override class func requiredKeyPaths() -> Set<String>? { ["collection", "key", "value"] }

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["collection": "collection", "key": "key", "value": "value"]
    }
}

@objc(DbListParamModel)
public class DbListParamModel: SPKMethodModel {
    @objc public var collection: String?
    @objc public var prefix: String?

    public override class func requiredKeyPaths() -> Set<String>? { ["collection"] }

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["collection": "collection", "prefix": "prefix"]
    }
}

@objc(DbClearParamModel)
public class DbClearParamModel: SPKMethodModel {
    @objc public var collection: String?

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["collection": "collection"]
    }
}

@objc(DbValueResultModel)
public class DbValueResultModel: SPKMethodModel {
    /// The document, `nil` (serialized as `null`) when there is none.
    @objc public var value: String?

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["value": "value"]
    }
}

@objc(DbJsonResultModel)
public class DbJsonResultModel: SPKMethodModel {
    @objc public var json: String?

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["json": "json"]
    }
}

@objc(DbOkResultModel)
public class DbOkResultModel: SPKMethodModel {
    @objc public var ok: Bool = false

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["ok": "ok"]
    }

    static func ok() -> DbOkResultModel {
        let model = DbOkResultModel()
        model.ok = true
        return model
    }
}

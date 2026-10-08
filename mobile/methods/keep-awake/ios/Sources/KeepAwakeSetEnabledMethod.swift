import Foundation
import SparklingMethod
import UIKit

/// `KeepAwake.setEnabled({enabled})`: keeps the screen on while the cooking
/// screen is up.
///
/// Unlike the Android implementation, which flags the calling activity's
/// window, iOS has one process-wide idle timer. The cooking screen turns it
/// back on when it unmounts (`src/lib/keep-awake.ts`), so the flag never
/// outlives the screen.
///
/// Must subclass `PipeMethod` directly: `MethodRegistry.autoRegisterGlobalMethods`
/// only registers direct subclasses.
@objc(KeepAwakeSetEnabledMethod)
public final class KeepAwakeSetEnabledMethod: PipeMethod {
    public override var methodName: String { "KeepAwake.setEnabled" }
    public override class func methodName() -> String { "KeepAwake.setEnabled" }

    @objc public override var paramsModelClass: AnyClass { KeepAwakeSetEnabledParamModel.self }
    @objc public override var resultModelClass: AnyClass { KeepAwakeSetEnabledResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? KeepAwakeSetEnabledParamModel else {
            completionHandler.handleCompletion(status: .invalidParameter(message: "Invalid parameter model type"), result: nil)
            return
        }
        let enabled = params.enabled
        DispatchQueue.main.async {
            UIApplication.shared.isIdleTimerDisabled = enabled
            let result = KeepAwakeSetEnabledResultModel()
            result.success = true
            completionHandler.handleCompletion(status: .succeeded(), result: result)
        }
    }
}

@objc(KeepAwakeSetEnabledParamModel)
public class KeepAwakeSetEnabledParamModel: SPKMethodModel {
    @objc public var enabled: Bool = false

    public override class func requiredKeyPaths() -> Set<String>? { ["enabled"] }

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["enabled": "enabled"]
    }
}

@objc(KeepAwakeSetEnabledResultModel)
public class KeepAwakeSetEnabledResultModel: SPKMethodModel {
    @objc public var success: Bool = false

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["success": "success"]
    }
}

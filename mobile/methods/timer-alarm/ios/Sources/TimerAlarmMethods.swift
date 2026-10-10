import Foundation
import SparklingMethod
import UIKit
import UserNotifications

// `TimerAlarm.schedule` / `TimerAlarm.cancel` (see ../../README.md): a local
// notification with a sound at an absolute time, so a cooking step still
// rings when the phone is locked or the app was suspended. Both classes
// subclass `PipeMethod` directly because
// `MethodRegistry.autoRegisterGlobalMethods` only picks up direct subclasses.

@objc(TimerAlarmScheduleMethod)
public final class TimerAlarmScheduleMethod: PipeMethod {
    public override var methodName: String { "TimerAlarm.schedule" }
    public override class func methodName() -> String { "TimerAlarm.schedule" }
    @objc public override var paramsModelClass: AnyClass { TimerAlarmScheduleParamModel.self }
    @objc public override var resultModelClass: AnyClass { TimerAlarmScheduleResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? TimerAlarmScheduleParamModel else {
            completionHandler.handleCompletion(status: .invalidParameter(message: "Invalid parameter model type"), result: nil)
            return
        }
        guard let id = params.identifier, !id.isEmpty, params.at > 0 else {
            completionHandler.handleCompletion(status: .invalidParameter(message: "TimerAlarm.schedule: empty id or time"), result: nil)
            return
        }
        TimerAlarmCenter.shared.schedule(
            id: id,
            at: Date(timeIntervalSince1970: params.at / 1000),
            title: params.title ?? "",
            body: params.body ?? ""
        ) { outcome in
            DispatchQueue.main.async {
                switch outcome {
                case .scheduled(let notifications):
                    let result = TimerAlarmScheduleResultModel()
                    result.scheduled = true
                    result.exact = true
                    result.notifications = notifications
                    completionHandler.handleCompletion(status: .succeeded(), result: result)
                case .failed(let message):
                    completionHandler.handleCompletion(status: .failed(message: "TimerAlarm.schedule failed: \(message)"), result: nil)
                }
            }
        }
    }
}

@objc(TimerAlarmCancelMethod)
public final class TimerAlarmCancelMethod: PipeMethod {
    public override var methodName: String { "TimerAlarm.cancel" }
    public override class func methodName() -> String { "TimerAlarm.cancel" }
    @objc public override var paramsModelClass: AnyClass { TimerAlarmCancelParamModel.self }
    @objc public override var resultModelClass: AnyClass { TimerAlarmCancelResultModel.self }

    @objc public override func call(withParamModel paramModel: Any, completionHandler: CompletionHandlerProtocol) {
        guard let params = paramModel as? TimerAlarmCancelParamModel else {
            completionHandler.handleCompletion(status: .invalidParameter(message: "Invalid parameter model type"), result: nil)
            return
        }
        guard let id = params.identifier, !id.isEmpty else {
            completionHandler.handleCompletion(status: .invalidParameter(message: "TimerAlarm.cancel: empty id"), result: nil)
            return
        }
        TimerAlarmCenter.shared.cancel(id: id)
        let result = TimerAlarmCancelResultModel()
        result.ok = true
        completionHandler.handleCompletion(status: .succeeded(), result: result)
    }
}

// MARK: - Notification center

/// Owns the pending timer notifications and, as the center's delegate, keeps
/// them audible while the app is in the foreground (the default is to
/// swallow a notification for the active app).
final class TimerAlarmCenter: NSObject, UNUserNotificationCenterDelegate {
    static let shared = TimerAlarmCenter()

    enum Outcome {
        /// `notifications` is whether the user allows them; the request is queued regardless.
        case scheduled(notifications: Bool)
        case failed(String)
    }

    private let center = UNUserNotificationCenter.current()

    private override init() {
        super.init()
    }

    func schedule(id: String, at: Date, title: String, body: String, completion: @escaping (Outcome) -> Void) {
        let center = self.center
        center.delegate = self
        // Answers at once with the stored decision after the first prompt.
        center.requestAuthorization(options: [.alert, .sound]) { granted, _ in
            let content = UNMutableNotificationContent()
            content.title = title
            content.body = body
            content.sound = .default
            let seconds = max(1, at.timeIntervalSinceNow)
            let trigger = UNTimeIntervalNotificationTrigger(timeInterval: seconds, repeats: false)
            let request = UNNotificationRequest(identifier: id, content: content, trigger: trigger)
            center.add(request) { error in
                if let error = error {
                    completion(.failed(error.localizedDescription))
                } else {
                    completion(.scheduled(notifications: granted))
                }
            }
        }
    }

    func cancel(id: String) {
        center.removePendingNotificationRequests(withIdentifiers: [id])
        center.removeDeliveredNotifications(withIdentifiers: [id])
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        if #available(iOS 14.0, *) {
            completionHandler([.banner, .list, .sound])
        } else {
            completionHandler([.alert, .sound])
        }
    }
}

// MARK: - Models

@objc(TimerAlarmScheduleParamModel)
public class TimerAlarmScheduleParamModel: SPKMethodModel {
    // `id` is the JSON key; the property avoids the Objective-C type name.
    @objc public var identifier: String?
    /// Epoch milliseconds.
    @objc public var at: Double = 0
    @objc public var title: String?
    @objc public var body: String?

    public override class func requiredKeyPaths() -> Set<String>? { ["id", "at", "title", "body"] }

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["identifier": "id", "at": "at", "title": "title", "body": "body"]
    }
}

@objc(TimerAlarmScheduleResultModel)
public class TimerAlarmScheduleResultModel: SPKMethodModel {
    @objc public var scheduled: Bool = false
    @objc public var exact: Bool = false
    @objc public var notifications: Bool = false

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["scheduled": "scheduled", "exact": "exact", "notifications": "notifications"]
    }
}

@objc(TimerAlarmCancelParamModel)
public class TimerAlarmCancelParamModel: SPKMethodModel {
    @objc public var identifier: String?

    public override class func requiredKeyPaths() -> Set<String>? { ["id"] }

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["identifier": "id"]
    }
}

@objc(TimerAlarmCancelResultModel)
public class TimerAlarmCancelResultModel: SPKMethodModel {
    @objc public var ok: Bool = false

    @objc public override class func jsonKeyPathsByPropertyKey() -> [AnyHashable: Any] {
        ["ok": "ok"]
    }
}

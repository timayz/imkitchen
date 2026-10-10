package app.imkitchen.timeralarm

import app.imkitchen.timeralarm.cancel.AbsTimerAlarmCancelMethodIDL
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IDLBridgeMethod
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel
import com.tiktok.sparkling.method.runtime.depend.BridgeBaseRuntime

/** `TimerAlarm.cancel({id})`: drops the pending alarm and any notification it already posted. */
class TimerAlarmCancelMethod : AbsTimerAlarmCancelMethodIDL() {

    override fun handle(params: Params, callback: CompletionBlock<Result>, type: BridgePlatformType) {
        val context = BridgeBaseRuntime.applicationContext
            ?: return callback.onFailure(IDLBridgeMethod.FAIL, "Context not provided in host")
        val id = params.id
        if (id.isEmpty()) {
            return callback.onFailure(IDLBridgeMethod.INVALID_PARAM, "TimerAlarm.cancel: empty id")
        }
        try {
            TimerAlarms.cancel(context, id)
            callback.onSuccess(Result::class.java.createXModel().apply { ok = true })
        } catch (e: Exception) {
            callback.onFailure(IDLBridgeMethod.FAIL, "TimerAlarm.cancel failed: $e")
        }
    }
}

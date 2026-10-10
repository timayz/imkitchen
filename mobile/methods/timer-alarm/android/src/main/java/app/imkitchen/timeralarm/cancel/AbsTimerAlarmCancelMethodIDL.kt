package app.imkitchen.timeralarm.cancel

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `TimerAlarm.cancel`; see `TimerAlarms` for the semantics. */
abstract class AbsTimerAlarmCancelMethodIDL :
    AbsSparklingIDLMethod<AbsTimerAlarmCancelMethodIDL.Params, AbsTimerAlarmCancelMethodIDL.Result>() {

    @IDLMethodName(name = "TimerAlarm.cancel", params = ["id"], results = ["ok"])
    final override val name: String = "TimerAlarm.cancel"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "id")
        val id: String
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "ok")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "ok")
        var ok: Boolean?
    }
}

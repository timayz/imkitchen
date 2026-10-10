package app.imkitchen.timeralarm.schedule

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `TimerAlarm.schedule`; see `TimerAlarms` for the semantics. */
abstract class AbsTimerAlarmScheduleMethodIDL :
    AbsSparklingIDLMethod<AbsTimerAlarmScheduleMethodIDL.Params, AbsTimerAlarmScheduleMethodIDL.Result>() {

    @IDLMethodName(
        name = "TimerAlarm.schedule",
        params = ["id", "at", "title", "body"],
        results = ["scheduled", "exact", "notifications"]
    )
    final override val name: String = "TimerAlarm.schedule"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "id")
        val id: String

        /** Epoch milliseconds. */
        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "at")
        val at: Number

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "title")
        val title: String

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "body")
        val body: String
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "scheduled")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "scheduled")
        var scheduled: Boolean?

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "exact")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "exact")
        var exact: Boolean?

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "notifications")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "notifications")
        var notifications: Boolean?
    }
}

package app.imkitchen.timeralarm

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import app.imkitchen.timeralarm.schedule.AbsTimerAlarmScheduleMethodIDL
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import com.tiktok.sparkling.hybridkit.service.HybridActivityStackManager
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IDLBridgeMethod
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel
import com.tiktok.sparkling.method.runtime.depend.BridgeBaseRuntime

/**
 * `TimerAlarm.schedule({id, at, title, body})`: rings at `at` (epoch ms) even
 * if the device sleeps or the app dies in between. On API 33+ the first call
 * asks for the notification permission; the alarm is set either way and the
 * result's `notifications` tells the caller whether it will be visible.
 */
class TimerAlarmScheduleMethod : AbsTimerAlarmScheduleMethodIDL() {

    override fun handle(params: Params, callback: CompletionBlock<Result>, type: BridgePlatformType) {
        val context = BridgeBaseRuntime.applicationContext
            ?: return callback.onFailure(IDLBridgeMethod.FAIL, "Context not provided in host")
        val id = params.id
        val at = params.at.toLong()
        if (id.isEmpty() || at <= 0) {
            return callback.onFailure(IDLBridgeMethod.INVALID_PARAM, "TimerAlarm.schedule: empty id or time")
        }
        requestNotificationPermission(context)
        try {
            val scheduled = TimerAlarms.schedule(context, id, at, params.title, params.body)
            callback.onSuccess(
                Result::class.java.createXModel().apply {
                    this.scheduled = true
                    this.exact = scheduled.exact
                    this.notifications = NotificationManagerCompat.from(context).areNotificationsEnabled()
                }
            )
        } catch (e: Exception) {
            callback.onFailure(IDLBridgeMethod.FAIL, "TimerAlarm.schedule failed: $e")
        }
    }

    private fun requestNotificationPermission(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return
        val permission = Manifest.permission.POST_NOTIFICATIONS
        if (ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED) return
        // The host activity owns the result callback; the receiver re-checks
        // `areNotificationsEnabled` when the alarm fires, so nothing to wait for.
        HybridActivityStackManager.getTopActivity()?.let { activity ->
            activity.runOnUiThread { activity.requestPermissions(arrayOf(permission), REQUEST_CODE) }
        }
    }

    private companion object {
        const val REQUEST_CODE = 0x71A1
    }
}

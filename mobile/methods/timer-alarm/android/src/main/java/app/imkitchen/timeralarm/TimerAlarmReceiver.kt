package app.imkitchen.timeralarm

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Fired by `AlarmManager` when a cooking timer ends; posts the ringing notification. */
class TimerAlarmReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != TimerAlarms.ACTION_FIRE) return
        val id = intent.getStringExtra(TimerAlarms.EXTRA_ID) ?: return
        TimerAlarms.ring(
            context,
            id,
            intent.getStringExtra(TimerAlarms.EXTRA_TITLE).orEmpty(),
            intent.getStringExtra(TimerAlarms.EXTRA_BODY).orEmpty()
        )
    }
}

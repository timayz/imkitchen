package app.imkitchen.timeralarm

import android.app.AlarmManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.media.AudioAttributes
import android.media.AudioManager
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat

/**
 * The cooking timer's alarm: an `AlarmManager` wake-up that posts a
 * notification with the device's alarm ringtone, so the step still rings when
 * the screen is off, the app is in the background, or the process was killed.
 *
 * One pending alarm per `id` (the cooking screen uses `cooking:<recipe>:<step>`);
 * scheduling the same id again replaces the previous time.
 */
object TimerAlarms {
    /** Bumped whenever the channel's fixed settings (sound) change; the old one is deleted. */
    const val CHANNEL_ID = "cooking_timer_bell"
    private val RETIRED_CHANNELS = listOf("cooking_timer")
    const val ACTION_FIRE = "app.imkitchen.timeralarm.FIRE"
    const val EXTRA_ID = "id"
    const val EXTRA_TITLE = "title"
    const val EXTRA_BODY = "body"

    /** Outcome of [schedule]: `exact` is false when the OS only allowed an inexact alarm. */
    data class Scheduled(val exact: Boolean)

    fun schedule(context: Context, id: String, at: Long, title: String, body: String): Scheduled {
        ensureChannel(context)
        val alarms = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        val intent = pendingIntent(context, id, title, body)
        val exact = Build.VERSION.SDK_INT < Build.VERSION_CODES.S || alarms.canScheduleExactAlarms()
        if (exact) {
            // An alarm-clock alarm is the one kind the OS never defers: not
            // by Doze, not by app standby buckets, not by OEM battery savers
            // honouring the platform rules. `setExactAndAllowWhileIdle` is
            // only "allowed" while idle and rang late on a dozing phone.
            alarms.setAlarmClock(AlarmManager.AlarmClockInfo(at, launchIntent(context, id)), intent)
        } else {
            alarms.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, intent)
        }
        return Scheduled(exact)
    }

    fun cancel(context: Context, id: String) {
        val alarms = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        val intent = pendingIntent(context, id, null, null)
        alarms.cancel(intent)
        intent.cancel()
        NotificationManagerCompat.from(context).cancel(notificationId(id))
    }

    /** Posts the ringing notification; called by [TimerAlarmReceiver] when the alarm fires. */
    fun ring(context: Context, id: String, title: String, body: String) {
        ensureChannel(context)
        val manager = NotificationManagerCompat.from(context)
        if (!manager.areNotificationsEnabled()) return
        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_timer_alarm)
            .setContentTitle(title)
            .setContentText(body)
            .setCategory(NotificationCompat.CATEGORY_ALARM)
            .setPriority(NotificationCompat.PRIORITY_MAX)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            // Pre-Oreo devices have no channel: sound and vibration go on the notification itself.
            .setSound(alarmSound(context), AudioManager.STREAM_ALARM)
            .setVibrate(VIBRATION)
            .setAutoCancel(true)
            .setContentIntent(launchIntent(context, id))
            .build()
        // Keep ringing until the notification is dismissed or the shade opened: a
        // kitchen timer that plays its sound once and goes quiet is easy to miss.
        notification.flags = notification.flags or android.app.Notification.FLAG_INSISTENT
        try {
            manager.notify(notificationId(id), notification)
        } catch (_: SecurityException) {
            // POST_NOTIFICATIONS revoked between scheduling and firing: nothing to show.
        }
    }

    private val VIBRATION = longArrayOf(0, 500, 300, 500, 300, 500)

    /** Brings the app's task back to the front (the splash activity leaves it as it is). */
    private fun launchIntent(context: Context, id: String): PendingIntent? =
        context.packageManager.getLaunchIntentForPackage(context.packageName)?.let {
            it.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            PendingIntent.getActivity(
                context,
                notificationId(id),
                it,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
        }

    /** The app's own bell (`res/raw/cooking_timer.wav`, rendered by `scripts/timer-sound.mjs`). */
    private fun alarmSound(context: Context): Uri =
        Uri.parse("android.resource://${context.packageName}/${R.raw.cooking_timer}")

    private fun ensureChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        RETIRED_CHANNELS.forEach { manager.deleteNotificationChannel(it) }
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        val channel = NotificationChannel(
            CHANNEL_ID,
            context.getString(R.string.timer_alarm_channel_name),
            NotificationManager.IMPORTANCE_HIGH
        ).apply {
            description = context.getString(R.string.timer_alarm_channel_description)
            setSound(
                alarmSound(context),
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_ALARM)
                    .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                    .build()
            )
            enableVibration(true)
            vibrationPattern = VIBRATION
            lockscreenVisibility = NotificationCompat.VISIBILITY_PUBLIC
        }
        manager.createNotificationChannel(channel)
    }

    /**
     * `Intent.filterEquals` ignores extras, so the id goes into the data URI
     * to keep one PendingIntent per timer; the extras carry the text.
     */
    private fun pendingIntent(context: Context, id: String, title: String?, body: String?): PendingIntent {
        val intent = Intent(context, TimerAlarmReceiver::class.java)
            .setAction(ACTION_FIRE)
            .setData(Uri.fromParts("imkitchen-timer", id, null))
            .putExtra(EXTRA_ID, id)
        if (title != null) intent.putExtra(EXTRA_TITLE, title)
        if (body != null) intent.putExtra(EXTRA_BODY, body)
        return PendingIntent.getBroadcast(
            context,
            notificationId(id),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
    }

    private fun notificationId(id: String): Int = id.hashCode()
}

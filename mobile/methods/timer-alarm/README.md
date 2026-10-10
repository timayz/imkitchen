# sparkling-timer-alarm

Custom Sparkling Method behind the cooking screen's step timer: the countdown
itself is JavaScript, but the *ring* goes through the OS so it still happens
when the screen is off, the app is in the background, or the process was
killed. Same layout as `../keep-awake` (hand-written against the Sparkling IDL
annotations; `sparkling-method-cli codegen` 2.0.1 emits broken stubs).

- `TimerAlarm.schedule({id, at, title, body})` → `{scheduled, exact, notifications}`:
  ring at `at` (epoch milliseconds); the same `id` replaces a pending alarm.
- `TimerAlarm.cancel({id})` → `{ok}`.

Android (`app.imkitchen.timeralarm`): `AlarmManager.setExactAndAllowWhileIdle`
→ `TimerAlarmReceiver` → a `cooking_timer` channel notification with the
device's alarm ringtone (`TimerAlarms.kt`). The library manifest adds
`POST_NOTIFICATIONS` (asked at runtime on the first schedule),
`SCHEDULE_EXACT_ALARM` up to API 32 and `USE_EXACT_ALARM` from API 33: the
auto-granted exact-alarm permission that Play reserves for alarm/timer
features. If `canScheduleExactAlarms()` is false anyway the alarm is set
inexact (`exact: false` in the result) and may ring a few minutes late in
Doze. Registered in `SparklingApplication.kt`. Alarms do not survive a reboot.

iOS (`TimerAlarmMethods.swift`): `UNUserNotificationCenter` time-interval
request with the default sound; the first schedule prompts for permission.
`TimerAlarmCenter` is set as the center's delegate so the notification is
still shown and audible while the app is in the foreground. Auto-registered
(direct `PipeMethod` subclasses); pod `ios/Sparkling-TimerAlarm.podspec`.
Built by the `Mobile` workflow only.

Linked into the app with `npm install ./methods/timer-alarm` + `npm run autolink`.

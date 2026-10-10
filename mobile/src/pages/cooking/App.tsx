import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { type CookingScreen, getCooking } from '../../lib/api/kitchen.js'
import { nextStatus } from '../../lib/cooking.js'
import { formatClock, remainingSeconds } from '../../lib/countdown.js'
import { aisle, course } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { setKeepAwake } from '../../lib/keep-awake.js'
import { back, openExternal, pageParams } from '../../lib/nav.js'
import { enqueue } from '../../lib/offline/queue.js'
import { storageGet, storageRemove, storageSet } from '../../lib/storage.js'
import { cancelTimerAlarm, scheduleTimerAlarm } from '../../lib/timer-alarm.js'
import { useResource } from '../../lib/use-resource.js'
import { Button } from '../../ui/Button.js'
import { OfflineBanner } from '../../ui/OfflineBanner.js'
import { Spinner } from '../../ui/Spinner.js'

/**
 * Cooking mode: the ingredient list first, then one instruction per step with
 * an optional timer. Its own native container (`cooking.lynx.bundle?id=…`),
 * so the back gesture returns to the kitchen and the screen stays awake only
 * while it is up. The screen is cached locally and steps go through the
 * offline queue, so cooking keeps working without a connection.
 */
export function App() {
  const id = pageParams().id ?? ''
  const {
    data: screen,
    stale,
    offline,
    error,
    refresh,
  } = useResource<CookingScreen>(id ? `cooking:${id}` : null, () => getCooking(id), [id])
  const [checked, setChecked] = useState<Record<string, boolean>>({})
  const redirected = useRef(false)
  const moving = useRef(false)

  useEffect(() => {
    setKeepAwake(true)
    return () => setKeepAwake(false)
  }, [])

  // Nothing to show in-app: open the original instead. Only on fresh data, so
  // an offline launch never bounces out to the browser.
  useEffect(() => {
    if (!screen || stale || redirected.current) return
    if (screen.external_url && !screen.origin_embeddable) {
      redirected.current = true
      openExternal(screen.external_url)
      back()
    }
  }, [screen, stale])

  const move = useCallback(
    (direction: 'next' | 'prev') => {
      if (!screen || moving.current) return
      moving.current = true
      // Leaving the step is what ends its timer; closing the screen does not,
      // so a timer started here still rings and is restored when you return.
      const step = screen.steps.current
      if (step && step.time_next > 0) forgetTimer(timerId(id, step.index))
      const status = nextStatus(direction, screen.status, screen.recipe.instructions.length)
      // The queue patches the cached screen (re-rendering it) and sends the
      // absolute status when the server is reachable.
      void enqueue({ kind: 'status', id, status }).finally(() => {
        moving.current = false
      })
    },
    [screen, id]
  )

  if (!screen) {
    if (error) {
      return (
        <view className="screen cook cook--center">
          <view className="card">
            <text className="error">{error}</text>
            <Button label={t('common.retry')} onTap={refresh} variant="secondary" />
            <Button label={t('common.close')} onTap={back} variant="ghost" />
          </view>
        </view>
      )
    }
    return (
      <view className="screen cook cook--center">
        <Spinner size="lg" />
      </view>
    )
  }

  const { recipe } = screen

  const c = course(recipe.recipe_type)
  const current = screen.steps.current
  const total = recipe.instructions.length

  let eyebrow = ''
  if (screen.show_ingredients) eyebrow = t('cooking.ingredients', { n: recipe.ingredients.length })
  else if (current && screen.steps.coming.length === 0) eyebrow = t('cooking.final')
  else if (current) eyebrow = t('cooking.step_of', { n: current.index + 1, total })

  return (
    <view className="screen cook">
      <view className="cook__bar">
        <view className="cook__close" bindtap={back}>
          <text className="cook__close-text">✕</text>
        </view>
        <view className="cook__bar-center">
          <text className="cook__eyebrow">{eyebrow}</text>
          <text className="cook__title">{recipe.name}</text>
        </view>
        <view className="cook__spacer" />
      </view>
      {offline && <OfflineBanner onRetry={refresh} />}

      {screen.origin_embeddable && recipe.origin ? (
        <webview
          src={recipe.origin}
          style={{ width: '100%', height: `${(lynx.__globalProps.screenHeight ?? 800) - 120}px` }}
        />
      ) : screen.show_ingredients ? (
        <>
          <scroll-view className="cook__main" scroll-orientation="vertical">
            <view className="cook__aisles">
              {screen.ingredient_aisles.map((a) => {
                const s = aisle(a.key)
                return (
                  <view key={a.key} className="aisle">
                    <view className="aisle__head">
                      <view className="aisle__icon" style={{ backgroundColor: s.soft }}>
                        <text className="aisle__emoji">{s.emoji}</text>
                      </view>
                      <text className="aisle__label">{s.label}</text>
                    </view>
                    {a.items.map((i) => {
                      const on = checked[i.key] === true
                      return (
                        <view
                          key={i.key}
                          className="ing"
                          bindtap={() => setChecked((prev) => ({ ...prev, [i.key]: !on }))}
                        >
                          <view className={on ? 'ing__box ing__box--on' : 'ing__box'}>
                            {on && <text className="ing__check">✓</text>}
                          </view>
                          <text className={on ? 'ing__name ing__name--on' : 'ing__name'}>{i.name}</text>
                          <text className="ing__qty">{i.quantity_label}</text>
                        </view>
                      )
                    })}
                  </view>
                )
              })}
            </view>
          </scroll-view>
          <view className="cook__footer">
            <view className="cook__btn cook__btn--ink" bindtap={() => move('next')}>
              <text className="cook__btn-text cook__btn-text--light">{t('cooking.start')}</text>
            </view>
          </view>
        </>
      ) : current ? (
        <>
          <view className="cook__dots">
            {screen.steps.completed.map((s) => (
              <view key={`c${s.index}`} className="dot" style={{ backgroundColor: c.tint }} />
            ))}
            <view className="dot" style={{ backgroundColor: c.tint }} />
            {screen.steps.coming.map((s) => (
              <view key={`n${s.index}`} className="dot dot--off" />
            ))}
          </view>
          <view className="plate-wrap">
            <view className="plate" style={{ backgroundColor: c.soft }}>
              <text className="plate__emoji">{c.emoji}</text>
            </view>
            <view className="plate__badge">
              <text className="plate__badge-text" style={{ color: c.tint }}>
                {t('cooking.step', { n: current.index + 1 })}
              </text>
            </view>
          </view>
          <scroll-view className="cook__main" scroll-orientation="vertical">
            <text className="cook__instruction">{current.description}</text>
          </scroll-view>
          <view className="cook__footer">
            {current.time_next > 0 && (
              <Timer
                key={current.index}
                minutes={current.time_next}
                alarmId={timerId(id, current.index)}
                title={recipe.name}
                step={current.index + 1}
              />
            )}
            <view className="cook__nav">
              <view className="cook__btn cook__btn--paper" bindtap={() => move('prev')}>
                <text className="cook__btn-text">{t('cooking.back')}</text>
              </view>
              {screen.steps.coming.length === 0 ? (
                <view className="cook__btn cook__btn--primary cook__btn--wide" bindtap={back}>
                  <text className="cook__btn-text cook__btn-text--light">{t('cooking.done')}</text>
                </view>
              ) : (
                <view className="cook__btn cook__btn--ink cook__btn--wide" bindtap={() => move('next')}>
                  <text className="cook__btn-text cook__btn-text--light">{t('cooking.next')}</text>
                </view>
              )}
            </view>
          </view>
        </>
      ) : recipe.origin ? (
        <view className="cook--center cook__imported">
          <view className="card cook__imported-card">
            <text className="cook__eyebrow">{t('cooking.imported')}</text>
            <text className="body">{t('cooking.imported_hint')}</text>
            <Button label={t('cooking.open_original')} onTap={() => openExternal(recipe.origin!)} />
          </view>
        </view>
      ) : null}
    </view>
  )
}

/** One alarm and one stored deadline per recipe + step. */
function timerId(recipeId: string, step: number): string {
  return `cooking:${recipeId}:${step}`
}

/** Drops a step's alarm and remembered deadline (no-op when there is none). */
function forgetTimer(alarmId: string): void {
  cancelTimerAlarm(alarmId)
  void storageRemove(alarmId).catch(() => {})
}

/**
 * Countdown for the current step's `time_next`, start/pause on tap.
 *
 * It keeps a deadline instead of counting ticks (`lib/countdown.ts`), so a
 * throttled or suspended JS thread shows the right time as soon as it runs
 * again, and the ring itself is handed to the OS (`lib/timer-alarm.ts`): the
 * step still rings when the screen is off or the app is asleep. The deadline
 * is also written to native storage, so the countdown survives the process
 * being killed and is picked up again when the step is reopened. Pausing
 * cancels the alarm; moving to another step does too (see `move`); reaching
 * zero leaves the notification alone.
 */
function Timer({ minutes, alarmId, title, step }: { minutes: number; alarmId: string; title: string; step: number }) {
  const total = minutes * 60
  const [deadline, setDeadline] = useState<number | null>(null)
  const [remaining, setRemaining] = useState(total)
  const rang = useRef(false)
  const running = deadline !== null
  const done = !running && remaining === 0

  // A deadline left by an earlier visit: still ahead, resume; already past, it rang.
  useEffect(() => {
    let cancelled = false
    void storageGet<number | string>(alarmId)
      .then((stored) => {
        if (cancelled || stored === null) return
        const at = Number(stored)
        if (!Number.isFinite(at) || at <= 0) return
        if (at > Date.now()) {
          start(at)
        } else {
          rang.current = true
          setRemaining(0)
        }
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [alarmId])

  useEffect(() => {
    if (deadline === null) return
    const tick = () => {
      const left = remainingSeconds(deadline, Date.now())
      setRemaining(left)
      if (left === 0) {
        rang.current = true
        setDeadline(null)
      }
    }
    tick()
    const handle = setInterval(tick, 250)
    // Timers stall in the background; catch up the moment the screen is back.
    const emitter = lynx.getJSModule('GlobalEventEmitter')
    emitter.addListener('onShow', tick)
    emitter.addListener('onEnterForeground', tick)
    return () => {
      clearInterval(handle)
      emitter.removeListener('onShow', tick)
      emitter.removeListener('onEnterForeground', tick)
    }
  }, [deadline])

  const start = (at: number) => {
    rang.current = false
    setDeadline(at)
    scheduleTimerAlarm({ id: alarmId, at, title, body: t('cooking.timer_notify', { n: step }) })
    void storageSet(alarmId, at).catch(() => {})
  }

  const toggle = () => {
    if (deadline !== null) {
      setRemaining(remainingSeconds(deadline, Date.now()))
      setDeadline(null)
      forgetTimer(alarmId)
    } else if (done) {
      setRemaining(total)
      forgetTimer(alarmId)
    } else {
      start(Date.now() + remaining * 1000)
    }
  }

  const label = running ? t('cooking.timer_running') : done ? t('cooking.timer_done') : t('cooking.timer_ready')

  return (
    <view className={running ? 'timer timer--running' : done ? 'timer timer--done' : 'timer'}>
      <view className="timer__icon">
        <text className="timer__icon-text">⏲</text>
      </view>
      <view className="timer__body">
        <text className="timer__label">{label}</text>
        <text className="timer__display">{formatClock(remaining)}</text>
      </view>
      <view className="timer__toggle" bindtap={toggle}>
        <text className="timer__toggle-text">{running ? '❚❚' : done ? '↺' : '▶'}</text>
      </view>
    </view>
  )
}

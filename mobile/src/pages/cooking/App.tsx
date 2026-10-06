import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import { type CookingScreen, getCooking, step } from '../../lib/api/kitchen.js'
import { aisle, course } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { setKeepAwake } from '../../lib/keep-awake.js'
import { back, openExternal, pageParams } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'

type State =
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; screen: CookingScreen }

/**
 * Cooking mode: the ingredient list first, then one instruction per step with
 * an optional timer. Its own native container (`cooking.lynx.bundle?id=…`),
 * so the back gesture returns to the kitchen and the screen stays awake only
 * while it is up.
 */
export function App() {
  const id = pageParams().id ?? ''
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [busy, setBusy] = useState(false)
  const [checked, setChecked] = useState<Record<string, boolean>>({})

  const fail = (err: unknown) =>
    setState({ kind: 'error', message: err instanceof ApiError ? err.message : t('error.network') })

  useEffect(() => {
    setKeepAwake(true)
    getCooking(id)
      .then((screen) => {
        if (screen.external_url && !screen.origin_embeddable) {
          openExternal(screen.external_url)
          back()
          return
        }
        setState({ kind: 'ready', screen })
      })
      .catch(fail)
    return () => setKeepAwake(false)
  }, [id])

  const move = useCallback(
    async (direction: 'next' | 'prev') => {
      if (busy) return
      setBusy(true)
      try {
        const screen = await step(id, direction)
        setState({ kind: 'ready', screen })
      } catch (err) {
        fail(err)
      } finally {
        setBusy(false)
      }
    },
    [busy, id],
  )

  if (state.kind === 'loading') {
    return (
      <view className="screen cook cook--center">
        <text className="muted">{t('common.loading')}</text>
      </view>
    )
  }
  if (state.kind === 'error') {
    return (
      <view className="screen cook cook--center">
        <view className="card">
          <text className="error">{state.message}</text>
          <Button label={t('common.close')} onTap={back} variant="secondary" />
        </view>
      </view>
    )
  }

  const { screen } = state
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
            {current.time_next > 0 && <Timer key={current.index} minutes={current.time_next} />}
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

/** Countdown for the current step's `time_next`, start/pause on tap. */
function Timer({ minutes }: { minutes: number }) {
  const [seconds, setSeconds] = useState(minutes * 60)
  const [running, setRunning] = useState(false)
  const timer = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    if (!running) return
    timer.current = setInterval(() => {
      setSeconds((s) => {
        if (s <= 1) {
          setRunning(false)
          return 0
        }
        return s - 1
      })
    }, 1000)
    return () => {
      if (timer.current) clearInterval(timer.current)
    }
  }, [running])

  const mm = String(Math.floor(seconds / 60)).padStart(2, '0')
  const ss = String(seconds % 60).padStart(2, '0')

  return (
    <view className={running ? 'timer timer--running' : 'timer'}>
      <view className="timer__icon">
        <text className="timer__icon-text">⏲</text>
      </view>
      <view className="timer__body">
        <text className="timer__label">{running ? t('cooking.timer_running') : t('cooking.timer_ready')}</text>
        <text className="timer__display">
          {mm}:{ss}
        </text>
      </view>
      <view className="timer__toggle" bindtap={() => setRunning((r) => !r)}>
        <text className="timer__toggle-text">{running ? '❚❚' : '▶'}</text>
      </view>
    </view>
  )
}

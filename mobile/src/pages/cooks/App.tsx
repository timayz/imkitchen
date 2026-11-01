import { useCallback, useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import { errorMessage } from '../../lib/api/client.js'
import { type FollowedCook, getFollowedCooks } from '../../lib/api/recipes.js'
import { avatar } from '../../lib/avatar.js'
import { t } from '../../lib/i18n/index.js'
import { back, push } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { Spinner } from '../../ui/Spinner.js'
import './App.css'

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; cooks: FollowedCook[] }

/** The chefs the user follows: `cooks.lynx.bundle`. */
export function App() {
  const [state, setState] = useState<State>({ kind: 'loading' })

  const load = useCallback(() => {
    setState({ kind: 'loading' })
    getFollowedCooks()
      .then((cooks) => setState({ kind: 'ready', cooks }))
      .catch((err: unknown) => setState({ kind: 'error', message: errorMessage(err) }))
  }, [])

  // Reload when coming back from a cook page, where a follow may have changed.
  useEffect(() => {
    load()
    const onShow = () => load()
    lynx.getJSModule('GlobalEventEmitter').addListener('onShow', onShow)
    return () => lynx.getJSModule('GlobalEventEmitter').removeListener('onShow', onShow)
  }, [load])

  if (state.kind === 'loading') {
    return (
      <view className="screen cooks--center">
        <Spinner size="lg" />
      </view>
    )
  }
  if (state.kind === 'error') {
    return (
      <view className="screen cooks--center">
        <view className="card">
          <text className="error">{state.message}</text>
          <Button label={t('common.close')} onTap={back} variant="secondary" />
        </view>
      </view>
    )
  }

  const { cooks } = state

  return (
    <view className="screen">
      <scroll-view className="cooks__scroll" scroll-orientation="vertical">
        <view className="content">
          <view className="row" style={{ gap: '12px' }}>
            <view className="cook__close" bindtap={back}>
              <text className="cook__close-text">←</text>
            </view>
            <text className="rec__eyebrow">{t('cooks.count', { n: cooks.length }).toUpperCase()}</text>
          </view>
          <text className="cooks__title">{t('cooks.title')}</text>
          <text className="body">{t('cooks.subtitle')}</text>

          {cooks.length === 0 ? (
            <view className="card rec__empty">
              <view className="rec__empty-art" style={{ backgroundColor: '#fde3cf' }}>
                <text className="rec__empty-emoji">👩‍🍳</text>
              </view>
              <text className="rec__empty-title">{t('cooks.empty_title')}</text>
              <text className="body rec__center">{t('cooks.empty')}</text>
              <Button label={t('common.close')} onTap={back} variant="ink" block />
            </view>
          ) : (
            <view className="cooks__list">
              {cooks.map((cook) => {
                const a = avatar(cook.username)
                return (
                  <view
                    key={cook.username}
                    className="card cooks__row"
                    bindtap={() => void push('cook', { username: cook.username })}
                  >
                    <view className="cooks__avatar" style={{ backgroundColor: a.bg }}>
                      <text className="cooks__avatar-text" style={{ color: a.fg }}>
                        {a.initials}
                      </text>
                    </view>
                    <view className="cooks__identity">
                      <text className="cooks__name" text-maxline="1">
                        @{cook.username}
                      </text>
                      <text className="cooks__meta">
                        {t('recipes.shared_count', { n: cook.shared })} · {cook.follower}{' '}
                        {t('cook.followers').toLowerCase()}
                      </text>
                      {cook.description !== '' && (
                        <text className="body" text-maxline="2">
                          {cook.description}
                        </text>
                      )}
                    </view>
                    <text className="cooks__chevron">›</text>
                  </view>
                )
              })}
            </view>
          )}
        </view>
      </scroll-view>
    </view>
  )
}

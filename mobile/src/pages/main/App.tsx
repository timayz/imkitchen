import { useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { setUnauthorizedHandler } from '../../lib/api/client.js'
import { loadSession } from '../../lib/auth/session.js'
import { t } from '../../lib/i18n/index.js'
import { pageParams, replace } from '../../lib/nav.js'
import { drain, pendingCount, subscribePending } from '../../lib/offline/queue.js'
import { isOfflineNow, subscribeOffline } from '../../lib/offline/status.js'
import { BottomTabs, type Tab } from '../../ui/BottomTabs.js'
import { OfflineBanner } from '../../ui/OfflineBanner.js'
import { Spinner } from '../../ui/Spinner.js'
import groceriesIcon from '../../assets/icons/groceries.png'
import kitchenIcon from '../../assets/icons/kitchen.png'
import recipesIcon from '../../assets/icons/recipes.png'
import settingsIcon from '../../assets/icons/settings.png'
import { GroceriesTab } from './tabs/GroceriesTab.js'
import { KitchenTab } from './tabs/kitchen/KitchenTab.js'
import { RecipesTab } from './tabs/recipes/RecipesTab.js'
import { SettingsTab } from './tabs/settings/SettingsTab.js'

export type TabKey = 'kitchen' | 'groceries' | 'recipes' | 'settings'

/** How often to retry sending queued changes while some are waiting. */
const DRAIN_INTERVAL_MS = 30_000

/**
 * The app shell: one native container hosting the four tabs. Pushed screens
 * (recipe detail, cooking…) are separate bundles. The shell also owns the
 * offline banner and keeps draining the offline queue.
 */
export function App() {
  const [ready, setReady] = useState(false)
  const [tab, setTab] = useState<TabKey>((pageParams().tab as TabKey | undefined) ?? 'kitchen')
  // Tabs refetch when selected again and when the container comes back to
  // the foreground (e.g. returning from the cooking screen).
  const [refreshKey, setRefreshKey] = useState(0)
  const [offline, setOffline] = useState(isOfflineNow())
  const [pending, setPending] = useState(0)
  const bump = () => {
    setRefreshKey((k) => k + 1)
    void drain()
  }

  useEffect(() => {
    const emitter = lynx.getJSModule('GlobalEventEmitter')
    const onShow = () => bump()
    emitter.addListener('onShow', onShow)
    emitter.addListener('onEnterForeground', onShow)
    return () => {
      emitter.removeListener('onShow', onShow)
      emitter.removeListener('onEnterForeground', onShow)
    }
  }, [])

  // Queued changes are sent on launch, whenever the server becomes reachable
  // again, and on a timer while any are still waiting.
  useEffect(() => {
    const unsubscribeOffline = subscribeOffline((next) => {
      setOffline(next)
      if (!next) void drain()
    })
    const unsubscribePending = subscribePending(setPending)
    void pendingCount().then(setPending)
    void drain()
    return () => {
      unsubscribeOffline()
      unsubscribePending()
    }
  }, [])

  useEffect(() => {
    if (pending === 0) return
    const handle = setInterval(() => void drain(), DRAIN_INTERVAL_MS)
    return () => clearInterval(handle)
  }, [pending])

  const selectTab = (next: TabKey) => {
    if (next === tab) bump()
    else setTab(next)
  }

  useEffect(() => {
    setUnauthorizedHandler(() => {
      void replace('login')
    })
    loadSession().then((session) => {
      if (session) setReady(true)
      else void replace('login')
    })
    return () => setUnauthorizedHandler(null)
  }, [])

  if (!ready) {
    return (
      <view className="screen shell__loading">
        <Spinner size="lg" />
      </view>
    )
  }

  // Same order and icons as the web's mobile nav (templates/_user.html).
  const tabs: Tab<TabKey>[] = [
    { key: 'kitchen', label: t('tabs.kitchen'), icon: kitchenIcon },
    { key: 'recipes', label: t('tabs.recipes'), icon: recipesIcon },
    { key: 'groceries', label: t('tabs.groceries'), icon: groceriesIcon },
    { key: 'settings', label: t('tabs.settings'), icon: settingsIcon },
  ]

  return (
    <view className="screen shell">
      {(offline || pending > 0) && <OfflineBanner pending={pending} onRetry={bump} />}
      <view className="shell__body">
        {tab === 'kitchen' && <KitchenTab refreshKey={refreshKey} onAddRecipes={() => selectTab('recipes')} />}
        {tab === 'groceries' && <GroceriesTab refreshKey={refreshKey} onAddRecipes={() => selectTab('recipes')} />}
        {tab === 'recipes' && <RecipesTab refreshKey={refreshKey} />}
        {tab === 'settings' && <SettingsTab refreshKey={refreshKey} />}
      </view>
      <BottomTabs tabs={tabs} active={tab} onSelect={selectTab} />
    </view>
  )
}

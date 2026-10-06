import { useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { setUnauthorizedHandler } from '../../lib/api/client.js'
import { loadSession } from '../../lib/auth/session.js'
import { t } from '../../lib/i18n/index.js'
import { pageParams, replace } from '../../lib/nav.js'
import { BottomTabs, type Tab } from '../../ui/BottomTabs.js'
import groceriesIcon from '../../assets/icons/groceries.png'
import kitchenIcon from '../../assets/icons/kitchen.png'
import recipesIcon from '../../assets/icons/recipes.png'
import settingsIcon from '../../assets/icons/settings.png'
import { GroceriesTab } from './tabs/GroceriesTab.js'
import { KitchenTab } from './tabs/kitchen/KitchenTab.js'
import { RecipesTab } from './tabs/recipes/RecipesTab.js'
import { SettingsTab } from './tabs/settings/SettingsTab.js'

export type TabKey = 'kitchen' | 'groceries' | 'recipes' | 'settings'

/**
 * The app shell: one native container hosting the four tabs. Pushed screens
 * (recipe detail, cooking…) are separate bundles.
 */
export function App() {
  const [ready, setReady] = useState(false)
  const [tab, setTab] = useState<TabKey>((pageParams().tab as TabKey | undefined) ?? 'kitchen')
  // Tabs refetch when selected again and when the container comes back to
  // the foreground (e.g. returning from the cooking screen).
  const [refreshKey, setRefreshKey] = useState(0)
  const bump = () => setRefreshKey((k) => k + 1)

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
        <text className="muted">{t('common.loading')}</text>
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

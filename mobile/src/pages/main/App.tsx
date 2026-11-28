import { useState } from '@lynx-js/react';

import dishesIcon from '../../assets/icons/dishes.png';
import groceriesIcon from '../../assets/icons/groceries.png';
import listIcon from '../../assets/icons/list.png';
import recipesIcon from '../../assets/icons/recipes.png';
import settingsIcon from '../../assets/icons/settings.png';
import { type MessageKey, t } from '../../lib/i18n/index.js';
import { pageParams, push } from '../../lib/nav.js';
import { BottomTabs, type Tab } from '../../ui/BottomTabs.js';
import { Screen } from '../../ui/Screen.js';
import './App.css';

// The screens, in the order the web application's navigation gives them.
const TABS = ['recipes', 'dishes', 'list', 'groceries', 'settings'] as const;

export type TabKey = (typeof TABS)[number];

const ICONS: Record<TabKey, string> = {
  recipes: recipesIcon,
  dishes: dishesIcon,
  list: listIcon,
  groceries: groceriesIcon,
  settings: settingsIcon,
};

function initialTab(): TabKey {
  const tab = pageParams().tab as TabKey | undefined;
  return tab && TABS.includes(tab) ? tab : 'recipes';
}

/**
 * The app shell: one native container hosting the tabs, each a screen of its
 * own feature. Screens opened from a tab are separate bundles, pushed on top.
 */
export function App() {
  const [tab, setTab] = useState<TabKey>(initialTab);

  const tabs: Tab<TabKey>[] = TABS.map((key) => ({
    key,
    label: t(`tabs.${key}`),
    icon: ICONS[key],
  }));

  return (
    <Screen>
      <view className="shell__body">
        <scroll-view className="shell__scroll" scroll-orientation="vertical">
          <view className="content">
            <text className="h1">{t(`tabs.${tab}`)}</text>
            {tab === 'settings' ? <Settings /> : <Empty message={`${tab}.empty`} />}
          </view>
        </scroll-view>
      </view>
      <BottomTabs tabs={tabs} active={tab} onSelect={setTab} />
    </Screen>
  );
}

/** What a tab shows until its feature is built. */
function Empty({ message }: { message: MessageKey }) {
  return <text className="muted">{t(message)}</text>;
}

function Settings() {
  return (
    <view className="card">
      <view
        id="settings-about"
        className="row"
        accessibility-element={true}
        accessibility-label={t('settings.about')}
        accessibility-traits="button"
        bindtap={() => void push('about')}
      >
        <text className="body">{t('settings.about')}</text>
        <text className="muted">›</text>
      </view>
    </view>
  );
}

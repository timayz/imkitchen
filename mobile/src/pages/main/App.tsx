import { useCallback, useEffect, useState } from '@lynx-js/react';

import '../../styles/base.css';
import './App.css';
import { setUnauthorizedHandler } from '../../lib/api/client.js';
import {
  type TourDto,
  type TourStatus,
  advanceTour,
  completeTour,
  getTours,
  skipTour,
} from '../../lib/api/tours.js';
import { loadSession } from '../../lib/auth/session.js';
import { t } from '../../lib/i18n/index.js';
import { pageParams, replace } from '../../lib/nav.js';
import { drain, pendingCount, subscribePending } from '../../lib/offline/queue.js';
import { isOfflineNow, subscribeOffline } from '../../lib/offline/status.js';
import { type ActiveTour, type TourStep, openTour, pickTour } from '../../lib/tours.js';
import { BottomTabs, type Tab } from '../../ui/BottomTabs.js';
import { OfflineBanner } from '../../ui/OfflineBanner.js';
import { Spinner } from '../../ui/Spinner.js';
import { TourOverlay } from '../../ui/TourOverlay.js';
import groceriesIcon from '../../assets/icons/groceries.png';
import kitchenIcon from '../../assets/icons/kitchen.png';
import recipesIcon from '../../assets/icons/recipes.png';
import settingsIcon from '../../assets/icons/settings.png';
import { GroceriesTab } from './tabs/GroceriesTab.js';
import { KitchenTab } from './tabs/kitchen/KitchenTab.js';
import { RecipesTab } from './tabs/recipes/RecipesTab.js';
import { SettingsTab } from './tabs/settings/SettingsTab.js';

export type TabKey = 'kitchen' | 'groceries' | 'recipes' | 'settings';

/** How often to retry sending queued changes while some are waiting. */
const DRAIN_INTERVAL_MS = 30_000;

/**
 * The app shell: one native container hosting the four tabs. Pushed screens
 * (recipe detail, cooking…) are separate bundles. The shell also owns the
 * offline banner and keeps draining the offline queue.
 */
export function App() {
  const [ready, setReady] = useState(false);
  const [tab, setTab] = useState<TabKey>((pageParams().tab as TabKey | undefined) ?? 'kitchen');
  // Tabs refetch when selected again and when the container comes back to
  // the foreground (e.g. returning from the cooking screen).
  const [refreshKey, setRefreshKey] = useState(0);
  const [offline, setOffline] = useState(isOfflineNow());
  const [pending, setPending] = useState(0);
  const bump = () => {
    setRefreshKey((k) => k + 1);
    void drain();
  };

  useEffect(() => {
    const emitter = lynx.getJSModule('GlobalEventEmitter');
    const onShow = () => bump();
    emitter.addListener('onShow', onShow);
    emitter.addListener('onEnterForeground', onShow);
    return () => {
      emitter.removeListener('onShow', onShow);
      emitter.removeListener('onEnterForeground', onShow);
    };
  }, []);

  // Queued changes are sent on launch, whenever the server becomes reachable
  // again, and on a timer while any are still waiting.
  useEffect(() => {
    const unsubscribeOffline = subscribeOffline((next) => {
      setOffline(next);
      if (!next) void drain();
    });
    const unsubscribePending = subscribePending(setPending);
    void pendingCount().then(setPending);
    void drain();
    return () => {
      unsubscribeOffline();
      unsubscribePending();
    };
  }, []);

  useEffect(() => {
    if (pending === 0) return;
    const handle = setInterval(() => void drain(), DRAIN_INTERVAL_MS);
    return () => clearInterval(handle);
  }, [pending]);

  const selectTab = (next: TabKey) => {
    if (next === tab) bump();
    else setTab(next);
  };

  // Guided tours: the server says where the user is; the tab that matches
  // opens its tour once its content is on screen. Progress is posted as
  // the user moves (best effort) and mirrored locally so a finished tour
  // does not come back before the next refetch.
  const [tours, setTours] = useState<TourDto[] | null>(null);
  const [tour, setTour] = useState<ActiveTour | null>(null);
  const loadTours = useCallback(() => {
    getTours()
      .then((r) => setTours(r.tours))
      .catch(() => {});
  }, []);
  useEffect(() => {
    if (ready) loadTours();
  }, [ready, loadTours]);

  const mark = useCallback((id: string, status: TourStatus, step: number) => {
    setTours((prev) => prev && prev.map((t) => (t.id === id ? { ...t, status, step } : t)));
  }, []);
  const record = useCallback(
    (id: string, step: TourStep) => {
      mark(id, 'in_progress', step.index);
      void advanceTour(id, step.index).catch(() => {});
    },
    [mark]
  );

  useEffect(() => {
    if (!ready || !tours || tour) return;
    const pick = pickTour(tours, tab);
    if (!pick) return;
    let cancelled = false;
    void openTour(pick).then((opened) => {
      if (cancelled || !opened) return;
      setTour(opened);
      record(opened.id, opened.steps[opened.pos]!);
    });
    return () => {
      cancelled = true;
    };
    // Re-evaluated on tab change and resume only: a tour that just closed
    // waits for the next visit before the tab's next tour opens.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ready, tours === null, tab, refreshKey]);

  const tourNext = () => {
    if (!tour) return;
    if (tour.pos >= tour.steps.length - 1) {
      mark(tour.id, 'completed', tour.steps[tour.pos]!.index);
      void completeTour(tour.id).catch(() => {});
      setTour(null);
      return;
    }
    const pos = tour.pos + 1;
    setTour({ ...tour, pos });
    record(tour.id, tour.steps[pos]!);
  };
  const tourBack = () => {
    if (!tour || tour.pos === 0) return;
    const pos = tour.pos - 1;
    setTour({ ...tour, pos });
    record(tour.id, tour.steps[pos]!);
  };
  const tourSkip = () => {
    if (!tour) return;
    const step = tour.steps[tour.pos]!;
    mark(tour.id, 'skipped', step.index);
    void skipTour(tour.id, step.index).catch(() => {});
    setTour(null);
  };
  // The anchor vanished (list refreshed, row removed): drop the step.
  const tourMissing = () => {
    if (!tour) return;
    const steps = tour.steps.filter((_, i) => i !== tour.pos);
    if (steps.length === 0) {
      setTour(null);
      return;
    }
    const pos = Math.min(tour.pos, steps.length - 1);
    setTour({ ...tour, steps, pos });
    record(tour.id, steps[pos]!);
  };

  useEffect(() => {
    setUnauthorizedHandler(() => {
      void replace('login');
    });
    loadSession().then((session) => {
      if (session) setReady(true);
      else void replace('login');
    });
    return () => setUnauthorizedHandler(null);
  }, []);

  if (!ready) {
    return (
      <view className="screen shell__loading">
        <Spinner size="lg" />
      </view>
    );
  }

  // Same order and icons as the web's mobile nav (templates/_user.html).
  const tabs: Tab<TabKey>[] = [
    { key: 'kitchen', label: t('tabs.kitchen'), icon: kitchenIcon },
    { key: 'recipes', label: t('tabs.recipes'), icon: recipesIcon },
    { key: 'groceries', label: t('tabs.groceries'), icon: groceriesIcon },
    { key: 'settings', label: t('tabs.settings'), icon: settingsIcon },
  ];

  return (
    <view className="screen shell">
      {offline && <OfflineBanner pending={pending} onRetry={bump} />}
      <view className="shell__body">
        {tab === 'kitchen' && (
          <KitchenTab refreshKey={refreshKey} onAddRecipes={() => selectTab('recipes')} />
        )}
        {tab === 'groceries' && (
          <GroceriesTab refreshKey={refreshKey} onAddRecipes={() => selectTab('recipes')} />
        )}
        {tab === 'recipes' && <RecipesTab refreshKey={refreshKey} />}
        {tab === 'settings' && <SettingsTab refreshKey={refreshKey} onToursReset={loadTours} />}
      </view>
      <BottomTabs tabs={tabs} active={tab} onSelect={selectTab} />
      {tour && (
        <TourOverlay
          step={tour.steps[tour.pos]!}
          position={tour.pos}
          total={tour.steps.length}
          onNext={tourNext}
          onBack={tourBack}
          onSkip={tourSkip}
          onMissing={tourMissing}
        />
      )}
    </view>
  );
}

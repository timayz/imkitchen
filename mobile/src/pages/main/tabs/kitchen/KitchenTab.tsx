import { type ReactNode, useCallback, useEffect, useRef, useState } from '@lynx-js/react';

import { errorMessage } from '../../../../lib/api/client.js';
import { type Entry, type Overview, generate, getKitchen } from '../../../../lib/api/kitchen.js';
import { writeDoc } from '../../../../lib/cache.js';
import { course, minutes } from '../../../../lib/course.js';
import { t } from '../../../../lib/i18n/index.js';
import {
  type Item,
  dishTime,
  doneCount,
  isCooking,
  isDone,
  items,
} from '../../../../lib/kitchen-items.js';
import { push } from '../../../../lib/nav.js';
import { enqueue } from '../../../../lib/offline/queue.js';
import { useResource } from '../../../../lib/use-resource.js';
import { Button } from '../../../../ui/Button.js';
import { Spinner } from '../../../../ui/Spinner.js';
import { GenerateSheet } from './GenerateSheet.js';
import { PlatSheet } from './PlatSheet.js';
import './KitchenTab.css';

export interface KitchenTabProps {
  /** Bumped by the shell when the tab is (re)selected or the app resumes. */
  refreshKey: number;
  /** Switches the shell to the Recipes tab (onboarding "Add recipes"). */
  onAddRecipes: () => void;
}

export function KitchenTab({ refreshKey, onAddRecipes }: KitchenTabProps) {
  // Cached locally: renders at once and refreshes in the background.
  const { data, error, refresh } = useResource<Overview>('kitchen', getKitchen, [refreshKey]);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [sheet, setSheet] = useState(false);
  const [busy, setBusy] = useState(false);
  // The composed dish whose recipes are being picked from, by id: the sheet
  // reads the live item so statuses stay current while it is open.
  const [platId, setPlatId] = useState<string | null>(null);

  useEffect(() => {
    if (data) setOverview(data);
  }, [data]);

  const onGenerate = useCallback(async (count: number, household_size: number) => {
    setBusy(true);
    setNotice(null);
    try {
      const next = await generate(count, household_size);
      await writeDoc('kitchen', next);
      setSheet(false);
    } catch (err) {
      setNotice(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }, []);

  // Removing goes through the offline queue: the cached list drops the entry
  // at once and the server learns about it when reachable.
  const onRemove = useCallback((id: string) => {
    setNotice(null);
    void enqueue({ kind: 'list', id, in_list: false });
  }, []);

  const onRemoveDish = useCallback((id: string) => {
    setNotice(null);
    setPlatId(null);
    void enqueue({ kind: 'dish_remove', id });
  }, []);

  if (!overview) {
    if (error) {
      return (
        <view className="content">
          <view className="card">
            <text className="error">{error}</text>
            <Button label={t('common.retry')} onTap={refresh} variant="secondary" />
          </view>
        </view>
      );
    }
    return (
      <view className="content content--center">
        <Spinner size="lg" />
      </view>
    );
  }

  if (overview.kind === 'empty') {
    // One card, one action: the guided tour explains the rest.
    return (
      <scroll-view className="tab-scroll" scroll-orientation="vertical">
        <view className="content">
          {notice && <text className="error">{notice}</text>}
          <view id="kt-header" className="kt-header">
            <view className="kt-header__text">
              <text className="h1">{t('tabs.kitchen')}</text>
              <text className="muted">{t('kitchen.nothing')}</text>
            </view>
          </view>
          <view id="kt-empty" className="card kt-empty">
            <view className="kt-empty__art">
              <text className="kt-empty__emoji">👨‍🍳</text>
            </view>
            {overview.has_recipes ? (
              <>
                <text className="kt-empty__title">{t('kitchen.collection')}</text>
                <text className="body kt-center">{t('kitchen.empty_ready_hint')}</text>
                <view id="kt-cta" className="kt-empty__cta">
                  <Button
                    label={`✨ ${t('kitchen.generate')}`}
                    onTap={() => setSheet(true)}
                    variant="ink"
                    block
                  />
                </view>
                <Button label={t('kitchen.add_by_hand')} onTap={onAddRecipes} variant="ghost" />
                <Button
                  label={`🍽️ ${t('kitchen.dishes')}`}
                  onTap={() => void push('dishes')}
                  variant="ghost"
                />
              </>
            ) : (
              <>
                <text className="kt-empty__title">{t('kitchen.empty_caption')}</text>
                <text className="body kt-center">{t('kitchen.empty_first_hint')}</text>
                <view id="kt-cta" className="kt-empty__cta">
                  <Button
                    label={t('kitchen.add_first_recipe')}
                    onTap={onAddRecipes}
                    variant="ink"
                    block
                  />
                </view>
              </>
            )}
          </view>
        </view>
        <GenerateSheet
          open={sheet}
          busy={busy}
          remaining={0}
          onClose={() => setSheet(false)}
          onGenerate={onGenerate}
        />
      </scroll-view>
    );
  }

  const list = overview;
  const remaining = list.entries.filter((e) => e.status.status !== 'completed').length;
  const allCooked = list.total_count > 0 && list.completed_count === list.total_count;
  // One row per loose recipe or composed dish. Still to cook first, in the
  // server's order; cooked ones sink to the bottom.
  const all = items(list);
  const rows = [...all.filter((item) => !isDone(item)), ...all.filter(isDone)];
  const plat =
    all.find(
      (item): item is Item & { kind: 'dish' } => item.kind === 'dish' && item.id === platId
    ) ?? null;

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        {notice && <text className="error">{notice}</text>}
        {list.entries.length === 0 ? (
          <>
            <Header caption={t('kitchen.empty_caption')} />
            <Progress done={0} total={5} />
            <view id="kt-empty" className="card kt-empty">
              <view className="kt-empty__art">
                <text className="kt-empty__emoji">👨‍🍳</text>
              </view>
              <text className="kt-empty__title">{t('kitchen.nothing')}</text>
              <text className="body kt-center">{t('kitchen.nothing_hint')}</text>
              <Button
                label={`✨ ${t('kitchen.generate')}`}
                onTap={() => setSheet(true)}
                variant="ink"
                block
              />
              <Button label={t('kitchen.browse_recipes')} onTap={onAddRecipes} variant="ghost" />
            </view>
          </>
        ) : (
          <>
            <Header
              caption={t('kitchen.cooked_round', {
                done: list.completed_count,
                total: list.total_count,
              })}
              onRegenerate={() => setSheet(true)}
            />
            <Progress done={list.completed_count} total={list.total_count} />
            {allCooked && (
              <view className="kt-done">
                <text className="kt-done__text">✓ {t('kitchen.all_cooked')}</text>
              </view>
            )}
            <view id="kt-list" className="kt-section">
              <text className="kt-eyebrow">
                {t('kitchen.your_list')} · {rows.length}
              </text>
              <view className="kt-section__line" />
            </view>
            <text id="kt-hint" className="muted">
              {t('kitchen.swipe_hint')}
            </text>
            {rows.map((item, i) =>
              item.kind === 'dish' ? (
                <DishRow
                  key={`dish:${item.id}`}
                  id={i === 0 ? 'kt-row' : undefined}
                  item={item}
                  busy={busy}
                  onOpen={() => setPlatId(item.id)}
                  onRemove={() => onRemoveDish(item.id)}
                />
              ) : (
                <Row
                  key={item.entry.id}
                  id={i === 0 ? 'kt-row' : undefined}
                  entry={item.entry}
                  busy={busy}
                  onRemove={() => onRemove(item.entry.id)}
                />
              )
            )}
            {list.prep_ahead.length > 0 && (
              <view className="kt-prep">
                <text className="kt-prep__title">⏰ {t('kitchen.prep_ahead')}</text>
                {list.prep_ahead.map((e) => (
                  <text key={e.id} className="kt-prep__text">
                    <text className="kt-prep__name">{e.name}</text> · {e.advance_prep}
                  </text>
                ))}
              </view>
            )}
          </>
        )}
      </view>
      <GenerateSheet
        open={sheet}
        busy={busy}
        remaining={remaining}
        onClose={() => setSheet(false)}
        onGenerate={onGenerate}
      />
      <PlatSheet
        plat={plat}
        busy={busy}
        onClose={() => setPlatId(null)}
        onCook={(id) => {
          setPlatId(null);
          void push('cooking', { id });
        }}
        onRemove={() => plat && onRemoveDish(plat.id)}
      />
    </scroll-view>
  );
}

function Header({ caption, onRegenerate }: { caption: string; onRegenerate?: () => void }) {
  return (
    <view id="kt-header" className="kt-header">
      <view className="kt-header__text">
        <text className="h1">{t('kitchen.up_next')}</text>
        <text className="muted">{caption}</text>
      </view>
      <view id="kt-dishes" className="kt-dishes" bindtap={() => void push('dishes')}>
        <text className="kt-dishes__glyph">🍽️</text>
      </view>
      {onRegenerate && (
        <view id="kt-regen" className="kt-regen" bindtap={onRegenerate}>
          <text className="kt-regen__glyph">✨</text>
        </view>
      )}
    </view>
  );
}

/** One segment per meal in the round, filled once cooked. */
function Progress({ done, total }: { done: number; total: number }) {
  const segments = Array.from({ length: Math.max(total, 1) }, (_, i) => i < done);
  return (
    <view className="kt-progress">
      {segments.map((on, i) => (
        <view
          key={i}
          className={on ? 'kt-progress__seg kt-progress__seg--on' : 'kt-progress__seg'}
        />
      ))}
    </view>
  );
}

function Chip({ tone, text }: { tone: 'herb' | 'main'; text: string }) {
  return (
    <view className={`kt-chip kt-chip--${tone}`}>
      <text className={`kt-chip__text kt-chip__text--${tone}`}>{text}</text>
    </view>
  );
}

interface SwipeableProps {
  /** Anchor for the guided tour (first row only). */
  id?: string;
  /** The actions revealed by swiping left, right-aligned. */
  actions: ReactNode;
  /** How far a fully open row travels: the width of `actions`. */
  reveal: number;
  done: boolean;
  onTap: () => void;
  children: ReactNode;
}

/** Width of one action revealed by swiping a row left. */
const ACTION = 88;
/** Horizontal travel before a touch counts as a swipe rather than a tap. */
const SLOP = 8;

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

/**
 * A list row: tap it, or swipe left to reveal its actions. The offset is
 * plain state, so the row follows the finger through the background thread;
 * the snap at the end animates through CSS.
 */
function Swipeable({ id, actions, reveal, done, onTap, children }: SwipeableProps) {
  const [dx, setDx] = useState(0);
  const [open, setOpen] = useState(false);
  const [dragging, setDragging] = useState(false);
  const start = useRef<{ x: number; dx: number } | null>(null);
  const moved = useRef(false);

  const onTouchStart = (e: { touches: { clientX: number }[] }) => {
    const touch = e.touches[0];
    if (!touch) return;
    start.current = { x: touch.clientX, dx: open ? -reveal : 0 };
    moved.current = false;
  };
  const onTouchMove = (e: { touches: { clientX: number }[] }) => {
    const touch = e.touches[0];
    if (!touch || !start.current) return;
    const delta = touch.clientX - start.current.x;
    if (Math.abs(delta) > SLOP) {
      moved.current = true;
      setDragging(true);
    }
    setDx(clamp(start.current.dx + delta, -reveal, 0));
  };
  const onTouchEnd = () => {
    if (!start.current) return;
    start.current = null;
    const snap = dx < -reveal / 2;
    setOpen(snap);
    setDx(snap ? -reveal : 0);
    setDragging(false);
  };
  const tap = () => {
    // A swipe ends with a tap on some hosts; never treat it as a tap.
    if (moved.current) {
      moved.current = false;
      return;
    }
    if (open) {
      setOpen(false);
      setDx(0);
      return;
    }
    onTap();
  };

  const classes = ['kt-row'];
  if (done) classes.push('kt-row--done');
  if (!dragging) classes.push('kt-row--snap');

  return (
    <view id={id} className="kt-swipe">
      <view className="kt-swipe__actions">{actions}</view>
      <view
        className={classes.join(' ')}
        style={{ transform: `translateX(${dx}px)` }}
        bindtap={tap}
        bindtouchstart={onTouchStart}
        bindtouchmove={onTouchMove}
        bindtouchend={onTouchEnd}
        bindtouchcancel={onTouchEnd}
      >
        {children}
      </view>
    </view>
  );
}

interface RowProps {
  id?: string;
  entry: Entry;
  busy: boolean;
  onRemove: () => void;
}

/** A loose recipe: tap to cook it, swipe left to reveal "Recipe" and "Remove". */
function Row({ id, entry, busy, onRemove }: RowProps) {
  const c = course(entry.recipe_type);
  const done = entry.status.status === 'completed';
  const cooking = entry.status.status === 'cooking';

  return (
    <Swipeable
      id={id}
      reveal={ACTION * 2}
      done={done}
      // The cooking screen itself opens the original site for recipes that
      // cannot be cooked in-app.
      onTap={() => {
        if (!busy) void push('cooking', { id: entry.id });
      }}
      actions={
        <>
          <view
            className="kt-swipe__action kt-swipe__action--recipe"
            bindtap={() => void push('recipe', { id: entry.id })}
          >
            <text className="kt-swipe__glyph">📖</text>
            <text className="kt-swipe__text">{t('kitchen.recipe_short')}</text>
          </view>
          <view
            className="kt-swipe__action kt-swipe__action--remove"
            bindtap={busy ? undefined : onRemove}
          >
            <text className="kt-swipe__glyph">🗑</text>
            <text className="kt-swipe__text">{t('kitchen.remove_short')}</text>
          </view>
        </>
      }
    >
      <view className="kt-row__tile" style={{ backgroundColor: c.soft }}>
        <text className="kt-row__emoji">{c.emoji}</text>
      </view>
      <view className="kt-row__body">
        <text
          className={done ? 'kt-row__name kt-row__name--done' : 'kt-row__name'}
          text-maxline="1"
        >
          {entry.name}
        </text>
        <text className="kt-row__meta">
          {c.label} · {minutes(entry.total_time)}
        </text>
      </view>
      {done && <Chip tone="herb" text={`✓ ${t('kitchen.cooked_badge')}`} />}
      {cooking && <Chip tone="main" text={`🔥 ${t('kitchen.cooking')}`} />}
      <text className="kt-row__chevron">›</text>
    </Swipeable>
  );
}

interface DishRowProps {
  id?: string;
  item: Item & { kind: 'dish' };
  busy: boolean;
  onOpen: () => void;
  onRemove: () => void;
}

/** A composed dish: tap to pick which of its recipes to cook, swipe left to remove it whole. */
function DishRow({ id, item, busy, onOpen, onRemove }: DishRowProps) {
  const done = isDone(item);
  const cooking = isCooking(item);

  return (
    <Swipeable
      id={id}
      reveal={ACTION}
      done={done}
      onTap={() => {
        if (!busy) onOpen();
      }}
      actions={
        <view
          className="kt-swipe__action kt-swipe__action--remove"
          bindtap={busy ? undefined : onRemove}
        >
          <text className="kt-swipe__glyph">🗑</text>
          <text className="kt-swipe__text">{t('kitchen.remove_short')}</text>
        </view>
      }
    >
      <view className="kt-row__tile kt-row__tile--dish">
        <text className="kt-row__emoji">🍽️</text>
      </view>
      <view className="kt-row__body">
        <text className="kt-row__eyebrow">{t('kitchen.dish')}</text>
        <text
          className={done ? 'kt-row__name kt-row__name--done' : 'kt-row__name'}
          text-maxline="1"
        >
          {item.name}
        </text>
        <text className="kt-row__meta">
          {t('kitchen.dish_recipes', { n: item.entries.length })} ·{' '}
          {minutes(dishTime(item.entries))}
        </text>
      </view>
      {done && <Chip tone="herb" text={`✓ ${t('kitchen.cooked_badge')}`} />}
      {cooking && <Chip tone="main" text={`🔥 ${t('kitchen.cooking')}`} />}
      {!done && !cooking && (
        <text className="kt-row__count">
          {doneCount(item.entries)}/{item.entries.length}
        </text>
      )}
      <text className="kt-row__chevron">›</text>
    </Swipeable>
  );
}

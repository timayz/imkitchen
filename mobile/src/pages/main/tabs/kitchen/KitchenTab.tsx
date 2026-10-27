import { useCallback, useEffect, useRef, useState } from '@lynx-js/react';

import { errorMessage } from '../../../../lib/api/client.js';
import { type Entry, type Overview, generate, getKitchen } from '../../../../lib/api/kitchen.js';
import { writeDoc } from '../../../../lib/cache.js';
import { course, minutes } from '../../../../lib/course.js';
import { t } from '../../../../lib/i18n/index.js';
import { push } from '../../../../lib/nav.js';
import { enqueue } from '../../../../lib/offline/queue.js';
import { useResource } from '../../../../lib/use-resource.js';
import { Button } from '../../../../ui/Button.js';
import { Spinner } from '../../../../ui/Spinner.js';
import { GenerateSheet } from './GenerateSheet.js';
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

  useEffect(() => {
    if (data) setOverview(data);
  }, [data]);

  const onGenerate = useCallback(async (count: number) => {
    setBusy(true);
    setNotice(null);
    try {
      const next = await generate(count);
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
  // Still to cook first, in the server's order; cooked ones sink to the bottom.
  const rows = [
    ...list.entries.filter((e) => e.status.status !== 'completed'),
    ...list.entries.filter((e) => e.status.status === 'completed'),
  ];

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
            {rows.map((entry, i) => (
              <Row
                key={entry.id}
                id={i === 0 ? 'kt-row' : undefined}
                entry={entry}
                busy={busy}
                onRemove={() => onRemove(entry.id)}
              />
            ))}
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

interface RowProps {
  /** Anchor for the guided tour (first row only). */
  id?: string;
  entry: Entry;
  busy: boolean;
  onRemove: () => void;
}

/** Width of one action revealed by swiping a row left. */
const ACTION = 88;
/** Travel of a fully open row: the "Recipe" and "Remove" actions side by side. */
const REVEAL = ACTION * 2;
/** Horizontal travel before a touch counts as a swipe rather than a tap. */
const SLOP = 8;

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

/**
 * A list entry: tap to cook it, swipe left to reveal "Recipe" and "Remove".
 * The offset is plain state, so the row follows the finger through the
 * background thread; the snap at the end animates through CSS.
 */
function Row({ id, entry, busy, onRemove }: RowProps) {
  const c = course(entry.recipe_type);
  const done = entry.status.status === 'completed';
  const cooking = entry.status.status === 'cooking';
  const [dx, setDx] = useState(0);
  const [open, setOpen] = useState(false);
  const [dragging, setDragging] = useState(false);
  const start = useRef<{ x: number; dx: number } | null>(null);
  const moved = useRef(false);

  const onTouchStart = (e: { touches: { clientX: number }[] }) => {
    const touch = e.touches[0];
    if (!touch) return;
    start.current = { x: touch.clientX, dx: open ? -REVEAL : 0 };
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
    setDx(clamp(start.current.dx + delta, -REVEAL, 0));
  };
  const onTouchEnd = () => {
    if (!start.current) return;
    start.current = null;
    const snap = dx < -REVEAL / 2;
    setOpen(snap);
    setDx(snap ? -REVEAL : 0);
    setDragging(false);
  };
  const onTap = () => {
    // A swipe ends with a tap on some hosts; never treat it as a cook.
    if (moved.current) {
      moved.current = false;
      return;
    }
    if (open) {
      setOpen(false);
      setDx(0);
      return;
    }
    // The cooking screen itself opens the original site for recipes that
    // cannot be cooked in-app.
    if (!busy) void push('cooking', { id: entry.id });
  };

  const classes = ['kt-row'];
  if (done) classes.push('kt-row--done');
  if (!dragging) classes.push('kt-row--snap');

  return (
    <view id={id} className="kt-swipe">
      <view className="kt-swipe__actions">
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
      </view>
      <view
        className={classes.join(' ')}
        style={{ transform: `translateX(${dx}px)` }}
        bindtap={onTap}
        bindtouchstart={onTouchStart}
        bindtouchmove={onTouchMove}
        bindtouchend={onTouchEnd}
        bindtouchcancel={onTouchEnd}
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
      </view>
    </view>
  );
}

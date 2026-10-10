import { useEffect, useState } from '@lynx-js/react';

import { t } from '../lib/i18n/index.js';
import { type Rect, type TourStep, measure, scrollIntoView } from '../lib/tours.js';
import { Button } from './Button.js';
import './TourOverlay.css';

export interface TourOverlayProps {
  step: TourStep;
  /** Position among the steps shown (0-based) and how many there are. */
  position: number;
  total: number;
  onNext: () => void;
  onBack: () => void;
  onSkip: () => void;
  /** The step's anchor is gone from the screen. */
  onMissing: () => void;
}

/** Breathing room around the spotlighted element. */
const PAD = 8;
/** Space between the spotlight and the card. */
const GAP = 12;
const MARGIN = 12;
/** The card's height is not known before layout; this is the tallest one. */
const CARD_HEIGHT = 230;
const ROOT_ID = 'tour-root';

const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/**
 * Dims everything but one element and anchors a step card to it. Mounted
 * by the tab shell as the last child of the screen so it also covers the
 * tab bar; measures with Lynx UI methods, so every anchor needs an `id`.
 */
export function TourOverlay({
  step,
  position,
  total,
  onNext,
  onBack,
  onSkip,
  onMissing,
}: TourOverlayProps) {
  const [viewport, setViewport] = useState<Rect | null>(null);
  const [rect, setRect] = useState<Rect | null>(null);

  useEffect(() => {
    void measure(ROOT_ID).then((r) => r && setViewport(r));
  }, []);

  useEffect(() => {
    let cancelled = false;
    setRect(null);
    void (async () => {
      const box = await spotlight(step.targetIds, viewport);
      if (cancelled) return;
      if (box) setRect(box);
      else onMissing();
    })();
    return () => {
      cancelled = true;
    };
  }, [step, viewport]);

  const last = position === total - 1;
  const dots = Array.from({ length: total }, (_, i) => i);

  if (!viewport || !rect) return <view id={ROOT_ID} className="tour" />;

  const w = viewport.width;
  const h = viewport.height;
  const top = Math.max(0, rect.top - PAD);
  const left = Math.max(0, rect.left - PAD);
  const right = Math.min(w, rect.right + PAD);
  const bottom = Math.min(h, rect.bottom + PAD);
  const inset = lynx.__globalProps.bottomHeight ?? 0;

  // Below the spotlight when it fits, else above, else pinned to the bottom.
  const card =
    bottom + GAP + CARD_HEIGHT <= h - inset - MARGIN
      ? { top: `${bottom + GAP}px` }
      : top - GAP - CARD_HEIGHT >= MARGIN
        ? { top: `${top - GAP - CARD_HEIGHT}px` }
        : { bottom: `${MARGIN + inset}px` };

  const swallow = () => {};

  return (
    <view id={ROOT_ID} className="tour">
      <view
        className="tour__dim"
        style={{ top: '0px', left: '0px', right: '0px', height: `${top}px` }}
        catchtap={swallow}
      />
      <view
        className="tour__dim"
        style={{ top: `${bottom}px`, left: '0px', right: '0px', bottom: '0px' }}
        catchtap={swallow}
      />
      <view
        className="tour__dim"
        style={{ top: `${top}px`, left: '0px', width: `${left}px`, height: `${bottom - top}px` }}
        catchtap={swallow}
      />
      <view
        className="tour__dim"
        style={{ top: `${top}px`, left: `${right}px`, right: '0px', height: `${bottom - top}px` }}
        catchtap={swallow}
      />
      <view
        className="tour__ring"
        style={{
          top: `${top}px`,
          left: `${left}px`,
          width: `${right - left}px`,
          height: `${bottom - top}px`,
        }}
        catchtap={swallow}
      />
      <view className="tour__card" style={card}>
        <view className="tour__top">
          <text className="tour__count">
            {t('tour.step_of', { n: position + 1, m: total }).toUpperCase()}
          </text>
          <text className="tour__skip" bindtap={onSkip}>
            {t('tour.skip')}
          </text>
        </view>
        <text className="tour__title">{step.title}</text>
        <text className="tour__body">{step.body}</text>
        <view className="tour__foot">
          <view className="tour__dots">
            {dots.map((i) => (
              <view key={i} className={i === position ? 'tour__dot tour__dot--on' : 'tour__dot'} />
            ))}
          </view>
          <view className="tour__actions">
            {position > 0 && <Button label={t('tour.back')} onTap={onBack} variant="secondary" />}
            <Button label={last ? t('tour.done') : t('tour.next')} onTap={onNext} />
          </view>
        </view>
      </view>
    </view>
  );
}

/**
 * The box of the first candidate on screen. An anchor below the fold is
 * scrolled into view first; one that never measures reports as missing.
 */
async function spotlight(ids: string[], viewport: Rect | null): Promise<Rect | null> {
  for (let attempt = 0; attempt < 4; attempt++) {
    for (const id of ids) {
      let box = await measure(id);
      if (!box) continue;
      if (viewport && (box.top < 0 || box.bottom > viewport.height - 72)) {
        scrollIntoView(id);
        await wait(350);
        box = (await measure(id)) ?? box;
      }
      return box;
    }
    await wait(250);
  }
  return null;
}

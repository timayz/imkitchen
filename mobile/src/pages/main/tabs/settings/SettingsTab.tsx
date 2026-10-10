import { useCallback, useState } from '@lynx-js/react';

import { errorMessage } from '../../../../lib/api/client.js';
import { logout, me, type Me } from '../../../../lib/api/auth.js';
import {
  type General,
  type Preferences,
  type Session,
  deleteAccount,
  getGeneral,
  getSessions,
  requestAccountPasswordReset,
  revokeSession,
  setUsername,
  updateAisleOrder,
  updatePreferences,
  updateProfile,
} from '../../../../lib/api/settings.js';
import { resetTours } from '../../../../lib/api/tours.js';
import { clearSession } from '../../../../lib/auth/session.js';
import { writeDoc } from '../../../../lib/cache.js';
import { aisle, course } from '../../../../lib/course.js';
import { t } from '../../../../lib/i18n/index.js';
import { replace } from '../../../../lib/nav.js';
import { useResource } from '../../../../lib/use-resource.js';
import { Button } from '../../../../ui/Button.js';
import { Spinner } from '../../../../ui/Spinner.js';
import { TextField } from '../../../../ui/TextField.js';
import { AislesSheet } from './AislesSheet.js';
import { DeleteSheet } from './DeleteSheet.js';
import { DevicesSheet } from './DevicesSheet.js';
import { PreferencesSheet } from './PreferencesSheet.js';
import { ProfileSheet } from './ProfileSheet.js';
import './SettingsTab.css';

const GENERAL_KEY = 'settings:general';

type Open = 'preferences' | 'aisles' | 'profile' | 'devices' | 'delete' | null;

export interface SettingsTabProps {
  refreshKey: number;
  /** The tours were reset: the shell refetches them so they play again. */
  onToursReset?: () => void;
}

export function SettingsTab({ refreshKey, onToursReset }: SettingsTabProps) {
  // Cached locally: the tab renders at once every time it is selected and
  // refreshes in the background.
  const {
    data: user,
    error: userError,
    refresh: refreshUser,
  } = useResource<Me>('me', me, [refreshKey]);
  const {
    data: general,
    error: generalError,
    refresh: refreshGeneral,
  } = useResource<General>(GENERAL_KEY, getGeneral, [refreshKey]);
  const {
    data: sessions,
    error: sessionsError,
    refresh: refreshSessions,
  } = useResource<Session[]>('settings:sessions', getSessions, [refreshKey]);
  const [open, setOpen] = useState<Open>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<{ kind: 'ok' | 'error'; text: string } | null>(null);
  const [username, setUsernameInput] = useState('');
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const message = (err: unknown) => errorMessage(err);

  const reload = useCallback(() => {
    refreshUser();
    refreshGeneral();
    refreshSessions();
  }, [refreshUser, refreshGeneral, refreshSessions]);

  /** Runs one action, reports the outcome under the header and optionally closes the sheet. */
  const run = useCallback(
    async (
      action: () => Promise<void>,
      ok: string,
      opts: { reload?: boolean; close?: boolean } = {}
    ) => {
      if (busy) return;
      setBusy(true);
      setNotice(null);
      try {
        await action();
        setNotice({ kind: 'ok', text: ok });
        if (opts.close) setOpen(null);
        if (opts.reload) reload();
      } catch (err) {
        setNotice({ kind: 'error', text: message(err) });
      } finally {
        setBusy(false);
      }
    },
    [busy, reload]
  );

  // A saved change lands in the cached document, which is what the tab reads.
  const patchGeneral = async (p: Partial<General>) => {
    if (general) await writeDoc(GENERAL_KEY, { ...general, ...p });
  };

  const savePreferences = (draft: Preferences) =>
    run(
      async () => {
        await updatePreferences(draft);
        await patchGeneral(draft);
      },
      t('settings.preferences_saved'),
      { close: true }
    );

  const saveAisles = (order: string[]) =>
    run(
      async () => {
        await updateAisleOrder(order);
        await patchGeneral({ aisle_order: order });
      },
      t('settings.aisles_saved'),
      { close: true }
    );

  const saveProfile = (description: string) =>
    run(
      async () => {
        await updateProfile(description);
        await patchGeneral({ description });
      },
      t('settings.profile_saved'),
      { close: true }
    );

  const revokeOthers = (sessions: Session[]) =>
    run(
      async () => {
        for (const s of sessions) if (!s.current) await revokeSession(s.id);
      },
      t('settings.revoked_others'),
      { reload: true }
    );

  const signOut = useCallback(async () => {
    try {
      await logout();
    } catch {
      // The token is gone client-side either way.
    }
    await clearSession();
    await replace('login');
  }, []);

  const confirmDelete = useCallback(
    async (password: string) => {
      if (busy) return;
      setBusy(true);
      setDeleteError(null);
      try {
        await deleteAccount(password);
        // Every session is gone server-side; the 204 never trips the 401 handler.
        await clearSession();
        await replace('login');
      } catch (err) {
        setDeleteError(message(err));
        setBusy(false);
      }
    },
    [busy]
  );

  if (!user || !general || !sessions) {
    const error = userError ?? generalError ?? sessionsError;
    if (error) {
      return (
        <view className="content">
          <view className="card">
            <text className="error">{error}</text>
            <Button label={t('common.retry')} onTap={reload} variant="secondary" />
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

  const initial = (user.username ?? user.email).slice(0, 1).toUpperCase();
  const diets = general.dietary_restrictions.map((d) => t(`diet.${d}` as const));
  const courses = [course('MainCourse'), ...general.recipe_types.map(course)].map((c) => c.label);
  const pct = Math.round(general.cuisine_variety_weight * 100);
  const aisles = general.aisle_order.slice(0, 3).map((c) => aisle(`shopping_${c}`).label);

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        <view className="set-header">
          <text className="cook__eyebrow">{t('settings.eyebrow')}</text>
          <text className="h1">{t('settings.title')}</text>
        </view>

        {notice && (
          <text className={notice.kind === 'ok' ? 'success' : 'error'}>{notice.text}</text>
        )}

        {user.username ? (
          <view className="set-hero">
            <view className="set-hero__band">
              <view className="set-hero__avatar">
                <text className="set-hero__initial">{initial}</text>
              </view>
              <view className="set-hero__id">
                <text className="set-hero__name">{user.username}</text>
                <text className="set-hero__email">{user.email}</text>
              </view>
            </view>
            <view className="set-hero__body">
              {((user.premium_enabled && user.is_premium) || user.is_chef) && (
                <view className="row">
                  {user.premium_enabled && user.is_premium && (
                    <text className="badge badge--premium">{t('settings.premium')}</text>
                  )}
                  {user.is_chef && <text className="badge badge--chef">{t('settings.chef')}</text>}
                </view>
              )}
              {general.description.length > 0 && (
                <text className="set-hero__desc">{general.description}</text>
              )}
              <view className="set-hero__edit" bindtap={() => setOpen('profile')}>
                <text className="set-hero__edit-text">{t('settings.edit_profile')}</text>
                <text className="set-hero__edit-chevron">›</text>
              </view>
            </view>
          </view>
        ) : (
          <view className="set-hero">
            <view className="set-hero__band set-hero__band--plain">
              <view className="set-hero__avatar set-hero__avatar--plain">
                <text className="set-hero__initial set-hero__initial--plain">{initial}</text>
              </view>
              <view className="set-hero__id">
                <text className="set-hero__name">{user.email}</text>
                <text className="set-hero__email">{t('settings.signed_in')}</text>
              </view>
            </view>
            <view className="set-hero__body">
              <view className="set-note">
                <view className="set-note__icon">
                  <text className="set-note__glyph">✍️</text>
                </view>
                <text className="set-note__text">
                  <text className="set-note__strong">{t('settings.username_prompt')} </text>
                  {t('settings.username_hint')}
                </text>
              </view>
              <TextField
                label={t('settings.username')}
                value=""
                onChange={setUsernameInput}
                placeholder="chef_jane"
                maxlength={25}
              />
              <Button
                label={t('settings.username_set')}
                onTap={() =>
                  run(() => setUsername(username.trim()), t('settings.username_done'), {
                    reload: true,
                  })
                }
                disabled={busy || username.trim().length < 3}
                block
              />
            </view>
          </view>
        )}

        <Section label={t('settings.preferences')} />
        <view className="set-group">
          <Row
            id="set-household"
            emoji="👪"
            tone="entree"
            title={t('settings.household')}
            meta={
              general.household_size === 1
                ? t('settings.person')
                : t('settings.people', { n: general.household_size })
            }
            onTap={() => setOpen('preferences')}
          />
          <Row
            emoji="🌱"
            tone="herb"
            title={t('settings.diet')}
            meta={diets.length > 0 ? diets.join(' · ') : t('settings.none')}
            onTap={() => setOpen('preferences')}
          />
          <Row
            id="set-courses"
            emoji="🍰"
            tone="dessert"
            title={t('settings.courses')}
            meta={courses.join(' · ')}
            onTap={() => setOpen('preferences')}
          />
          <Row
            emoji="🌍"
            tone="condiment"
            title={t('settings.variety')}
            meta={t('settings.variety_meta', { pct })}
            onTap={() => setOpen('preferences')}
          />
          <Row
            id="set-aisles"
            emoji="🛒"
            tone="main"
            title={t('settings.aisles')}
            meta={t('settings.aisles_meta', { aisles: aisles.join(' › ') })}
            onTap={() => setOpen('aisles')}
            last
          />
        </view>

        <Section label={t('settings.tours')} />
        <view className="set-group">
          <Row
            emoji="🧭"
            tone="herb"
            title={t('settings.replay_tours')}
            meta={t('settings.replay_hint')}
            trailing="↻"
            onTap={() => run(resetTours, t('settings.tours_reset')).then(() => onToursReset?.())}
            last
          />
        </view>

        <Section label={t('settings.security')} />
        <view className="set-group">
          <Row
            emoji="🔑"
            tone="cream"
            title={t('settings.password')}
            meta={t('settings.password_reset')}
            trailing="✉️"
            onTap={() => run(requestAccountPasswordReset, t('settings.password_sent'))}
          />
          <Row
            emoji="📱"
            tone="entree"
            title={t('settings.sessions')}
            meta={
              sessions.length <= 1
                ? t('settings.devices_one')
                : t('settings.devices_many', { n: sessions.length })
            }
            onTap={() => setOpen('devices')}
          />
          <Row
            emoji="↪"
            tone="cream"
            title={t('settings.logout')}
            meta={t('settings.logout_hint')}
            trailing=""
            onTap={signOut}
            last
          />
        </view>

        <view className="set-row set-row--card" bindtap={() => setOpen('delete')}>
          <view className="set-row__tile set-row__tile--main">
            <text className="set-row__emoji">🗑️</text>
          </view>
          <view className="set-row__body">
            <text className="set-row__title set-row__title--danger">{t('settings.danger')}</text>
            <text className="set-row__meta">{t('settings.delete_meta')}</text>
          </view>
          <text className="set-row__chevron">›</text>
        </view>

        <text className="set-footer">{t('settings.billing_note')}</text>
      </view>

      <PreferencesSheet
        open={open === 'preferences'}
        busy={busy}
        value={{
          household_size: general.household_size,
          dietary_restrictions: general.dietary_restrictions,
          recipe_types: general.recipe_types,
          cuisine_variety_weight: general.cuisine_variety_weight,
        }}
        onClose={() => setOpen(null)}
        onSave={savePreferences}
      />
      <AislesSheet
        open={open === 'aisles'}
        busy={busy}
        value={general.aisle_order}
        onClose={() => setOpen(null)}
        onSave={saveAisles}
      />
      <ProfileSheet
        open={open === 'profile'}
        busy={busy}
        username={user.username ?? ''}
        description={general.description}
        onClose={() => setOpen(null)}
        onSave={saveProfile}
      />
      <DevicesSheet
        open={open === 'devices'}
        busy={busy}
        sessions={sessions}
        onClose={() => setOpen(null)}
        onRevoke={(id) => run(() => revokeSession(id), t('settings.revoked'), { reload: true })}
        onRevokeOthers={() => revokeOthers(sessions)}
      />
      <DeleteSheet
        open={open === 'delete'}
        busy={busy}
        error={deleteError}
        onClose={() => {
          setDeleteError(null);
          setOpen(null);
        }}
        onConfirm={confirmDelete}
      />
    </scroll-view>
  );
}

function Section({ label }: { label: string }) {
  return (
    <view className="set-section">
      <text className="set-eyebrow">{label.toUpperCase()}</text>
      <view className="set-section__line" />
    </view>
  );
}

type Tone = 'entree' | 'herb' | 'dessert' | 'condiment' | 'cream' | 'main';

interface RowProps {
  /** Anchor for the guided tour. */
  id?: string;
  emoji: string;
  tone: Tone;
  title: string;
  meta: string;
  /** Replaces the chevron: a glyph, or '' for nothing. */
  trailing?: string;
  onTap: () => void;
  last?: boolean;
}

function Row({ id, emoji, tone, title, meta, trailing, onTap, last }: RowProps) {
  return (
    <view id={id} className={last ? 'set-row set-row--last' : 'set-row'} bindtap={onTap}>
      <view className={`set-row__tile set-row__tile--${tone}`}>
        <text className="set-row__emoji">{emoji}</text>
      </view>
      <view className="set-row__body">
        <text className="set-row__title">{title}</text>
        <text className="set-row__meta">{meta}</text>
      </view>
      {trailing === undefined ? (
        <text className="set-row__chevron">›</text>
      ) : (
        trailing !== '' && <text className="set-row__glyph">{trailing}</text>
      )}
    </view>
  );
}

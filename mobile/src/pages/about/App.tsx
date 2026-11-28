import { APP_VERSION } from '../../lib/config.js';
import { hostProps, os } from '../../lib/device.js';
import { type MessageKey, t } from '../../lib/i18n/index.js';
import { back } from '../../lib/nav.js';
import { theme } from '../../lib/theme.js';
import { Screen } from '../../ui/Screen.js';

/** What the app is and what the host told it: `about.lynx.bundle`. */
export function App() {
  const props = hostProps();

  const rows: [MessageKey, string][] = [
    ['about.version', APP_VERSION],
    [
      'about.platform',
      [os() === 'ios' ? 'iOS' : 'Android', props.osVersion].filter(Boolean).join(' '),
    ],
    ['about.language', props.language || '—'],
    ['about.theme', t(`about.theme.${theme()}`)],
  ];

  return (
    <Screen>
      <view className="content">
        <text
          id="about-back"
          className="link"
          accessibility-element={true}
          accessibility-traits="button"
          bindtap={back}
        >
          {`‹ ${t('common.back')}`}
        </text>
        <text className="h1">{t('about.title')}</text>
        <view className="card">
          {rows.map(([label, value], index) => (
            <view key={label} className={index === 0 ? 'row' : 'row row--divided'}>
              <text className="body">{t(label)}</text>
              <text className="muted">{value}</text>
            </view>
          ))}
        </view>
      </view>
    </Screen>
  );
}

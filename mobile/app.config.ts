// @ts-nocheck
import { defineConfig } from '@lynx-js/rspeedy'
import { pluginQRCode } from '@lynx-js/qrcode-rsbuild-plugin'
import { pluginReactLynx } from '@lynx-js/react-rsbuild-plugin'
import type { AppConfig } from 'sparkling-app-cli'

/**
 * Where the app talks to. Every build targets the dev server on the host
 * unless told otherwise, so a stray build can never hit production:
 *   - Android emulator: http://10.0.2.2:3000 (the default)
 *   - USB device:       `adb reverse tcp:3000 tcp:3000` + http://localhost:3000
 *   - release:          `npm run build:release` (sets https://imkitchen.app)
 * Override with `IMKITCHEN_API_URL=…`.
 */
const apiUrl = process.env.IMKITCHEN_API_URL ?? 'http://10.0.2.2:3000'

const lynxConfig = defineConfig({
  source: {
    entry: {
      // One bundle per native container. `main` hosts the tab bar; pushed
      // screens get their own bundle so they keep the native back gesture.
      main: './src/pages/main/index.tsx',
      login: './src/pages/login/index.tsx',
      reset: './src/pages/reset/index.tsx',
      cooking: './src/pages/cooking/index.tsx',
      recipe: './src/pages/recipe/index.tsx',
      cook: './src/pages/cook/index.tsx',
      'recipe-edit': './src/pages/recipe-edit/index.tsx',
      'recipe-import': './src/pages/recipe-import/index.tsx',
    },
    define: {
      __API_BASE_URL__: JSON.stringify(apiUrl),
      __APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? '0.0.0'),
    },
  },
  output: {
    assetPrefix: 'asset:///',
    filename: {
      bundle: '[name].lynx.bundle',
    },
  },
  plugins: [
    pluginQRCode({
      schema(url: string): string {
        return `${url}?fullscreen=true`
      },
    }),
    pluginReactLynx(),
  ],
})

const config: AppConfig = {
  lynxConfig,
  appName: 'imkitchen',
  platform: {
    android: {
      packageName: 'app.imkitchen.android',
    },
    ios: {
      bundleIdentifier: 'app.imkitchen.android',
    },
  },
  paths: {
    androidAssets: 'android/app/src/main/assets',
    iosAssets: 'ios/SparklingGo/SparklingGo/Resources/Assets',
  },
  appIcon: './resource/app_icon.png',
  router: {
    main: {
      path: './lynxPages/main',
    },
  },
  plugin: [
    [
      'splash-screen',
      {
        // Matches the web app's cream surface (`--color-cream` in tailwind.css).
        backgroundColor: '#fbf5e9',
        image: './resource/splash_icon.png',
        dark: {
          image: './resource/splash_icon.png',
          backgroundColor: '#1b140c',
        },
        imageWidth: 160,
      },
    ],
  ],
}

export default config

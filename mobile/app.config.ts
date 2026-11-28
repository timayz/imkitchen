// @ts-nocheck
import { defineConfig } from '@lynx-js/rspeedy'
import { pluginQRCode } from '@lynx-js/qrcode-rsbuild-plugin'
import { pluginReactLynx } from '@lynx-js/react-rsbuild-plugin'
import type { AppConfig } from 'sparkling-app-cli'

const lynxConfig = defineConfig({
  source: {
    entry: {
      // One bundle per native container. `main` hosts the tab bar; a pushed
      // screen gets its own bundle so it keeps the native back gesture.
      // Keep in step with `Bundle` in src/lib/nav.ts.
      main: './src/pages/main/index.tsx',
      about: './src/pages/about/index.tsx',
    },
    define: {
      __APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? '0.0.0'),
    },
  },
  output: {
    assetPrefix: 'asset:///',
    // Lynx loads custom fonts from url()/data: only, so the Fraunces subset
    // referenced by src/styles/fonts.css is inlined as base64 whatever its size.
    dataUriLimit: { font: Number.MAX_SAFE_INTEGER },
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
      bundleIdentifier: 'app.imkitchen.ios',
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
        // `--color-paper` in src/styles/tokens.css, light and dark.
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

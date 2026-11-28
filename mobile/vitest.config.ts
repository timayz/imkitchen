// Copyright (c) 2025 TikTok Pte. Ltd.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.
import { defineConfig, mergeConfig } from 'vitest/config'
import { createVitestConfig } from '@lynx-js/react/testing-library/vitest-config'

const defaultConfig = await createVitestConfig()
const config = defineConfig({
  // What `source.define` in app.config.ts injects into the bundles.
  define: {
    __APP_VERSION__: JSON.stringify('test'),
  },
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'lcov', 'html'],
      exclude: [
        'node_modules/**',
        'dist/**',
        'vitest.config.ts',
        '**/*.d.ts',
        '**/*.config.*',
        '**/mockData/**',
        '**/tests/**'
      ]
    }
  },
})

export default mergeConfig(defaultConfig, config)

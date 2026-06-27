import { defineConfig, devices } from '@playwright/test'

const frontendUrl = 'http://127.0.0.1:4173'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI
    ? [['github'], ['html', { open: 'never' }]]
    : [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: frontendUrl,
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: [
    {
      command: 'mkdir -p data && cargo run --all-features',
      cwd: '../..',
      env: {
        AI_PROVIDER: 'local-deterministic',
        CORS_ALLOWED_ORIGINS: frontendUrl,
        DATABASE_URL: 'sqlite://data/e2e.db?mode=rwc',
        JWT_SECRET: 'intelliread-e2e-only-secret-32-characters',
        RUST_LOG: 'warn',
        SERVER_HOST: '127.0.0.1',
        SERVER_PORT: '3000',
      },
      reuseExistingServer: false,
      timeout: 300_000,
      url: 'http://127.0.0.1:3000/api/v1/health',
    },
    {
      command: 'npm run dev -- --host 127.0.0.1 --port 4173 --strictPort',
      env: {
        VITE_API_BASE_URL: 'http://127.0.0.1:3000/api/v1',
      },
      reuseExistingServer: false,
      timeout: 120_000,
      url: frontendUrl,
    },
  ],
})

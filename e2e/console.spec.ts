import { test, expect } from '@playwright/test';

test('Check for console errors', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      consoleErrors.push(msg.text());
    }
  });

  await page.goto('http://localhost:5173');
  await page.waitForLoadState('networkidle');

  expect(consoleErrors).toEqual([]);
});

import { expect, test } from '@playwright/test'

test('user can read, save an AI term, and complete a review', async ({ page }, testInfo) => {
  const accountSuffix = `${testInfo.workerIndex}-${testInfo.retry}-${Date.now()}`
  const username = `e2e-${accountSuffix}`

  await page.goto('/register')
  await page.getByLabel('用户名').fill(username)
  await page.getByLabel('邮箱').fill(`${username}@example.com`)
  await page.getByLabel('密码').fill('e2e-password')
  await page.getByRole('button', { name: '立即注册' }).click()

  await expect(page).toHaveURL('/')
  await expect(page.getByText(username, { exact: true })).toBeVisible()

  await page.getByLabel('文献标题（可选）').fill('E2E Machine Learning Notes')
  await page.locator('input[type="file"]').setInputFiles({
    name: 'e2e-machine-learning.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from(
      'A neural network model uses an algorithm and a dataset for evaluation.\n\nThe model improves performance when the algorithm learns useful feature representations.',
    ),
  })

  await expect(page).toHaveURL(/\/documents\/[^/]+$/)
  await expect(page.getByRole('heading', { name: 'E2E Machine Learning Notes' })).toBeVisible()

  await page.getByRole('button', { name: '分析文献' }).click()
  await expect(page.getByText(/Provider：local-deterministic/)).toBeVisible()

  const firstTerm = page.getByRole('button', { name: '加入生词' }).first()
  await expect(firstTerm).toBeVisible()
  await firstTerm.click()
  await expect(page.getByText(/已加入生词本：/)).toBeVisible()

  await page.getByRole('link', { name: '生词本' }).first().click()
  await expect(page).toHaveURL('/vocabulary')
  await expect(page.getByRole('heading', { name: '生词本' })).toBeVisible()
  await expect(page.getByText('共 1 条')).toBeVisible()

  await page.getByRole('link', { name: '开始复习' }).click()
  await expect(page).toHaveURL('/review')
  await expect(page.getByRole('heading', { name: '复习队列' })).toBeVisible()
  await page.getByRole('button', { name: '显示释义' }).click()
  await page.getByRole('button', { name: /记住了/ }).click()

  await expect(page.getByText('已记录：记住了')).toBeVisible()
  await expect(page.getByText('本轮复习已完成')).toBeVisible()
})

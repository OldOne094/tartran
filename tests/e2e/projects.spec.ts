import { test, expect, _electron as electron, type ElectronApplication, type Page } from '@playwright/test'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const appDir = join(__dirname, '..', '..')

async function launch(userData: string): Promise<ElectronApplication> {
  return electron.launch({
    args: [appDir],
    env: {
      ...process.env,
      TARTRAN_USER_DATA: userData,
      ELECTRON_DISABLE_SECURITY_WARNINGS: 'true'
    }
  })
}

test('project lifecycle: create, persist across relaunch, delete', async () => {
  const userData = mkdtempSync(join(tmpdir(), 'tartran-e2e-'))

  let app = await launch(userData)
  let page: Page = await app.firstWindow()
  await page.waitForLoadState('domcontentloaded')

  await page.getByRole('button', { name: 'New Project' }).click()
  await page.getByLabel('Novel title').fill('E2E Novel')
  await page.getByRole('button', { name: 'Create Project' }).click()
  await expect(page.getByText('E2E Novel')).toBeVisible({ timeout: 10_000 })
  await app.close()

  app = await launch(userData)
  page = await app.firstWindow()
  await page.waitForLoadState('domcontentloaded')
  await expect(page.getByText('E2E Novel')).toBeVisible({ timeout: 10_000 })

  await page.getByTitle('Delete').click()
  await page.getByRole('button', { name: 'Delete' }).last().click()
  await expect(page.getByText('E2E Novel')).not.toBeVisible({ timeout: 10_000 })
  await app.close()
})

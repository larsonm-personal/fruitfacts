const puppeteer = require('puppeteer-extra')

const {
  DEFAULT_INTERCEPT_RESOLUTION_PRIORITY,
  executablePath
} = require('puppeteer')
const AdblockerPlugin = require('puppeteer-extra-plugin-adblocker')
puppeteer.use(
  AdblockerPlugin({
    interceptResolutionPriority: DEFAULT_INTERCEPT_RESOLUTION_PRIORITY,
    blockTrackers: true,
    blockTrackersAndAnnoyances: true
  })
)

const args = process.argv.slice(2)

const USAGE_STRING = 'node index.js [web address to save] [screenshot path]'
if (args.length < 2) {
  console.log(USAGE_STRING)
  process.exit(1)
}
const web_address = args[0]
const output_path = args[1]
const siteUrl = new URL(web_address)

function isArsUsdaPage() {
  return siteUrl.hostname.endsWith('ars.usda.gov')
}

async function clickFirstVisible(page, selectors) {
  for (const selector of selectors) {
    const elements = await page.$$(selector)
    for (const element of elements) {
      const box = await element.boundingBox()
      if (box && box.width > 0 && box.height > 0) {
        await element.click({ delay: 20 }).catch(() => {})
        return true
      }
    }
  }

  return false
}

async function clickFirstMatchingText(page, patterns) {
  await page.evaluate(patternsText => {
    const patterns = patternsText.map(pattern => new RegExp(pattern, 'i'))

    const clickableTags = new Set(['A', 'BUTTON', 'DIV', 'SPAN'])
    const visible = element => {
      const style = window.getComputedStyle(element)
      const rect = element.getBoundingClientRect()
      return (
        style.display !== 'none' &&
        style.visibility !== 'hidden' &&
        rect.width > 0 &&
        rect.height > 0
      )
    }

    const elements = Array.from(document.querySelectorAll('body *')).reverse()

    for (const element of elements) {
      if (!clickableTags.has(element.tagName) && element.getAttribute('role') !== 'button') {
        continue
      }

      const text = [
        element.innerText,
        element.value,
        element.getAttribute('aria-label'),
        element.getAttribute('title')
      ]
        .filter(Boolean)
        .join(' ')

      if (visible(element) && patterns.some(pattern => pattern.test(text))) {
        element.dispatchEvent(
          new MouseEvent('click', {
            bubbles: true,
            cancelable: true,
            view: window
          })
        )
        return
      }
    }
  }, patterns.map(pattern => pattern.source))
}

async function dismissUsdaArsPopup(page) {
  const clickedSelector = await clickFirstVisible(page, [
    '#prefix-dismissButton',
    '.prefix-overlay-action-dismiss',
    '.prefix-overlay-close',
    '[aria-label*="close" i]',
    '[title*="close" i]'
  ])
  if (clickedSelector) {
    return
  }

  await clickFirstMatchingText(page, [
    /no thanks/,
    /remind me later/,
    /close subscription dialog/
  ])
}

async function waitForImages(page, timeoutMs) {
  await page.evaluate(timeout => {
    return new Promise(resolve => {
      const pending = Array.from(document.images).filter(image => !image.complete)
      if (pending.length === 0) {
        resolve()
        return
      }

      let remaining = pending.length
      const timer = setTimeout(resolve, timeout)
      const markDone = () => {
        remaining -= 1
        if (remaining <= 0) {
          clearTimeout(timer)
          resolve()
        }
      }

      for (const image of pending) {
        image.addEventListener('load', markDone, { once: true })
        image.addEventListener('error', markDone, { once: true })
      }
    })
  }, timeoutMs)
}

async function hideBrokenImages(page) {
  await page.evaluate(() => {
    for (const image of document.images) {
      const rect = image.getBoundingClientRect()
      if (image.complete && image.naturalWidth === 0 && rect.width > 0 && rect.height > 0) {
        image.style.display = 'none'
      }
    }
  })
}

puppeteer
  .launch({
    headless: true,
    ignoreHTTPSErrors: true,
    executablePath: executablePath(),
    args: ['--disable-notifications']
  })
  .then(async browser => {
    const page = await browser.newPage()
    page.setViewport({ width: 800, height: 1200 })
    page.setDefaultNavigationTimeout(60 * 1000) // ms, longer timeout for wayback machine stuff
    await page.goto(web_address)

    await page.waitForTimeout(3 * 1000)
    if (isArsUsdaPage()) {
      await dismissUsdaArsPopup(page)
    }
    await page.waitForTimeout(1 * 1000)
    if (isArsUsdaPage()) {
      await dismissUsdaArsPopup(page)
    }
    await waitForImages(page, 8 * 1000)
    if (isArsUsdaPage()) {
      await hideBrokenImages(page)
    }
    await page.waitForTimeout(1 * 1000)
    await page.screenshot({
      type: 'jpeg',
      quality: 75,
      path: output_path,
      fullPage: false
    })

    console.log(`puppeteer finished`)
    await browser.close()
  })

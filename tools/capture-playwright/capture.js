#!/usr/bin/env node
/**
 * CrawlWe Capture - Playwright
 *
 * Usage: node capture.js <url> [output-dir]
 */

import { chromium } from 'playwright';
import fs from 'fs/promises';
import path from 'path';

const url = process.argv[2];
const outputDir = process.argv[3] || './capture';

if (!url) {
  console.log('Usage: node capture.js <url> [output-dir]');
  process.exit(1);
}

console.log(`\n  CrawlWe Capture\n  ===============\n  URL: ${url}\n`);

async function capture() {
  await fs.mkdir(path.join(outputDir, 'data'), { recursive: true });

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
  });

  const page = await context.newPage();

  // Navigate and wait for network idle
  console.log('  Loading...');
  await page.goto(url, { waitUntil: 'networkidle', timeout: 60000 });

  // Wait for fonts
  await page.evaluate(() => document.fonts.ready);

  // Scroll for lazy content
  console.log('  Scrolling...');
  await page.evaluate(async () => {
    const delay = ms => new Promise(r => setTimeout(r, ms));
    for (let y = 0; y < document.body.scrollHeight; y += 500) {
      window.scrollTo(0, y);
      await delay(100);
    }
    window.scrollTo(0, 0);
  });
  await page.waitForTimeout(500);

  // Extract
  console.log('  Extracting...');
  const title = await page.title();
  const html = await page.content();

  // Get CSS with all sources
  const css = await page.evaluate(async () => {
    let result = '';

    for (const sheet of document.styleSheets) {
      const src = sheet.href || 'inline';
      try {
        if (sheet.cssRules) {
          result += `/* === ${src} === */\n`;
          for (const rule of sheet.cssRules) {
            result += rule.cssText + '\n';
          }
          result += '\n';
        }
      } catch {
        // CORS - fetch it
        if (sheet.href) {
          try {
            const resp = await fetch(sheet.href);
            if (resp.ok) {
              result += `/* === ${sheet.href} === */\n`;
              result += await resp.text();
              result += '\n\n';
            }
          } catch {}
        }
      }
    }

    // Inline styles
    for (const el of document.querySelectorAll('style')) {
      if (el.textContent?.trim()) {
        result += '/* === inline style === */\n' + el.textContent + '\n\n';
      }
    }

    // CSS variables
    const root = getComputedStyle(document.documentElement);
    let vars = '';
    for (let i = 0; i < root.length; i++) {
      if (root[i].startsWith('--')) {
        vars += `  ${root[i]}: ${root.getPropertyValue(root[i])};\n`;
      }
    }
    if (vars) result += '/* === :root variables === */\n:root {\n' + vars + '}\n';

    return result;
  });

  // Screenshot
  console.log('  Screenshot...');
  await page.screenshot({
    path: path.join(outputDir, 'screenshot.png'),
    fullPage: true
  });

  // Save
  console.log('  Saving...');

  const cleanHtml = `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${title}</title>
  <link rel="stylesheet" href="styles.css">
</head>
${html.match(/<body[\s\S]*<\/body>/i)?.[0] || '<body></body>'}
</html>`;

  await Promise.all([
    fs.writeFile(path.join(outputDir, 'index.html'), cleanHtml),
    fs.writeFile(path.join(outputDir, 'styles.css'), css),
    fs.writeFile(path.join(outputDir, 'data', 'raw.html'), html),
    fs.writeFile(path.join(outputDir, 'data', 'meta.json'), JSON.stringify({
      url, title,
      captured: new Date().toISOString(),
      bytes: { html: html.length, css: css.length }
    }, null, 2))
  ]);

  await browser.close();

  console.log(`\n  Done: ${outputDir}/`);
  console.log(`  - index.html (${(cleanHtml.length/1024).toFixed(0)}KB)`);
  console.log(`  - styles.css (${(css.length/1024).toFixed(0)}KB)`);
  console.log(`  - screenshot.png\n`);
}

capture().catch(e => {
  console.error('Error:', e.message);
  process.exit(1);
});

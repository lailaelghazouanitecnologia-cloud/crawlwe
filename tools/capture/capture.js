#!/usr/bin/env node
/**
 * CrawlWe Capture - Puppeteer-based web page capture
 *
 * Usage: node capture.js <url> [output-dir]
 * Example: node capture.js https://www.adaline.ai/ ./output
 */

import puppeteer from 'puppeteer';
import fs from 'fs/promises';
import path from 'path';

const url = process.argv[2];
const outputDir = process.argv[3] || './capture';

if (!url) {
  console.error('Usage: node capture.js <url> [output-dir]');
  process.exit(1);
}

console.log('CrawlWe Capture (Puppeteer)');
console.log('===========================');
console.log(`URL: ${url}`);
console.log(`Output: ${outputDir}`);
console.log();

async function capture() {
  // Create output directories
  await fs.mkdir(outputDir, { recursive: true });
  await fs.mkdir(path.join(outputDir, 'data'), { recursive: true });

  // Launch browser
  console.log('Launching browser...');
  const browser = await puppeteer.launch({
    headless: 'new',
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--disable-dev-shm-usage',
      '--disable-gpu'
    ]
  });

  const page = await browser.newPage();

  // Set viewport
  await page.setViewport({ width: 1920, height: 1080 });

  // Navigate to page
  console.log('Loading page...');
  await page.goto(url, {
    waitUntil: 'networkidle0',
    timeout: 60000
  });

  // Wait for fonts
  console.log('Waiting for fonts...');
  await page.evaluate(() => document.fonts.ready);

  // Scroll to trigger lazy loading
  console.log('Scrolling for lazy content...');
  await autoScroll(page);

  // Wait a bit for any animations
  await new Promise(r => setTimeout(r, 1000));

  // Get page title
  const title = await page.title();
  console.log(`Title: ${title}`);

  // Get rendered HTML
  console.log('\nExtracting content...');
  const html = await page.content();
  console.log(`  HTML: ${html.length} bytes`);

  // Get all CSS (including CORS-protected)
  const css = await page.evaluate(getAllCSS);
  const corsStyles = await fetchCORSStylesheets(page);
  const fullCss = css + '\n' + corsStyles;
  console.log(`  CSS: ${fullCss.length} bytes`);

  // Get external resources info
  const resources = await page.evaluate(getResources);
  console.log(`  External scripts: ${resources.scripts.length}`);
  console.log(`  External styles: ${resources.styles.length}`);

  // Take screenshot
  console.log('\nTaking screenshot...');
  await page.screenshot({
    path: path.join(outputDir, 'screenshot.png'),
    fullPage: true
  });

  // Save files
  console.log('\nSaving files...');

  // Clean HTML with stylesheet link
  const cleanHtml = generateCleanHtml(html, title);
  await fs.writeFile(path.join(outputDir, 'index.html'), cleanHtml);
  console.log('  index.html');

  // Raw HTML
  await fs.writeFile(path.join(outputDir, 'data', 'raw.html'), html);
  console.log('  data/raw.html');

  // CSS
  await fs.writeFile(path.join(outputDir, 'styles.css'), fullCss);
  console.log('  styles.css');

  // Metadata
  const metadata = {
    url,
    title,
    captured_at: new Date().toISOString(),
    stats: {
      html_bytes: html.length,
      css_bytes: fullCss.length,
      external_scripts: resources.scripts.length,
      external_styles: resources.styles.length
    },
    resources
  };
  await fs.writeFile(
    path.join(outputDir, 'data', 'metadata.json'),
    JSON.stringify(metadata, null, 2)
  );
  console.log('  data/metadata.json');

  await browser.close();
  console.log(`\nDone! Output: ${outputDir}`);
}

async function fetchCORSStylesheets(page) {
  // Get list of CORS-protected stylesheets
  const corsUrls = await page.evaluate(() => {
    const urls = [];
    for (const sheet of document.styleSheets) {
      try {
        // Try to access rules - will throw if CORS
        sheet.cssRules;
      } catch (e) {
        if (sheet.href) urls.push(sheet.href);
      }
    }
    return urls;
  });

  if (corsUrls.length === 0) return '';

  console.log(`  Fetching ${corsUrls.length} CORS stylesheets...`);

  // Fetch each CORS stylesheet
  const results = await Promise.all(
    corsUrls.map(async (url) => {
      try {
        const css = await page.evaluate(async (styleUrl) => {
          const resp = await fetch(styleUrl);
          if (resp.ok) return await resp.text();
          return '';
        }, url);
        return `/* Source: ${url} */\n${css}\n`;
      } catch (e) {
        return `/* Failed to fetch: ${url} */\n`;
      }
    })
  );

  return results.join('\n');
}

async function autoScroll(page) {
  await page.evaluate(async () => {
    await new Promise((resolve) => {
      let totalHeight = 0;
      const distance = 400;
      const timer = setInterval(() => {
        const scrollHeight = document.body.scrollHeight;
        window.scrollBy(0, distance);
        totalHeight += distance;
        if (totalHeight >= scrollHeight) {
          clearInterval(timer);
          window.scrollTo(0, 0);
          resolve();
        }
      }, 100);
    });
  });
}

function getAllCSS() {
  let css = '';
  const fetchedUrls = new Set();

  // Get from stylesheets
  for (const sheet of document.styleSheets) {
    try {
      if (sheet.cssRules) {
        const source = sheet.href || 'inline';
        css += `/* Source: ${source} */\n`;
        for (const rule of sheet.cssRules) {
          css += rule.cssText + '\n';
        }
        css += '\n';
        if (sheet.href) fetchedUrls.add(sheet.href);
      }
    } catch (e) {
      // CORS - will fetch separately
      if (sheet.href) {
        css += `/* CORS: ${sheet.href} - fetched separately */\n`;
      }
    }
  }

  // Get inline styles
  const styleTags = document.querySelectorAll('style');
  for (const style of styleTags) {
    if (style.textContent?.trim()) {
      css += '/* Inline <style> */\n';
      css += style.textContent + '\n\n';
    }
  }

  // Get computed CSS variables
  const root = document.documentElement;
  const rootStyles = getComputedStyle(root);
  let rootVars = ':root {\n';
  let hasVars = false;
  for (let i = 0; i < rootStyles.length; i++) {
    const prop = rootStyles[i];
    if (prop.startsWith('--')) {
      rootVars += `  ${prop}: ${rootStyles.getPropertyValue(prop)};\n`;
      hasVars = true;
    }
  }
  rootVars += '}\n';
  if (hasVars) {
    css += '\n/* Computed CSS Variables */\n' + rootVars;
  }

  return css;
}

function getResources() {
  const scripts = Array.from(document.scripts)
    .filter(s => s.src)
    .map(s => s.src);

  const styles = Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
    .filter(l => l.href)
    .map(l => l.href);

  return { scripts, styles };
}

function generateCleanHtml(html, title) {
  // Extract body
  const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  const body = bodyMatch ? bodyMatch[0] : `<body>${html}</body>`;

  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${title}</title>
  <link rel="stylesheet" href="styles.css">
</head>
${body}
</html>`;
}

capture().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});

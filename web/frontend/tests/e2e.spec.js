const { test, expect, chromium } = require('@playwright/test');

const SERVER_URL = 'http://localhost:8080';

let browser;
let page;

test.describe('EDA4 Web E2E', () => {
  test.beforeAll(async () => {
    browser = await chromium.launch({ headless: true });
  });

  test.afterAll(async () => {
    await browser.close();
  });

  test.beforeEach(async () => {
    page = await browser.newPage();
    page.on('console', msg => {
      if (msg.type() === 'error') console.log('BROWSER ERROR:', msg.text());
    });
    await page.goto(SERVER_URL);
    await page.waitForSelector('.tabs');
  });

  test.afterEach(async () => {
    await page.close();
  });

  test('page loads with correct title', async () => {
    await expect(page).toHaveTitle(/EDA4 Web/);
  });

  test('all three tabs are present', async () => {
    await expect(page.locator('.tab[data-tab="verilog-sim"]')).toBeVisible();
    await expect(page.locator('.tab[data-tab="verilog-pnr"]')).toBeVisible();
    await expect(page.locator('.tab[data-tab="spice"]')).toBeVisible();
  });

  test('verilog sim tab is active by default', async () => {
    await expect(page.locator('#panel-verilog-sim')).toBeVisible();
    await expect(page.locator('.tab[data-tab="verilog-sim"]')).toHaveClass(/active/);
  });

  test('clicking tab switches panels', async () => {
    await page.click('.tab[data-tab="spice"]');
    await expect(page.locator('#panel-spice')).toBeVisible();
    await expect(page.locator('#panel-verilog-sim')).not.toBeVisible();
  });

  test('verilog sim sends request via HTTP fallback', async () => {
    await page.fill('#verilog-code', 'module foo(a, b, s);\n  input a;\n  input b;\n  output s;\n  and g(s, a, b);\nendmodule');
    await page.click('#btn-verilog-run');

    const output = page.locator('#verilog-output');
    await expect(output).not.toHaveText(/Ready\./, { timeout: 30000 });
    const text = await output.textContent();
    console.log('Verilog sim output:', text.slice(0, 200));
  });

  test('spice tab sends request via HTTP fallback', async () => {
    await page.click('.tab[data-tab="spice"]');
    await page.fill('#spice-code', 'V1 Vin gnd DC 10\nR1 Vin Vout 1k\nR2 Vout gnd 1k\n.END');
    await page.selectOption('#spice-analysis', 'dc');
    await page.click('#btn-spice-run');

    const output = page.locator('#spice-output');
    await expect(output).not.toHaveText(/Ready\./, { timeout: 30000 });
    const text = await output.textContent();
    console.log('SPICE output:', text.slice(0, 200));
  });

  test('spice DC analysis shows voltage results', async () => {
    await page.click('.tab[data-tab="spice"]');
    await page.fill('#spice-code', 'V1 Vin gnd DC 5\nR1 Vin gnd 1k\n.END');
    await page.selectOption('#spice-analysis', 'dc');
    await page.click('#btn-spice-run');

    const output = page.locator('#spice-output');
    await expect(output).toContainText('V', { timeout: 30000 });
    const text = await output.textContent();
    console.log('SPICE DC voltage:', text.slice(0, 300));
  });

  test('clear button clears output', async () => {
    await page.fill('#verilog-code', 'module foo(a, b, s);\n  input a;\n  input b;\n  output s;\n  and g(s, a, b);\nendmodule');
    await page.click('#btn-verilog-run');
    await page.waitForTimeout(500);
    await page.click('#btn-verilog-clear');
    const output = await page.locator('#verilog-output').textContent();
    expect(output.trim()).toBe('');
  });

  test('verilog pnr tab works', async () => {
    await page.click('.tab[data-tab="verilog-pnr"]');
    await page.fill('#pnr-code', 'module top(input a, output y); assign y = a; endmodule');
    await page.click('#btn-pnr-run');

    const output = page.locator('#pnr-output');
    await expect(output).not.toHaveText(/Ready\./, { timeout: 60000 });
    const text = await output.textContent();
    console.log('PnR output:', text.slice(0, 200));
  });

  test('WebSocket status indicator exists', async () => {
    const status = page.locator('#wsStatus');
    await expect(status).toBeVisible();
    const classes = await status.getAttribute('class');
    console.log('WS status classes:', classes);
  });

  test('verilog sim with valid gate-level code generates rust output', async () => {
    await page.fill('#verilog-code', 'module foo(a, b, s);\n  input a;\n  input b;\n  output s;\n  and g(s, a, b);\nendmodule');
    await page.click('#btn-verilog-run');

    const generated = page.locator('#verilog-generated');
    await expect(generated).not.toHaveText('', { timeout: 30000 });
    const text = await generated.textContent();
    console.log('Generated Rust:', text.slice(0, 200));
  });
});
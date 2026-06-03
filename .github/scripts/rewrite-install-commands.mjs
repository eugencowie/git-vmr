import { copyFileSync, readFileSync, writeFileSync } from "node:fs";

const INDEX_HTML = "target/public/index.html";

const INSTALLERS = [
  {
    assetName: "git-vmr-installer.sh",
    source: "target/public/git-vmr-installer.sh.txt",
    destination: "target/public/install.sh",
    command: "curl https://eugencowie.github.io/git-vmr/install.sh | sh",
    highlightedCommand:
      '<span style="color:#82aaff;">curl </span>' +
      '<span style="color:#c3e88d;">https://eugencowie.github.io/git-vmr/install.sh </span>' +
      '<span style="color:#89ddff;">| </span>' +
      '<span style="color:#82aaff;">sh</span>',
  },
  {
    assetName: "git-vmr-installer.ps1",
    source: "target/public/git-vmr-installer.ps1.txt",
    destination: "target/public/install.ps1",
    command: "irm https://eugencowie.github.io/git-vmr/install.ps1 | iex",
    highlightedCommand:
      '<span style="color:#82aaff;">irm </span>' +
      '<span style="color:#c3e88d;">https://eugencowie.github.io/git-vmr/install.ps1 </span>' +
      '<span style="color:#89ddff;">| </span>' +
      '<span style="color:#82aaff;">iex</span>',
  },
];

function escapeHtml(value, quote = false) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', quote ? "&quot;" : '"');
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function replaceAllWithCount(html, pattern, replacement) {
  let count = 0;
  const updated = html.replace(pattern, (...args) => {
    count += 1;
    return typeof replacement === "function" ? replacement(...args) : replacement;
  });

  return [updated, count];
}

function replacePreBlocks(html, assetName, highlightedCommand) {
  const pattern = new RegExp(
    `(<pre\\b[^>]*>)(?:(?!</pre>).)*?${escapeRegex(assetName)}(?:(?!</pre>).)*?(</pre>)`,
    "gs",
  );

  return replaceAllWithCount(
    html,
    pattern,
    (_match, openPre, closePre) => `${openPre}${highlightedCommand}${closePre}`,
  );
}

function replaceCopyButtons(html, assetName, command) {
  const pattern = new RegExp(
    `<button\\b(?=[^>]*\\bcopy-clipboard-button\\b)(?=[^>]*${escapeRegex(assetName)})[^>]*>`,
    "gs",
  );
  const button =
    '<button class="button copy-clipboard-button primary" ' +
    `data-copy="${escapeHtml(command, true)}">`;

  return replaceAllWithCount(html, pattern, button);
}

let html = readFileSync(INDEX_HTML, "utf8");

for (const { assetName, source, destination, command, highlightedCommand } of INSTALLERS) {
  copyFileSync(source, destination);
  console.log(`copied ${source} to ${destination}`);

  let count = 0;
  [html, count] = replacePreBlocks(html, assetName, highlightedCommand);
  const preCount = count;
  [html, count] = replaceCopyButtons(html, assetName, command);
  const buttonCount = count;

  if (preCount === 0) {
    throw new Error(`did not find installer command for ${assetName}`);
  }
  if (buttonCount === 0) {
    throw new Error(`did not find copy button for ${assetName}`);
  }

  console.log(
    `rewrote ${preCount} code block(s) and ${buttonCount} copy button(s) for ${assetName}`,
  );
}

writeFileSync(INDEX_HTML, html);

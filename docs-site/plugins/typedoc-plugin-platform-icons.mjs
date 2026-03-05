// TypeDoc local plugin: turns "Platform" text into text badges.
import { MarkdownPageEvent } from "typedoc-plugin-markdown";

const ONLY_PREFIX = "only works on ";
const NOT_PREFIX = "does not work on ";

const PLATFORM_SECTION_RE = /^(#{2,6} Platform)\n\n([^\n]+)\n/gm;

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function splitPlatformList(rawList) {
  return rawList
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}

function parsePlatformSentence(sentence) {
  const normalized = sentence.trim();

  let supported = [];
  let unsupported = [];

  if (normalized.startsWith(ONLY_PREFIX)) {
    const unsupportedIndex = normalized.indexOf(`, ${NOT_PREFIX}`);
    if (unsupportedIndex === -1) {
      supported = splitPlatformList(normalized.slice(ONLY_PREFIX.length));
    } else {
      supported = splitPlatformList(
        normalized.slice(ONLY_PREFIX.length, unsupportedIndex),
      );
      unsupported = splitPlatformList(
        normalized.slice(unsupportedIndex + 2 + NOT_PREFIX.length),
      );
    }
  } else if (normalized.startsWith(NOT_PREFIX)) {
    unsupported = splitPlatformList(normalized.slice(NOT_PREFIX.length));
  } else {
    return null;
  }

  return { supported, unsupported };
}

function renderBadge(platformName, isSupported) {
  const statusClass = isSupported
    ? "platform-badge--supported"
    : "platform-badge--unsupported";
  const statusText = isSupported ? "Supported on" : "Not supported on";
  const statusIcon = isSupported ? "✓" : "✕";
  const label = isSupported ? `${platformName}-only` : platformName;
  const escapedLabel = escapeHtml(label);
  const escapedTitle = escapeHtml(`${statusText} ${platformName}`);

  return `<span class="platform-badge ${statusClass}" title="${escapedTitle}" aria-label="${escapedTitle}"><span class="platform-badge__icon" aria-hidden="true">${statusIcon}</span><span class="platform-badge__label">${escapedLabel}</span></span>`;
}

function renderPlatformBadges(platforms) {
  const badges = [
    ...platforms.supported.map((platform) => renderBadge(platform, true)),
    ...platforms.unsupported.map((platform) => renderBadge(platform, false)),
  ];

  if (badges.length === 0) {
    return "";
  }

  return `<div class="platform-badges">\n${badges.join("\n")}\n</div>`;
}

/** @param {import("typedoc-plugin-markdown").MarkdownApplication} app */
export function load(app) {
  app.renderer.on(MarkdownPageEvent.END, (page) => {
    if (!page.contents) {
      return;
    }

    page.contents = page.contents.replace(
      PLATFORM_SECTION_RE,
      (fullMatch, heading, platformSentence) => {
        const parsedPlatforms = parsePlatformSentence(platformSentence);
        if (!parsedPlatforms) {
          return fullMatch;
        }

        const badges = renderPlatformBadges(parsedPlatforms);
        if (!badges) {
          return fullMatch;
        }

        return `${heading}\n\n${badges}\n`;
      },
    );
  });
}

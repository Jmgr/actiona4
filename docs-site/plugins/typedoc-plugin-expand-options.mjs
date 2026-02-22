// TypeDoc local plugin: inlines @expand-tagged interface/enum members
// directly into parameter sections, so users don't have to click through.
import { MarkdownPageEvent, MarkdownRendererEvent } from "typedoc-plugin-markdown";
import { ReflectionKind } from "typedoc";
import * as fs from "fs";
import * as path from "path";

const PROPERTIES_MARKER = "\n## Properties\n\n";
const ENUM_MEMBERS_MARKER = "\n## Enumeration Members\n\n";

/** Extract the Properties or Enumeration Members section from a page's markdown content. */
function extractProperties(contents) {
  // Try "Properties" first (interfaces), then "Enumeration Members" (enums)
  let marker = PROPERTIES_MARKER;
  let start = contents.indexOf(marker);
  if (start === -1) {
    marker = ENUM_MEMBERS_MARKER;
    start = contents.indexOf(marker);
  }
  if (start === -1) return "";
  const bodyStart = start + marker.length;
  const nextSection = contents.indexOf("\n## ", bodyStart);
  const body =
    nextSection === -1
      ? contents.slice(bodyStart)
      : contents.slice(bodyStart, nextSection);
  return body.trimEnd();
}

/**
 * Rewrite relative markdown links in `content` so they resolve correctly
 * when the content is moved from `fromDir` to `toDir`.
 */
function rewriteLinks(content, fromDir, toDir) {
  if (fromDir === toDir) return content;
  // Match markdown links: [text](target) where target doesn't start with http/#
  return content.replace(/\[([^\]]*)\]\(([^)]+)\)/g, (match, text, target) => {
    if (/^https?:\/\/|^#/.test(target)) return match;
    // Resolve the target from fromDir, then make it relative to toDir
    const abs = path.resolve(fromDir, target);
    const rel = path.relative(toDir, abs);
    return `[${text}](${rel})`;
  });
}

/**
 * Demote headings by `delta` levels (capped at h6).
 */
function demoteHeadings(content, delta) {
  return content.replace(/^(#{1,6}) /gm, (_, hashes) => {
    const newLevel = Math.min(hashes.length + delta, 6);
    return "#".repeat(newLevel) + " ";
  });
}

/**
 * Within already-inlined options content, find property lines whose sole type
 * is an @expand enum link, and inline those enum members with an extra nesting level.
 *
 * Matches lines of the form (inside a blockquote):
 *   > **prop**: [`EnumName`](path.md)
 *   > `optional` **prop**: [`EnumName`](path.md)
 *   > `readonly` **prop**: [`EnumName`](path.md)
 *
 * We look for these inside options-fields divs and append enum members after
 * the property's content block (before the next *** separator or heading).
 */
function inlineEnumPropsInOptions(content, expandNames, expandedContent, fromDir, toDir) {
  if (expandNames.size === 0) return content;

  // Match a property blockquote line whose type is solely an @expand enum link.
  // Group 1: the full line (including newline)
  // Group 2: the enum type name
  // Group 3: the link target
  const PROP_RE =
    /^(> (?:`[^`]+` )*\*\*[^*]+\*\*: \[`([A-Za-z0-9_]+)`\]\(([^)]+\.md)\))\n/gm;

  return content.replace(PROP_RE, (match, propLine, typeName, _linkTarget) => {
    if (!expandNames.has(typeName)) return match;

    const entry = expandedContent.get(typeName);
    if (!entry || !entry.props) return match;

    // Demote enum member headings to h6 (maximum depth) since we're already
    // deeply nested inside an options-fields div. The visual nesting is provided
    // by the CSS left-border, not heading levels.
    let demoted = entry.props.replace(/^(#{1,6}) /gm, () => "###### ");

    // Fix relative links
    demoted = rewriteLinks(demoted, entry.dir, toDir);

    return `${propLine}\n\n<div class="options-fields">\n\n${demoted}\n\n</div>\n`;
  });
}

/** @param {import("typedoc-plugin-markdown").MarkdownApplication} app */
export function load(app) {
  // Set of interface names marked @expand
  const expandNames = new Set();
  // Map from interface name -> { props: string, dir: string }
  // Populated during page rendering, used for substitution after all pages are written
  const expandedContent = new Map();
  // Map from output filename -> page contents that need substitution
  const pendingPages = new Map();

  app.renderer.on(MarkdownRendererEvent.BEGIN, (event) => {
    // Collect all interfaces/type aliases/enums with @expand modifier
    const visit = (refl) => {
      if (
        refl.kindOf(
          ReflectionKind.Interface | ReflectionKind.TypeAlias | ReflectionKind.Enum,
        ) &&
        refl.comment?.hasModifier("@expand")
      ) {
        expandNames.add(refl.name);
      }
      if (refl.children) {
        for (const child of refl.children) visit(child);
      }
    };
    visit(event.project);
  });

  app.renderer.on(MarkdownPageEvent.END, (page) => {
    if (!page.contents || expandNames.size === 0) return;

    // If this page is an @expand interface, extract its properties block
    const pageName = path.basename(page.filename, ".md");
    if (expandNames.has(pageName)) {
      expandedContent.set(pageName, {
        props: extractProperties(page.contents),
        dir: path.dirname(page.filename),
      });
    }

    // Check if this page references any @expand types as sole parameter types
    const RE = /^#{3,6} [^\n]+\n\n\[`([A-Za-z0-9_]+)`\]\([^)]+\.md\)\n\n/gm;
    RE.lastIndex = 0;
    let needsSubstitution = false;
    let m;
    while ((m = RE.exec(page.contents)) !== null) {
      if (expandNames.has(m[1])) {
        needsSubstitution = true;
        break;
      }
    }
    if (needsSubstitution) {
      pendingPages.set(page.filename, page.contents);
    }
  });

  app.renderer.on(MarkdownRendererEvent.END, () => {
    if (pendingPages.size === 0) return;

    // Match parameter headings followed by a sole @expand interface link
    const PARAM_RE =
      /^(#{3,6} [^\n]+\n\n)(\[`([A-Za-z0-9_]+)`\]\(([^)]+\.md)\))\n\n/gm;

    for (const [filename, originalContents] of pendingPages) {
      const pageDir = path.dirname(filename);

      let substituted = originalContents.replace(
        PARAM_RE,
        (match, heading, linkLine, typeName, _linkTarget) => {
          if (!expandNames.has(typeName)) return match;

          const entry = expandedContent.get(typeName);
          if (!entry || !entry.props) return match;

          // Demote property headings to nest under the parameter heading level
          const paramLevel = (heading.match(/^(#+)/) ?? ["", "###"])[1].length;
          let demoted = demoteHeadings(entry.props, paramLevel - 1);

          // Fix relative links to resolve from the consuming page's directory
          demoted = rewriteLinks(demoted, entry.dir, pageDir);

          // Second pass: inline @expand enum types referenced by properties
          // within the already-inlined options content
          demoted = inlineEnumPropsInOptions(
            demoted,
            expandNames,
            expandedContent,
            pageDir,
            pageDir,
          );

          return `${heading}${linkLine}\n\n<div class="options-fields">\n\n${demoted}\n\n</div>\n\n`;
        },
      );

      if (substituted !== originalContents) {
        fs.writeFileSync(filename, substituted, "utf-8");
      }
    }
  });
}

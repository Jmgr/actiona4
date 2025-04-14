// TypeDoc local plugin: adds an "async" badge to function/method signatures
// that return Task, ProgressTask, or Promise.
import { MarkdownPageEvent } from "typedoc-plugin-markdown";

const ASYNC_BADGE = '<span class="async-badge">async</span> ';

// Matches a TypeDoc markdown signature line whose return type is Task,
// ProgressTask, or Promise, and inserts the badge right after "> ", e.g.:
//   > **sleep**(`duration`): [`Task`](...)
//   > **race**\<`T`\>(`promises`): [`Task`](...)
//   > `static` **captureDisplay**(`id`): [`Promise`](...)
const SIGNATURE_RE =
  /^(> )((?:[^\n]*?)\): \[`(?:Task|ProgressTask|Promise)`\])/gm;

/** @param {import("typedoc-plugin-markdown").MarkdownApplication} app */
export function load(app) {
  app.renderer.on(MarkdownPageEvent.END, (page) => {
    if (page.contents) {
      page.contents = page.contents.replace(
        SIGNATURE_RE,
        (match, gt, rest) => `${gt}${ASYNC_BADGE}${rest}`,
      );
    }
  });
}

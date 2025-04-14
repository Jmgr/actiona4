// TypeDoc local plugin: links intrinsic/primitive types to MDN
import { MarkdownPageEvent } from "typedoc-plugin-markdown";

const INTRINSIC_LINKS = {
  string:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String",
  number:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number",
  boolean:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean",
  object:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Object",
  void: "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void",
  undefined:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined",
  null: "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null",
  bigint:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/BigInt",
  symbol:
    "https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Symbol",
};

/** @param {import("typedoc-plugin-markdown").MarkdownApplication} app */
export function load(app) {
  app.renderer.on(MarkdownPageEvent.END, (page) => {
    if (page.contents) {
      page.contents = page.contents.replace(
        /(?<!\[)`(string|number|boolean|object|void|undefined|null|bigint|symbol)`/g,
        (match, type) => {
          const url = INTRINSIC_LINKS[type];
          return url ? `[\`${type}\`](${url})` : match;
        },
      );
    }
  });
}

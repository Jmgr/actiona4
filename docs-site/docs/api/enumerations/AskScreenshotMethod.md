# Enumeration: AskScreenshotMethod


Controls which interactive screenshot method is used.

```ts
const image = await screen.askScreenshot({ method: ScreenshotMethod.Portal });
```

## Enumeration Members

### Auto

> **Auto**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Auto`

Use the platform-default interactive screenshot picker.

***

### Overlay

> **Overlay**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Overlay`

Use the bundled overlay selector only.

***

### Portal

> **Portal**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Portal`

Use the XDG Desktop Portal only.

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

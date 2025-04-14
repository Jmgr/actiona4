# Enumeration: AskScreenshotMethod


Controls which interactive screenshot method is used.

```ts
const image = await screen.askScreenshot({ method: ScreenshotMethod.Native });
```

## Enumeration Members

### Auto

> **Auto**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Auto`

Use the platform-default interactive screenshot picker, falling back to
the overlay selector if the native picker is unavailable.

***

### Native

> **Native**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Native`

Use the platform native picker (XDG Desktop Portal on Linux, Snipping
Tool on Windows). Fails if the native picker is unavailable.

***

### Overlay

> **Overlay**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Overlay`

Use the bundled overlay selector only.

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

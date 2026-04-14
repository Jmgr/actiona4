<p align="center">
    <img src="./assets/actiona.png" alt="Actiona's icon" />
</p>

# Actiona 4

Actiona is a desktop task automation tool. It automates tasks you would normally perform manually, such as transferring data between applications, running automated tests, or handling repetitive workflows. Actiona supports Windows 10 or later, and Ubuntu 22.04 or later (X11 [preferred](#wayland)). Other Linux distributions should also work.

Currently, Actiona 4 provides a command-line tool (CLI) that runs scripts written in JavaScript or TypeScript. An action editor, similar to the one in Actiona 3, is planned.

Actiona 4 is still considered **unstable**. Expect bugs, incomplete features, and API changes between releases.

This is a Rust rewrite of the tool. See the [previous version (Actiona 3)](https://github.com/Jmgr/actiona).

Join the [Discord server](https://discord.gg/ubTjJu3dVZ) for support, discussion, and development updates.

## Examples

**Visual automation** — find a UI element by its image and click it:

```typescript
// Load a reference image of a button, find it on screen, and click it.
const template = await Image.load("ok-button.png");
const match = await template.findOnScreen(SearchIn.desktop());

if (match) {
  await mouse.move(match);
  await mouse.click();
  println(`Clicked at ${match.position}`);
} else {
  println("Button not found on screen");
}
```

**Keyboard events** — react to hotkeys and expand text abbreviations while you type:

```typescript
// Take a screenshot and save it to a file when Ctrl+Shift+S is pressed.
keyboard.onKeys([Key.Control, Key.Shift, "s"], async () => {
  const image = await screen.captureDesktop();
  await image.save(`screenshot-${Date.now()}.png`);
  println("Screenshot saved");
});

// Expand abbreviations as you type (also called "hotstrings").
keyboard.onText("addr", "123 Main Street, Springfield");
keyboard.onText("sig",  "Best regards,\nYour Name");
keyboard.onText("date", () => new Date().toLocaleDateString());

println("Active. Press Escape to stop.");
await keyboard.waitForKeys([Key.Escape]);
```

## Installation

### Windows

* [Installer](https://github.com/Jmgr/actiona4/releases/latest/download/actiona-run-windows-x86_64.exe)
* [Archive](https://github.com/Jmgr/actiona4/releases/latest/download/actiona-run-windows-x86_64.zip)
<!---
* One-liner: (PowerShell)
  ```powershell
  irm https://github.com/Jmgr/actiona4/releases/latest/download/installer.ps1 | iex
  ```
-->

### Linux

* [AppImage](https://github.com/Jmgr/actiona4/releases/latest/download/actiona-run-x86_64.AppImage)
<!---
* One-liner: (shell)
  ```sh
  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Jmgr/actiona4/releases/latest/download/installer.sh | sh
  ```
-->

## Quick Start

Initialize a script directory:

```sh
actiona-run init my-scripts
cd my-scripts
```

Run the starter script:

```sh
actiona-run script.ts
```

## Usage

**Initialize a directory** — creates `tsconfig.json`, copies TypeScript definitions, and adds a starter script

```sh
actiona-run init
```

**Run a script** — evaluates the contents of a script

```sh
actiona-run myscript.ts
```

**Run code** — evaluates code

```sh
actiona-run eval "await mouse.move(45, 100)"
```

**Record and play a macro** — records mouse and keyboard actions for later replay

```sh
actiona-run macros record recording.amac
actiona-run macros play recording.amac
```

**REPL** — allows you to enter and evaluate code line by line

```sh
actiona-run repl
```

### Compilation

<!---
See the [documentation](https://actiona.app) and [compilation guide](https://actiona.app/compilation).
-->

Coming soon!

## FAQ

<a id="wayland"></a>
### Does it work under Wayland?

It does, but only a limited selection of features are available due to the Wayland security model. Features like mouse/keyboard simulation, automatic screenshots, window manipulation won't work at all.

If you need those features then the simplest option is to start an X11 session instead of Wayland, if your distribution allows you to. If that's not possible then another option is to set up a virtual machine with a Linux distribution that uses X11 instead of Wayland.

### Actiona 3 has an editor app, where is the one for Actiona 4?

It's not done yet. To make development faster we decided to split the work in two: the command-line interface tool (CLI) and the editor itself. If you don't want to write JavaScript or TypeScript to automate your desktop, you can still use Actiona 3's editor in the meantime.

### Will Actiona 3's scripts be compatible with Actiona 4?

No. Actiona 4 is a complete rewrite of the program and actions, so keeping a compatibility layer would be too complicated.

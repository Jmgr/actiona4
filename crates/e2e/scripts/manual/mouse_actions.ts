if (system.platform === "wayland") {
  println("skipping: not supported on Wayland");
  exit();
}

// Tiny pause before reading mouse.position() — enigo's X11 mouse moves are
// dispatched asynchronously and the X server may need a moment to update the
// reported cursor position.
async function settle(): Promise<void> {
  await sleep("30ms");
}

async function assertCursorAt(
  x: number,
  y: number,
  what: string,
): Promise<void> {
  await settle();
  const pos = mouse.position();
  assertEq(pos.x, x, `${what}: cursor x`);
  assertEq(pos.y, y, `${what}: cursor y`);
}

await mouse.move(400, 400);

// click overloads
await mouse.click();
await assertCursorAt(400, 400, "click() leaves the cursor in place");

await mouse.click({});
await assertCursorAt(400, 400, "click({}) leaves the cursor in place");

await mouse.click(450, 460);
await assertCursorAt(450, 460, "click(x, y)");

await mouse.click({ x: 470, y: 480 });
await assertCursorAt(470, 480, "click({x, y})");

await mouse.click(new Point(490, 500));
await assertCursorAt(490, 500, "click(Point)");

await mouse.click(510, 520, { button: Button.Right });
await assertCursorAt(510, 520, "click(x, y, opts)");

await mouse.click({ x: 530, y: 540 }, { amount: 2, interval: 0.05 });
await assertCursorAt(530, 540, "click(point, opts)");

await mouse.click({ position: { x: 550, y: 560 } });
await assertCursorAt(550, 560, "click({position})");

await mouse.move(300, 300);
await mouse.click({ duration: 0.05 });
await assertCursorAt(300, 300, "options-only click leaves the cursor in place");

// doubleClick overloads
await mouse.move(300, 300);
await mouse.doubleClick();
await assertCursorAt(300, 300, "doubleClick() leaves the cursor in place");

await mouse.doubleClick(360, 370);
await assertCursorAt(360, 370, "doubleClick(x, y)");

await mouse.doubleClick({ x: 380, y: 390 }, { delay: 0.05 });
await assertCursorAt(380, 390, "doubleClick(point, opts)");

// press / release overloads — every press is paired with a release.
await mouse.move(200, 200);
mouse.press();
mouse.release();
await assertCursorAt(200, 200, "press()/release() leave the cursor in place");

mouse.press(220, 230);
await assertCursorAt(220, 230, "press(x, y)");
mouse.release();

mouse.press({ x: 240, y: 250 }, { button: Button.Right });
await assertCursorAt(240, 250, "press(point, opts)");
mouse.release(Button.Right);

mouse.press({ button: Button.Middle });
mouse.release(260, 270, Button.Middle);
await assertCursorAt(260, 270, "release(x, y, button)");

mouse.press();
mouse.release({ x: 280, y: 290 });
await assertCursorAt(280, 290, "release(point)");

println("OK");

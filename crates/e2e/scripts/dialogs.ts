// Dialogs need nobody to click anything here: each one is closed by a timeout, an abort signal,
// a task cancellation or `close()`.

await assertRejectsContains(
  () => dialogs.selectOne("Pick", []),
  "no items",
  "dialogs.selectOne should reject an empty list",
);

await assertRejectsContains(
  () => dialogs.selectMany("Pick", []),
  "no items",
  "dialogs.selectMany should reject an empty list",
);

async function hasDialogBackend(): Promise<boolean> {
  try {
    const result = await dialogs.messageBox("Checking for a dialog backend", { timeout: "100ms" });
    assertEq(result, MessageBoxResult.Timeout, "the probe message box should time out");
    return true;
  } catch (error) {
    if (String(error).includes("no dialog backend")) {
      return false;
    }
    throw error;
  }
}

if (!(await hasDialogBackend())) {
  println("Skipping the dialog tests that need a backend: install zenity or kdialog");
} else {
  const timeout = "300ms";

  assertEq(
    await dialogs.messageBox("Times out", {
      buttons: MessageBoxButtons.YesNoCancel,
      labels: { yes: "Save", no: "Discard" },
      icon: MessageBoxIcon.Warning,
      timeout,
    }),
    MessageBoxResult.Timeout,
    "dialogs.messageBox should return Timeout",
  );

  assertEq(
    await dialogs.textInput("Times out", { value: "value", timeout }),
    undefined,
    "dialogs.textInput should return undefined on timeout",
  );

  assertEq(
    await dialogs.colorPicker({ value: new Color(255, 0, 0), timeout }),
    undefined,
    "dialogs.colorPicker should return undefined on timeout",
  );

  assertEq(
    await dialogs.selectOne("Times out", ["Apple", "Pear"], { selected: "Pear", timeout }),
    undefined,
    "dialogs.selectOne should return undefined on timeout",
  );

  assertEq(
    await dialogs.selectMany("Times out", ["Apple", "Pear"], { selected: ["Apple"], timeout }),
    undefined,
    "dialogs.selectMany should return undefined on timeout",
  );

  assertEq(
    await dialogs.date("Times out", { value: new Date(2030, 0, 15), timeout }),
    undefined,
    "dialogs.date should return undefined on timeout",
  );

  const files = await dialogs.pickFiles({ title: "Times out", timeout });
  assertEq(files.length, 0, "dialogs.pickFiles should return an empty array on timeout");

  await assertRejectsContains(
    async () => {
      const controller = new AbortController();
      const task = dialogs.textInput("Aborted", { signal: controller.signal });
      await sleep("300ms");
      controller.abort();
      await task;
    },
    "Cancelled",
    "aborting a dialog's signal should reject it",
  );

  await assertRejectsContains(
    async () => {
      const task = dialogs.messageBox("Cancelled");
      await sleep("300ms");
      task.cancel();
      await task;
    },
    "Cancelled",
    "cancelling a dialog's task should reject it",
  );

  {
    const progress = await dialogs.progress("Starting", { title: "Progress", value: 0.25 });
    assertEq(progress.value, 0.25, "Progress.value should start at the given value");
    assertEq(progress.text, "Starting", "Progress.text should start at the given text");

    progress.value = 2;
    assertEq(progress.value, 1, "Progress.value should be clamped");
    progress.value = undefined;
    assertEq(progress.value, undefined, "Progress.value should accept a busy bar");
    progress.text = "Working";
    assertEq(progress.text, "Working", "Progress.text should be updated");
    assert(!progress.cancelled, "Progress.cancelled should be false while open");

    await sleep("300ms");
    await progress.close();
    await progress.close();
    await progress.waitForCancel();
  }

  {
    const controller = new AbortController();
    const progress = await dialogs.progress("Aborted", {
      cancellable: true,
      signal: controller.signal,
    });
    await sleep("300ms");
    controller.abort();
    await progress.waitForCancel();
  }
}

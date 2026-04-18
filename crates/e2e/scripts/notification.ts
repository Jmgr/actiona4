async function notificationErrorMessage(script: string): Promise<string> {
  return await eval(`
    (async () => {
      try {
        ${script}
        return "no_error";
      } catch (error) {
        return String(error);
      }
    })()
  `);
}

{
  const errorMessage = await notificationErrorMessage(
    `await notification.show({ actions: [{ label: "test" }] });`,
  );
  assert(
    errorMessage.includes("identifier"),
    `missing action identifier should mention identifier, got ${JSON.stringify(errorMessage)}`,
  );
}

{
  const errorMessage = await notificationErrorMessage(
    `await notification.show({ actions: [{ identifier: "test" }] });`,
  );
  assert(
    errorMessage.includes("label"),
    `missing action label should mention label, got ${JSON.stringify(errorMessage)}`,
  );
}

{
  const errorMessage = await notificationErrorMessage(
    `await notification.show({ actions: [{ identifier: "ok", label: "OK" }] });`,
  );
  assert(
    !errorMessage.includes("missing required field"),
    `optional bool fields should not behave as required, got ${JSON.stringify(errorMessage)}`,
  );
}

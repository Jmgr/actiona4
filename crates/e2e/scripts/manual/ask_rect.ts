if (system.platform === "wayland") {
  println("skipping: not supported on Wayland");
  exit();
}

const rect = await screen.askRect();
if (!rect) throw new Error("user cancelled");

println(`selected rect: ${rect}`);

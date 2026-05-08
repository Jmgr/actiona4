if (system.platform === "wayland") {
  println("skipping: not supported on Wayland");
  exit();
}

const rect = await screen.askPosition();
if (!rect) throw new Error("user cancelled");

println(`selected position: ${rect}`);

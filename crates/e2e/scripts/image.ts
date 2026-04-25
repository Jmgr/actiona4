// Create a 10x10 image
const img = new Image(10, 10);
assertEq(img.width, 10, "img.width");
assertEq(img.height, 10, "img.height");
assertEq(img.size.width, 10, "img.size.width");
assertEq(img.size.height, 10, "img.size.height");

// rect property
assertEq(img.rect.x, 0, "img.rect.x");
assertEq(img.rect.y, 0, "img.rect.y");
assertEq(img.rect.width, 10, "img.rect.width");
assertEq(img.rect.height, 10, "img.rect.height");

// fill and getPixel
img.fill(Color.Red);
const px = img.getPixel(0, 0);
assertEq(px.r, 255, "filled pixel r");
assertEq(px.g, 0, "filled pixel g");
assertEq(px.b, 0, "filled pixel b");

// setPixel / getPixel roundtrip
img.setPixel(new Point(5, 5), Color.Blue);
const px2 = img.getPixel(5, 5);
assertEq(px2.r, 0, "set pixel r");
assertEq(px2.g, 0, "set pixel g");
assertEq(px2.b, 255, "set pixel b");

// clone
const clone = img.clone();
assert(img.equals(clone), "clone equals original");
assert(!img.equals(new Image(10, 10)), "unfilled image differs from filled");

// filled (non-mutating)
const filled = img.filled(Color.Green);
const fpx = filled.getPixel(0, 0);
assertEq(fpx.g, 128, "filled green g=128");

// invertColors then getPixel
const inv = img.filled(Color.White).invertColors();
const ipx = inv.getPixel(0, 0);
assertEq(ipx.r, 0, "inverted white is black r=0");
assertEq(ipx.g, 0, "inverted white is black g=0");
assertEq(ipx.b, 0, "inverted white is black b=0");

// resize
const resized = new Image(10, 10).filled(Color.Red).resized(20, 20);
assertEq(resized.width, 20, "resized width");
assertEq(resized.height, 20, "resized height");

// toString includes dimensions
const str = img.toString();
assert(str.includes("10"), "toString includes dimensions");

assertEq(String(Font.default()), "Font(path: <built-in>)", "Font.default().toString()");

const tempDirValue = standardPaths.temp;
assert(typeof tempDirValue === "string", "temp path should be available for image e2e tests");
const tempDir: string = tempDirValue;

const testDataDir = Path.join(__e2eManifestDir, "..", "core", "test-data");
const crownPath = Path.join(testDataDir, "Crown_icon_transparent.png");
const overlayPath = Path.join(testDataDir, "250px-Search-icon-transparent-background.png");
const sourcePath = Path.join(testDataDir, "input.png");
const templatePath = Path.join(testDataDir, "Crown_icon_transparent.png");

async function saveAndAssert(image: Image, label: string): Promise<void> {
  const outputPath = Path.join(tempDir, `actiona4_e2e_image_${label}_${random.uuid()}.png`);
  await image.save(outputPath);
  assert(await File.exists(outputPath), `${label} should save an output image`);
  await File.remove(outputPath);
}

type GeneratedOperation = {
  label: string;
  apply: (input: Image, overlay: Image) => void;
};

const generatedOperations: GeneratedOperation[] = [
  { label: "invert_colors", apply: (input) => { input.invertColors(); } },
  { label: "blur_default", apply: (input) => { input.blur(); } },
  { label: "blur_sigma", apply: (input) => { input.blur({ sigma: 4 }); } },
  { label: "flip_horizontal", apply: (input) => { input.flip(FlipDirection.Horizontal); } },
  { label: "flip_vertical", apply: (input) => { input.flip(FlipDirection.Vertical); } },
  { label: "hue_rotate", apply: (input) => { input.hueRotate(90); } },
  { label: "grayscale", apply: (input) => { input.grayscale(); } },
  { label: "crop_rect", apply: (input) => { input.crop(new Rect(5, 5, 50, 40)); } },
  { label: "resize_default", apply: (input) => { input.resize(30, 40); } },
  { label: "resize_keep_aspect", apply: (input) => { input.resize(30, 40, { keepAspectRatio: true }); } },
  { label: "adjust_brightness", apply: (input) => { input.adjustBrightness(20); } },
  { label: "adjust_contrast", apply: (input) => { input.adjustContrast(-15); } },
  { label: "fill_color", apply: (input) => { input.fill(Color.Green); } },
  { label: "set_pixel_point", apply: (input) => { input.setPixel(new Point(0, 0), Color.Blue); } },
  { label: "set_pixel_object", apply: (input) => { input.setPixel({ x: 1, y: 2 }, Color.Red); } },
  { label: "set_pixel_numbers", apply: (input) => { input.setPixel(3, 4, Color.Green); } },
  { label: "draw_cross", apply: (input) => { input.drawCross(new Point(10, 10), Color.Red); } },
  { label: "draw_line", apply: (input) => { input.drawLine(new Point(0, 0), new Point(20, 10), Color.Blue); } },
  { label: "draw_circle_hollow", apply: (input) => { input.drawCircle(new Point(25, 25), 10, Color.Green, { hollow: true }); } },
  { label: "draw_ellipse", apply: (input) => { input.drawEllipse(new Point(25, 20), 12, 8, Color.Black); } },
  { label: "draw_rectangle_hollow", apply: (input) => { input.drawRectangle(new Rect(5, 5, 30, 15), Color.Black, { hollow: true }); } },
  {
    label: "draw_text",
    apply: (input) => { input.drawText(new Point(5, 25), "Test", Color.White, { fontSize: 12, horizontalAlign: TextHorizontalAlign.Left }); },
  },
  {
    label: "draw_image",
    apply: (input, overlay) => { input.drawImage(new Point(10, 10), overlay, { sourceRect: new Rect(0, 0, 16, 16) }); },
  },
  { label: "rotate_0", apply: (input) => { input.rotate(0); } },
  { label: "rotate_90", apply: (input) => { input.rotate(90); } },
  { label: "rotate_45", apply: (input) => { input.rotate(45); } },
  { label: "rotate_0_center", apply: (input) => { input.rotate(0, { center: new Point(0, 0) }); } },
  { label: "rotate_90_center", apply: (input) => { input.rotate(90, { center: new Point(0, 0) }); } },
  { label: "rotate_45_center", apply: (input) => { input.rotate(45, { center: new Point(0, 0) }); } },
  {
    label: "rotate_0_center_default_color",
    apply: (input) => { input.rotate(0, { center: new Point(0, 0), defaultColor: Color.Red }); },
  },
  {
    label: "rotate_90_center_default_color",
    apply: (input) => { input.rotate(90, { center: new Point(0, 0), defaultColor: Color.Red }); },
  },
  {
    label: "rotate_45_center_default_color",
    apply: (input) => { input.rotate(45, { center: new Point(0, 0), defaultColor: Color.Red }); },
  },
  {
    label: "rotate_0_center_default_color_nearest",
    apply: (input) => { input.rotate(0, { center: new Point(0, 0), defaultColor: Color.Red, interpolation: Interpolation.Nearest }); },
  },
  {
    label: "rotate_90_center_default_color_nearest",
    apply: (input) => { input.rotate(90, { center: new Point(0, 0), defaultColor: Color.Red, interpolation: Interpolation.Nearest }); },
  },
  {
    label: "rotate_45_center_default_color_nearest",
    apply: (input) => { input.rotate(45, { center: new Point(0, 0), defaultColor: Color.Red, interpolation: Interpolation.Nearest }); },
  },
];

for (const operation of generatedOperations) {
  const input = await Image.load(crownPath);
  const overlay = await Image.load(overlayPath);
  operation.apply(input, overlay);
  await saveAndAssert(input, operation.label);
}

{
  const source = await Image.load(sourcePath);
  const template = await Image.load(templatePath);
  const task = source.find(template, { useColors: true });

  const stages: FindImageStage[] = [];
  for await (const progress of task) {
    stages.push(progress.stage);
  }

  const result = await task;
  assert(result !== undefined, "Image.find should return a match for the known fixture");
  assert(new Point(result).equals(result.position), "FindImageResult can be used as PointLike to construct a Point");
  assertEq(stages[0], FindImageStage.Downscaling, "find() first stage");
  assert(stages.includes(FindImageStage.Matching), "find() should report Matching");
  assert(stages.includes(FindImageStage.Filtering), "find() should report Filtering");
  assert(stages.includes(FindImageStage.ComputingResults), "find() should report ComputingResults");
  assertEq(stages[stages.length - 1], FindImageStage.Finished, "find() last stage");
}

{
  const source = await Image.load(sourcePath);
  const template = await Image.load(templatePath);
  const task = source.findAll(template, { useColors: true });

  const stages: FindImageStage[] = [];
  let lastPercent = 0;
  for await (const progress of task) {
    stages.push(progress.stage);
    lastPercent = progress.percent;
  }

  const results = await task;
  assertEq(results.length, 2, "Image.findAll should return the two colored matches");
  assertEq(lastPercent, 100, "Image.findAll progress should finish at 100%");
  assertEq(stages[0], FindImageStage.Downscaling, "findAll() first stage");
  assert(stages.includes(FindImageStage.Matching), "findAll() should report Matching");
  assert(stages.includes(FindImageStage.Filtering), "findAll() should report Filtering");
  assert(stages.includes(FindImageStage.ComputingResults), "findAll() should report ComputingResults");
  assertEq(stages[stages.length - 1], FindImageStage.Finished, "findAll() last stage");
}

{
  const source = await Image.load(sourcePath);
  const template = await Image.load(templatePath);
  const results = await source.findAll(template, { useColors: false });
  assertEq(results.length, 3, "grayscale matching should also include the distractor");
  assert(
    results.some((result) => result.position.equals(new Point(767, 252))),
    "grayscale matching should include the known distractor position",
  );
}

{
  const source = await Image.load(sourcePath);
  const template = await Image.load(templatePath);
  const result = await source.find(template, { useColors: true, enableGpu: true });
  assert(result !== undefined, "Image.find with enableGpu should return a match for the known fixture");
}

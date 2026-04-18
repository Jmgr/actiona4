function requireEnv(name: string): string {
  const value = app.env[name];
  if (typeof value !== "string" || value.length === 0)
    throw new Error(`Missing required environment variable ${name}`);
  return value;
}

function assertBytesEq(actual: Uint8Array, expected: Uint8Array, message: string): void {
  assertEq(actual.length, expected.length, `${message} length`);
  for (let i = 0; i < actual.length; i += 1) {
    if (actual[i] !== expected[i]) {
      throw new Error(
        `Assertion failed: ${message}\n  first mismatch at index ${i}\n  expected: ${expected[i]}\n  actual:   ${actual[i]}`,
      );
    }
  }
}

const textUrl = requireEnv("ACTIONA4_E2E_WEB_TEXT_URL");
const imageUrl = requireEnv("ACTIONA4_E2E_WEB_IMAGE_URL");
const binaryUrl = requireEnv("ACTIONA4_E2E_WEB_BINARY_URL");
const authUrl = requireEnv("ACTIONA4_E2E_WEB_AUTH_URL");
const progressUrl = requireEnv("ACTIONA4_E2E_WEB_PROGRESS_URL");
const downloadFileUrl = requireEnv("ACTIONA4_E2E_WEB_DOWNLOAD_FILE_URL");
const multipartTextUrl = requireEnv("ACTIONA4_E2E_WEB_MULTIPART_TEXT_URL");
const multipartBytesUrl = requireEnv("ACTIONA4_E2E_WEB_MULTIPART_BYTES_URL");
const multipartFileUrl = requireEnv("ACTIONA4_E2E_WEB_MULTIPART_FILE_URL");

const testDataDir = Path.join(__e2eManifestDir, "..", "core", "test-data");
const imageFixturePath = Path.join(testDataDir, "Crown_icon_transparent.png");
const expectedBytes = await File.readBytes(imageFixturePath);

{
  const text = await web.downloadText(textUrl);
  assertEq(text, "hello", "web.downloadText should return the response body");
}

{
  const downloadedImage = await web.downloadImage(imageUrl);
  const expectedImage = await Image.load(imageFixturePath);
  assert(downloadedImage.equals(expectedImage), "web.downloadImage should decode the served image");
}

{
  const bytes = await web.download(binaryUrl);
  assertBytesEq(bytes, expectedBytes, "web.download should return the served bytes");
}

{
  const text = await web.downloadText(authUrl, {
    userName: "user",
    password: "password",
    method: Method.Post,
  });
  assertEq(text, "hello", "web.downloadText should support basic auth");
}

{
  const download = web.download(progressUrl);
  const progressUpdates: WebProgress[] = [];
  for await (const progress of download) {
    progressUpdates.push(progress);
  }
  const bytes = await download;
  assertBytesEq(bytes, expectedBytes, "web.download progress task should still resolve with bytes");
  assert(progressUpdates.length > 0, "web.download should yield at least one progress update");
  assert(progressUpdates.some((progress) => progress.finished), "progress should report a finished state");
  assert(
    progressUpdates
      .filter((progress) => progress.total > 0)
      .every((progress) => progress.total === expectedBytes.length),
    "progress totals should match the served content length",
  );
}

{
  const tempDir = standardPaths.temp;
  assert(typeof tempDir === "string", "standardPaths.temp should be available");
  const downloadDirectory = Path.join(tempDir, `actiona4_e2e_web_${random.uuid()}`);
  await Directory.create(downloadDirectory);

  const downloadedPath = await web.downloadFile(downloadFileUrl, downloadDirectory);
  const expectedPath = Path.join(downloadDirectory, "example.png");
  assertEq(downloadedPath, expectedPath, "web.downloadFile should use the response filename");
  assert(await File.exists(downloadedPath), "web.downloadFile should create the target file");
  const downloadedBytes = await File.readBytes(downloadedPath);
  assertBytesEq(downloadedBytes, expectedBytes, "web.downloadFile should save the served bytes");

  await File.remove(downloadedPath);
  await Directory.remove(downloadDirectory);
}

{
  const form = new MultipartForm();
  form.addText("title", "hello multipart");
  const response = await web.downloadText(multipartTextUrl, {
    method: Method.Post,
    multipart: form,
  });
  assertEq(response, "ok", "multipart text field upload should succeed");
}

{
  const form = new MultipartForm();
  form.addBytes(
    "payload",
    new Uint8Array([104, 101, 108, 108, 111, 45, 98, 121, 116, 101, 115]),
    "payload.bin",
    "application/octet-stream",
  );
  const response = await web.downloadText(multipartBytesUrl, {
    method: Method.Post,
    multipart: form,
  });
  assertEq(response, "ok", "multipart bytes field upload should succeed");
}

{
  const tempDir = standardPaths.temp;
  assert(typeof tempDir === "string", "standardPaths.temp should be available for multipart file upload");
  const filepath = Path.join(tempDir, "multipart_test.txt");
  await File.writeText(filepath, "file-content");

  const form = new MultipartForm();
  form.addFile("file", filepath, "multipart_test.txt", "text/plain");
  const response = await web.downloadText(multipartFileUrl, {
    method: Method.Post,
    multipart: form,
  });
  assertEq(response, "ok", "multipart file field upload should succeed");

  await File.remove(filepath);
}

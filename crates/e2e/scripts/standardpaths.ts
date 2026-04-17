// All standard paths should be null or a non-empty string
const paths = [
  standardPaths.home,
  standardPaths.temp,
  standardPaths.music,
  standardPaths.desktop,
  standardPaths.documents,
  standardPaths.downloads,
  standardPaths.pictures,
  standardPaths.public,
  standardPaths.videos,
  standardPaths.cache,
];

for (const p of paths) {
  assert(
    p === null || (typeof p === "string" && p.length > 0),
    `standard path should be null or non-empty string, got ${JSON.stringify(p)}`,
  );
}

// home and temp are almost always present; at least one should be non-null
const hasHome = standardPaths.home !== null;
const hasTemp = standardPaths.temp !== null;
assert(hasHome || hasTemp, "at least one of home/temp should be non-null");

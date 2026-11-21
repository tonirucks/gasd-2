/** Case-insensitive subsequence search, favoring exact and contiguous matches. */
export function searchFolders(paths: string[], query: string): string[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return [...paths];
  return paths.map((path) => {
    const value = path.toLowerCase();
    const contiguous = value.indexOf(needle);
    if (contiguous >= 0) return { path, score: contiguous - 1000 };
    let cursor = 0;
    let score = 0;
    for (const character of needle) {
      const index = value.indexOf(character, cursor);
      if (index < 0) return { path, score: Infinity };
      score += index - cursor;
      cursor = index + 1;
    }
    return { path, score };
  }).filter(({ score }) => Number.isFinite(score))
    .sort((a, b) => a.score - b.score || a.path.localeCompare(b.path))
    .map(({ path }) => path);
}

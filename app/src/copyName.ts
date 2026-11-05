/** Fit a generated copy name to the backend's UTF-8 and editor's UTF-16 limits. */
export function copyName(
  name: string,
  suffix: " copy" | " (copy)" = " copy",
  maxUnits = 80,
): string {
  const encoder = new TextEncoder();
  let result = "";
  let bytes = encoder.encode(suffix).length;
  // Iterate code points so truncation never splits a surrogate pair.
  for (const character of name) {
    const length = encoder.encode(character).length;
    if (
      result.length + character.length + suffix.length > maxUnits ||
      bytes + length > 120
    )
      break;
    result += character;
    bytes += length;
  }
  return result + suffix;
}

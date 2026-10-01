import raw from "../../CHANGELOG.md?raw";

export interface Release {
  version: string;
  paragraphs: string[];
  items: string[];
}

/** Reads the project's own CHANGELOG.md: "## version" headings, bullets and plain lines. */
export function releases(): Release[] {
  const out: Release[] = [];
  for (const line of raw.split(/\r?\n/)) {
    const heading = /^##\s+(.+)$/.exec(line);
    if (heading) {
      out.push({ version: heading[1].trim(), paragraphs: [], items: [] });
      continue;
    }
    const current = out[out.length - 1];
    if (!current || !line.trim()) continue;
    const bullet = /^[-*]\s+(.+)$/.exec(line);
    if (bullet) current.items.push(bullet[1].trim());
    else current.paragraphs.push(line.trim());
  }
  return out;
}

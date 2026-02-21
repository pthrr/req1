export interface DiffSegment {
  type: "equal" | "added" | "removed";
  text: string;
}

/**
 * Word-level diff using LCS. Tokenizes on whitespace boundaries.
 */
export function wordDiff(a: string, b: string): DiffSegment[] {
  const tokensA = tokenize(a);
  const tokensB = tokenize(b);

  const lcs = computeLcs(tokensA, tokensB);
  const segments: DiffSegment[] = [];

  let ia = 0;
  let ib = 0;
  let il = 0;

  while (il < lcs.length) {
    // Emit removed words before next LCS match
    while (ia < tokensA.length && tokensA[ia] !== lcs[il]) {
      push(segments, "removed", tokensA[ia]);
      ia++;
    }
    // Emit added words before next LCS match
    while (ib < tokensB.length && tokensB[ib] !== lcs[il]) {
      push(segments, "added", tokensB[ib]);
      ib++;
    }
    // Emit equal
    push(segments, "equal", lcs[il]);
    ia++;
    ib++;
    il++;
  }

  // Remaining
  while (ia < tokensA.length) {
    push(segments, "removed", tokensA[ia]);
    ia++;
  }
  while (ib < tokensB.length) {
    push(segments, "added", tokensB[ib]);
    ib++;
  }

  return segments;
}

function tokenize(s: string): string[] {
  return s.split(/(\s+)/).filter((t) => t.length > 0);
}

function computeLcs(a: string[], b: string[]): string[] {
  const m = a.length;
  const n = b.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () =>
    new Array<number>(n + 1).fill(0),
  );

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] =
        a[i - 1] === b[j - 1]
          ? dp[i - 1][j - 1] + 1
          : Math.max(dp[i - 1][j], dp[i][j - 1]);
    }
  }

  const result: string[] = [];
  let i = m;
  let j = n;
  while (i > 0 && j > 0) {
    if (a[i - 1] === b[j - 1]) {
      result.push(a[i - 1]);
      i--;
      j--;
    } else if (dp[i - 1][j] > dp[i][j - 1]) {
      i--;
    } else {
      j--;
    }
  }

  return result.reverse();
}

function push(segments: DiffSegment[], type: DiffSegment["type"], text: string) {
  const last = segments[segments.length - 1];
  if (last && last.type === type) {
    last.text += text;
  } else {
    segments.push({ type, text });
  }
}

export interface SnapEdge {
  ms: number;
  source: 'start' | 'end';
}

export interface SnapResult {
  snapped: number;
  hit: boolean;
  source: 'start' | 'end' | 'zero' | null;
}

export function snap(
  candidateMs: number,
  edges: readonly SnapEdge[],
  thresholdMs: number,
): SnapResult {
  let bestDistance = Infinity;
  let bestSnapped = candidateMs;
  let bestSource: SnapResult['source'] = null;

  const zeroDistance = Math.abs(candidateMs - 0);
  if (zeroDistance <= thresholdMs && zeroDistance < bestDistance) {
    bestDistance = zeroDistance;
    bestSnapped = 0;
    bestSource = 'zero';
  }

  for (const edge of edges) {
    const distance = Math.abs(candidateMs - edge.ms);
    if (distance <= thresholdMs && distance < bestDistance) {
      bestDistance = distance;
      bestSnapped = edge.ms;
      bestSource = edge.source;
    }
  }

  return {
    snapped: bestSnapped,
    hit: bestSource !== null,
    source: bestSource,
  };
}

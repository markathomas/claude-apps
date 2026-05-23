export function msToPx(ms: number, pxPerSec: number): number {
  return (ms / 1000) * pxPerSec;
}

export function pxToMs(px: number, pxPerSec: number): number {
  if (pxPerSec === 0) return 0;
  return (px / pxPerSec) * 1000;
}

function pad2(n: number): string {
  return n < 10 ? `0${n}` : String(n);
}

export function formatTimecode(ms: number, fps: number): string {
  const safeMs = Math.max(0, Math.floor(ms));
  const totalSeconds = Math.floor(safeMs / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const remainderMs = safeMs - totalSeconds * 1000;
  const frames = Math.floor((remainderMs / 1000) * fps);
  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}:${pad2(frames)}`;
}

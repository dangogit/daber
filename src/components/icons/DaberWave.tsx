import React from "react";

/**
 * The Daber mark: a voice rendered as five rounded bars.
 *
 * The heights are deliberately uneven and peak off-centre — an evenly stepped
 * arc reads as a generic equaliser, while this reads as a moment of speech.
 * Drawn in `currentColor` so it inherits whatever it sits in (sidebar, tray,
 * onboarding) rather than carrying its own palette.
 */
const DaberWave = ({
  width,
  height,
  className,
}: {
  width?: number | string;
  height?: number | string;
  className?: string;
}) => (
  <svg
    width={width || 126}
    height={height || 126}
    viewBox="0 0 120 120"
    className={className ?? "fill-current"}
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <g>
      <rect x="12" y="46" width="14" height="28" rx="7" />
      <rect x="35" y="28" width="14" height="64" rx="7" />
      <rect x="58" y="10" width="14" height="100" rx="7" />
      <rect x="81" y="34" width="14" height="52" rx="7" />
      <rect x="104" y="52" width="14" height="16" rx="7" />
    </g>
  </svg>
);

export default DaberWave;

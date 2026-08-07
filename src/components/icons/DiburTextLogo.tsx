import React from "react";

/**
 * The Dibur wordmark: the voice mark, then the name.
 *
 * The Hebrew is drawn as outlines rather than an SVG `<text>` element. Live
 * text inherits `direction: rtl` from the document when the UI is in Hebrew,
 * which runs the word leftward out of its box and over the mark, and it also
 * depends on whichever Hebrew face the system happens to have. Outlines have
 * neither problem and render identically everywhere.
 *
 * Colours come from the theme tokens, so the mark and the name both follow
 * light and dark mode instead of being pinned to one background.
 */
const DiburTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => (
  <svg
    width={width}
    height={height}
    className={className}
    viewBox="0 0 306 78"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    role="img"
    aria-label="Dibur"
  >
    <g className="fill-logo-primary">
      <rect x="10" y="32.0" width="12" height="14" rx="6.0" />
      <rect x="30" y="24.0" width="12" height="30" rx="6.0" />
      <rect x="50" y="13.0" width="12" height="52" rx="6.0" />
      <rect x="70" y="26.0" width="12" height="26" rx="6.0" />
      <rect x="90" y="33.0" width="12" height="12" rx="6.0" />
    </g>
    <g className="fill-text" transform="translate(124 59.0)">
      <path d="M154.22 0.68Q151.59 0.68 150.1 -0.87Q148.61 -2.43 148.61 -5.19V-34.81H130.38Q128.43 -34.81 127.22 -36.04Q126 -37.26 126 -39.21Q126 -41.17 127.22 -42.39Q128.43 -43.6 130.38 -43.6H163.78Q165.73 -43.6 166.93 -42.39Q168.13 -41.17 168.13 -39.21Q168.13 -37.26 166.92 -36.04Q165.71 -34.81 163.75 -34.81H159.82V-5.19Q159.82 -2.43 158.33 -0.87Q156.85 0.68 154.22 0.68Z" />
      <path d="M114.68 -20.13Q112.05 -20.13 110.57 -21.68Q109.08 -23.24 109.08 -26V-38.43Q109.08 -41.19 110.57 -42.75Q112.05 -44.31 114.68 -44.31Q117.32 -44.31 118.8 -42.75Q120.29 -41.19 120.29 -38.43V-26Q120.29 -23.24 118.8 -21.68Q117.32 -20.13 114.68 -20.13Z" />
      <path d="M67.14 0Q65.19 0 63.97 -1.23Q62.76 -2.45 62.76 -4.4Q62.76 -6.36 63.97 -7.58Q65.19 -8.79 67.14 -8.79H85.73V-25.19Q85.73 -28.88 85.05 -30.97Q84.36 -33.06 82.6 -33.94Q80.85 -34.81 77.62 -34.81H67.26Q65.32 -34.81 64.1 -36.04Q62.88 -37.26 62.88 -39.21Q62.88 -41.17 64.1 -42.39Q65.32 -43.6 67.26 -43.6H78.31Q83.59 -43.6 87.16 -42.7Q90.74 -41.8 92.89 -39.68Q95.05 -37.57 95.99 -33.92Q96.94 -30.27 96.94 -24.76V-8.79H99.75Q101.75 -8.79 102.94 -7.6Q104.13 -6.41 104.13 -4.4Q104.13 -2.4 102.94 -1.2Q101.75 0 99.75 0Z" />
      <path d="M50.68 0.68Q48.05 0.68 46.57 -0.87Q45.08 -2.43 45.08 -5.19V-38.43Q45.08 -41.19 46.57 -42.75Q48.05 -44.31 50.68 -44.31Q53.32 -44.31 54.8 -42.75Q56.29 -41.19 56.29 -38.43V-5.19Q56.29 -2.43 54.8 -0.87Q53.32 0.68 50.68 0.68Z" />
      <path d="M31.41 0.77Q28.76 0.79 27.26 -0.77Q25.77 -2.32 25.77 -5.08V-23.87Q25.77 -27.95 25.05 -30.34Q24.32 -32.74 22.33 -33.77Q20.34 -34.81 16.47 -34.81H6.09Q4.14 -34.81 2.93 -36.04Q1.71 -37.26 1.71 -39.21Q1.71 -41.17 2.93 -42.39Q4.14 -43.6 6.09 -43.6H16.47Q24.27 -43.6 28.74 -41.73Q33.21 -39.86 35.09 -35.54Q36.97 -31.23 36.97 -23.93V-5.08Q36.97 -2.34 35.5 -0.79Q34.04 0.76 31.41 0.77Z" />
    </g>
  </svg>
);

export default DiburTextLogo;

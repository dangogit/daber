import React from "react";

/**
 * The name, held in a constant rather than inlined as JSX text. A wordmark is
 * the product's identity, not translatable copy — it reads דבר in every locale
 * — and keeping it out of the markup also keeps the i18n lint rule honest
 * instead of suppressed.
 */
const WORDMARK = "דבר";

/**
 * The Daber wordmark: the mark, then the name in Hebrew.
 *
 * Laid out left-to-right on purpose even though the word is Hebrew — a logo is
 * an image, not running text, and the mark reads as the "start" of it in either
 * script direction. The name is set in live text rather than outlines so it
 * picks up the system's Hebrew face and stays crisp at any size; the app is
 * macOS-first, where that face is dependable.
 */
const DaberTextLogo = ({
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
    viewBox="0 0 420 140"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    role="img"
    aria-label={WORDMARK}
  >
    <g className="fill-logo-primary">
      <rect x="8" y="58" width="16" height="26" rx="8" />
      <rect x="34" y="42" width="16" height="58" rx="8" />
      <rect x="60" y="22" width="16" height="98" rx="8" />
      <rect x="86" y="48" width="16" height="46" rx="8" />
      <rect x="112" y="64" width="16" height="14" rx="8" />
    </g>
    <text
      x="158"
      y="98"
      className="fill-text"
      fontSize="86"
      fontWeight="700"
      letterSpacing="2"
      fontFamily="'SF Hebrew', 'Arial Hebrew', 'Noto Sans Hebrew', system-ui, sans-serif"
    >
      {WORDMARK}
    </text>
  </svg>
);

export default DaberTextLogo;

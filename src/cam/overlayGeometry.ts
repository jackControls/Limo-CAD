/** Renderer-independent CAM presentation geometry. */
export type OverlayLinePattern = 'solid' | 'dotted';

export interface OverlayLineLayer {
  color: [number, number, number, number];
  width: number;
  /** Presentation meaning, kept semantic so renderers can preserve screen-space spacing. */
  pattern: OverlayLinePattern;
  segments: number[];
  /** Optional CAM timeline: one start/end time pair per retained segment. */
  playback?: {
    pathId: number;
    completedColor: [number, number, number, number];
    segmentTimes: number[];
  };
}

export interface OverlayPointLayer {
  color: [number, number, number, number];
  radius: number;
  /** Hollow rings retain geometric freedom; filled dots are fully constrained. */
  hollow?: boolean;
  positions: number[];
}

/** Triangle-list presentation geometry for presentation, never kernel geometry. */
export interface OverlayTriangleLayer {
  color: [number, number, number, number];
  /** World-space triangle vertices, packed as x, y, z. */
  positions: number[];
  /** Optional world-space vertex normals, packed one-for-one with positions.
   *  Surface-producing systems can provide smooth/feature-aware normals;
   *  command overlays can use flat normals. */
  normals?: number[];
  /** Physical CAM stock is opaque and studio-lit. Command/profile fills keep
   *  the historical unlit translucent overlay presentation. */
  material?: 'overlay' | 'machined_stock';
  /** Render after model depth so an internal selected profile remains visible. */
  xray: boolean;
}

/** Semantic CAD direction arrow. The renderer owns its shaft and arrowhead pixels. */
export interface OverlayArrow {
  start: [number, number, number];
  end: [number, number, number];
  color: [number, number, number, number];
  /** Approximate screen-space width in logical pixels. */
  width: number;
  xray: boolean;
}


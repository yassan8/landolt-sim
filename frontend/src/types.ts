export interface SimParams {
  /** Outer radius of the Landolt C ring in pixels */
  outerRadius: number
  /** Ring width as a fraction of outerRadius (0 < ratio < 1) */
  ringWidthRatio: number
  /** Angular width of the gap in degrees */
  gapDeg: number
  /** Clockwise rotation of the gap from the top (0 = gap at top) */
  rotationDeg: number
  /** Gaussian PSF sigma in pixels (0 = no blur) */
  sigma: number
}

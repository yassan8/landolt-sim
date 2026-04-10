import { useEffect, useRef } from 'react'
import type { SimParams } from '../types'

const CANVAS_SIZE = 320

interface Props {
  params: SimParams
  wasm: typeof import('../wasm/landolt_wasm')
}

export default function SimulationCanvas({ params, wasm }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const { outerRadius, ringWidthRatio, gapDeg, rotationDeg, sigma } = params
    const ringWidth = outerRadius * ringWidthRatio
    const cx = CANVAS_SIZE / 2
    const cy = CANVAS_SIZE / 2

    const rgba = wasm.render_landolt(
      CANVAS_SIZE,
      CANVAS_SIZE,
      cx,
      cy,
      outerRadius,
      ringWidth,
      gapDeg,
      rotationDeg,
      sigma,
      255,
      0,
    )

    const imageData = new ImageData(
      new Uint8ClampedArray(rgba.buffer as ArrayBuffer),
      CANVAS_SIZE,
      CANVAS_SIZE,
    )
    ctx.putImageData(imageData, 0, 0)
  }, [params, wasm])

  return (
    <div className="canvas-wrapper">
      <canvas
        ref={canvasRef}
        width={CANVAS_SIZE}
        height={CANVAS_SIZE}
        aria-label="Landolt C retinal image simulation"
      />
    </div>
  )
}

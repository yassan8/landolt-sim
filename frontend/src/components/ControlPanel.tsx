import type { SimParams } from '../types'

interface Props {
  params: SimParams
  onChange: (p: SimParams) => void
}

interface SliderDef {
  key: keyof SimParams
  label: string
  min: number
  max: number
  step: number
  unit?: string
}

const SLIDERS: SliderDef[] = [
  { key: 'outerRadius', label: '外径 (Outer radius)', min: 20, max: 140, step: 1, unit: 'px' },
  { key: 'ringWidthRatio', label: 'リング幅比 (Ring width ratio)', min: 0.05, max: 0.5, step: 0.01 },
  { key: 'gapDeg', label: 'ギャップ角度 (Gap angle)', min: 10, max: 120, step: 1, unit: '°' },
  { key: 'rotationDeg', label: '回転角度 (Rotation)', min: 0, max: 360, step: 1, unit: '°' },
  { key: 'sigma', label: 'PSF σ (Blur)', min: 0, max: 20, step: 0.5, unit: 'px' },
]

export default function ControlPanel({ params, onChange }: Props) {
  const handleChange = (key: keyof SimParams, value: number) => {
    onChange({ ...params, [key]: value })
  }

  return (
    <div className="control-panel">
      <h2>パラメータ</h2>
      {SLIDERS.map(({ key, label, min, max, step, unit }) => (
        <div key={key} className="slider-row">
          <label htmlFor={`slider-${key}`}>
            {label}
            <span className="value">
              {' '}
              {typeof params[key] === 'number' ? Number(params[key]).toFixed(step < 1 ? 2 : 0) : params[key]}
              {unit ?? ''}
            </span>
          </label>
          <input
            id={`slider-${key}`}
            type="range"
            min={min}
            max={max}
            step={step}
            value={params[key] as number}
            onChange={(e) => handleChange(key, parseFloat(e.target.value))}
          />
        </div>
      ))}
    </div>
  )
}

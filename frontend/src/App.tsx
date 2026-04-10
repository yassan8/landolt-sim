import { useEffect, useRef, useState } from 'react'
import './App.css'
import SimulationCanvas from './components/SimulationCanvas'
import ControlPanel from './components/ControlPanel'
import type { SimParams } from './types'

const DEFAULT_PARAMS: SimParams = {
  outerRadius: 60,
  ringWidthRatio: 0.2,
  gapDeg: 45,
  rotationDeg: 0,
  sigma: 0,
}

function App() {
  const [params, setParams] = useState<SimParams>(DEFAULT_PARAMS)
  const [ready, setReady] = useState(false)
  const wasmRef = useRef<typeof import('./wasm/landolt_wasm') | null>(null)

  useEffect(() => {
    import('./wasm/landolt_wasm').then((mod) => {
      mod.default().then(() => {
        wasmRef.current = mod
        setReady(true)
      })
    })
  }, [])

  return (
    <div className="app">
      <header>
        <h1>ランドルト環 網膜像シミュレーター</h1>
        <p className="subtitle">Landolt C Retinal Image Simulator</p>
      </header>
      <main>
        {ready && wasmRef.current ? (
          <>
            <SimulationCanvas params={params} wasm={wasmRef.current} />
            <ControlPanel params={params} onChange={setParams} />
          </>
        ) : (
          <p className="loading">Loading WASM…</p>
        )}
      </main>
    </div>
  )
}

export default App

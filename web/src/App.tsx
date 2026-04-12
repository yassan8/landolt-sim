import { Eye, RefreshCcw, Sparkles, Waves } from "lucide-react";
import { startTransition, useDeferredValue, useEffect, useMemo, useState } from "react";
import Plot from "react-plotly.js";
import type { Data, Layout } from "plotly.js";

type WasmModule = {
  default: () => Promise<void>;
  simulate_retinal_image_js: (input: unknown) => SimulationResult;
};

type ZernikeControl = {
  key: string;
  label: string;
  n: number;
  m: number;
  min: number;
  max: number;
  step: number;
};

type Controls = {
  sphere: number;
  cylinder: number;
  axis: number;
  sceEnabled: boolean;
  sceRho: number;
  imageSamples: number;
  pupilDiameterMm: number;
  wavelengthNm: number;
  targetFovArcmin: number;
  zernike: Record<string, number>;
};

type SimulationPhase = "interactive" | "settled";

type SimulationResult = {
  pupil_samples: number;
  wavefront: {
    width: number;
    height: number;
    wavefront_um: number[];
    min_um: number;
    max_um: number;
    coefficients: Array<{ mode: { n: number; m: number }; coefficient_um: number; source: string }>;
  };
  psf: {
    width: number;
    height: number;
    data: number[];
    fov_arcmin: number;
  };
  chart: {
    width: number;
    height: number;
    data: number[];
    x: number[];
    y: number[];
    placements: Array<{
      acuity: number;
      x_arcmin: number;
      y_arcmin: number;
      outer_radius_arcmin: number;
      gap_angle_degrees: number;
    }>;
  };
  retinal_image: number[];
};

type PanelProps = {
  title: string;
  x: number[];
  y: number[];
  values: number[];
  colorScale: Plotly.ColorScaleName | string[][];
  reverseScale?: boolean;
  logScale?: boolean;
  smooth?: boolean;
  axisUnit: string;
  annotations?: Array<{ x: number; y: number; text: string }>;
  xRange?: [number, number];
  yRange?: [number, number];
};

const zernikeControls: ZernikeControl[] = [
  { key: "comaVertical", label: "Coma C[3,-1]", n: 3, m: -1, min: -0.4, max: 0.4, step: 0.01 },
  { key: "comaHorizontal", label: "Coma C[3,1]", n: 3, m: 1, min: -0.4, max: 0.4, step: 0.01 },
  { key: "trefoilVertical", label: "Trefoil C[3,-3]", n: 3, m: -3, min: -0.4, max: 0.4, step: 0.01 },
  { key: "trefoilOblique", label: "Trefoil C[3,3]", n: 3, m: 3, min: -0.4, max: 0.4, step: 0.01 },
  { key: "spherical", label: "Spherical C[4,0]", n: 4, m: 0, min: -0.4, max: 0.4, step: 0.01 },
  { key: "secondaryAstigOblique", label: "Secondary Astig C[4,-2]", n: 4, m: -2, min: -0.4, max: 0.4, step: 0.01 },
  { key: "secondaryAstig", label: "Secondary Astig C[4,2]", n: 4, m: 2, min: -0.4, max: 0.4, step: 0.01 },
  { key: "quadrafoilOblique", label: "Quadrafoil C[4,-4]", n: 4, m: -4, min: -0.4, max: 0.4, step: 0.01 },
  { key: "quadrafoil", label: "Quadrafoil C[4,4]", n: 4, m: 4, min: -0.4, max: 0.4, step: 0.01 },
];

const initialZernike = Object.fromEntries(zernikeControls.map((control) => [control.key, 0]));

const initialControls: Controls = {
  sphere: 0,
  cylinder: 0,
  axis: 0,
  sceEnabled: false,
  sceRho: 0.12,
  imageSamples: 1024,
  pupilDiameterMm: 6,
  wavelengthNm: 555,
  targetFovArcmin: 240,
  zernike: initialZernike,
};

const interactiveImageSamples = 64;
const settleDelayMs = 180;

const landoltAcuities = [0.1, 0.2, 0.3, 0.5, 0.7, 1.0, 1.2, 1.5, 2.0];

let wasmPromise: Promise<WasmModule> | null = null;

async function loadWasm() {
  if (!wasmPromise) {
    wasmPromise = (async () => {
      const module = (await import("/pkg/landolt_sim.js")) as WasmModule;
      await module.default();
      return module;
    })();
  }

  return wasmPromise;
}

export default function App() {
  const [controls, setControls] = useState<Controls>(initialControls);
  const [simulationPhase, setSimulationPhase] = useState<SimulationPhase>("settled");
  const deferredControls = useDeferredValue(controls);
  const [result, setResult] = useState<SimulationResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(true);

  useEffect(() => {
    setSimulationPhase("interactive");
    const timeoutId = window.setTimeout(() => {
      setSimulationPhase("settled");
    }, settleDelayMs);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [controls]);

  const requestControls = useMemo(() => {
    if (simulationPhase === "settled") {
      return deferredControls;
    }

    return {
      ...deferredControls,
      imageSamples: Math.min(interactiveImageSamples, deferredControls.imageSamples),
    };
  }, [deferredControls, simulationPhase]);

  useEffect(() => {
    let cancelled = false;

    async function runSimulation() {
      setPending(true);
      try {
        const wasm = await loadWasm();
        const nextResult = wasm.simulate_retinal_image_js(buildSimulationRequest(requestControls));
        if (!cancelled) {
          startTransition(() => {
            setResult(nextResult);
            setError(null);
          });
        }
      } catch (simulationError) {
        if (!cancelled) {
          setError(simulationError instanceof Error ? simulationError.message : String(simulationError));
        }
      } finally {
        if (!cancelled) {
          setPending(false);
        }
      }
    }

    void runSimulation();

    return () => {
      cancelled = true;
    };
  }, [requestControls]);

  const chartAnnotations = useMemo(() => {
    if (!result) {
      return [];
    }

    return result.chart.placements.map((placement) => ({
      x: placement.x_arcmin,
      y: placement.y_arcmin - (placement.outer_radius_arcmin + 5),
      text: placement.acuity.toFixed(1),
    }));
  }, [result]);

  const psfAxis = result ? createSymmetricAxis(result.psf.width, result.psf.fov_arcmin) : [];

  return (
    <div className="app-shell">
      <header className="hero">
        <div>
          <h1>Landolt-Sim</h1>
          <p className="hero-copy">
            OSA Zernike 収差、Styles-Crawford 効果、FFT ベースの PSF、ランドルト環をブラウザ上でリアルタイム合成する網膜像シミュレーター。
          </p>
          <p className="hero-disclaimer">⚠️ 本ツールは、ゼルニケ多項式および波面収差論に基づく工学的な計算結果を表示するシミュレーターです。医学的な診断を目的としたものではなく、眼科受診や眼鏡処方の代わりになるものではありません。</p>
        </div>
      </header>

      <main className="layout-grid">
        <section className="control-panel glass-panel">
          <div className="panel-header-row">
            <div>
              <h2>Optical Controls</h2>
              <p>SCA と高次収差を同時に操作します。</p>
            </div>
            <button
              className="ghost-button"
              type="button"
              onClick={() => {
                setControls((current) => ({
                  ...current,
                  sphere: 0,
                  cylinder: 0,
                  axis: 0,
                  zernike: { ...initialZernike },
                }));
              }}
            >
              <RefreshCcw size={16} />
              Reset
            </button>
          </div>

          <div className="control-group">
            <h3>SCA Prescription</h3>
            <SliderRow label="Sphere S" value={controls.sphere} min={-10} max={10} step={0.25} unit="D" onChange={(value) => updateControl(setControls, "sphere", value)} />
            <SliderRow label="Cylinder C" value={controls.cylinder} min={-6} max={6} step={0.25} unit="D" onChange={(value) => updateControl(setControls, "cylinder", value)} />
            <SliderRow label="Axis Ax" value={controls.axis} min={0} max={180} step={1} unit="°" onChange={(value) => updateControl(setControls, "axis", value)} />
          </div>

          <div className="control-group">
            <h3>Higher-Order Zernike</h3>
            {zernikeControls.map((control) => (
              <SliderRow
                key={control.key}
                label={control.label}
                value={controls.zernike[control.key]}
                min={control.min}
                max={control.max}
                step={control.step}
                unit="μm"
                onChange={(value) => {
                  setControls((current) => ({
                    ...current,
                    zernike: {
                      ...current.zernike,
                      [control.key]: value,
                    },
                  }));
                }}
              />
            ))}
          </div>

          <div className="control-group">
            <h3>System</h3>
            <div className="toggle-row">
              <label htmlFor="sce-toggle">Styles-Crawford Effect</label>
              <input
                id="sce-toggle"
                type="checkbox"
                checked={controls.sceEnabled}
                onChange={(event) => {
                  setControls((current) => ({
                    ...current,
                    sceEnabled: event.target.checked,
                  }));
                }}
              />
            </div>
            <SliderRow label="SCE ρ" value={controls.sceRho} min={0} max={0.4} step={0.01} unit="" onChange={(value) => updateControl(setControls, "sceRho", value)} />
            <SliderRow label="Pupil" value={controls.pupilDiameterMm} min={2} max={8} step={0.1} unit="mm" onChange={(value) => updateControl(setControls, "pupilDiameterMm", value)} />
            <SliderRow label="Wavelength" value={controls.wavelengthNm} min={450} max={650} step={5} unit="nm" onChange={(value) => updateControl(setControls, "wavelengthNm", value)} />
            <div className="slider-row">
              <div className="slider-meta">
                <span>Target FOV</span>
                <strong>{controls.targetFovArcmin.toFixed(0)} arcmin</strong>
              </div>
            </div>
          </div>

          {error ? <p className="status error">{error}</p> : null}
          {pending ? <p className="status">Simulating…</p> : <p className="status ok">Ready</p>}
        </section>

        <section className="viewer-panel glass-panel">
          <div className="panel-header-row viewer-header">
            <div>
              <h2>Retinal Image Viewer</h2>
              <p>波面収差、PSF、ランドルト環、網膜像を同一条件で並列表示します。</p>
            </div>
          </div>

          {result ? (
            <div className="panel-grid">
              <PlotPanel
                title={`Wavefront Aberration (${controls.sphere.toFixed(2)} / ${controls.cylinder.toFixed(2)} / ${controls.axis.toFixed(0)}°)`}
                x={createLinearAxis(result.wavefront.width, -1, 1)}
                y={createLinearAxis(result.wavefront.height, -1, 1)}
                values={result.wavefront.wavefront_um}
                colorScale="Jet"
                axisUnit="normalized pupil"
              />
              <PlotPanel
                title="Point Spread Function"
                x={psfAxis}
                y={psfAxis}
                values={result.psf.data}
                colorScale="Greys"
                logScale
                axisUnit="arcmin"
              />
              <LandoltChartPanel
                title="Original Object (Landolt C Grid)"
                xRange={[result.chart.x[0], result.chart.x[result.chart.x.length - 1]]}
                yRange={[result.chart.y[0], result.chart.y[result.chart.y.length - 1]]}
                placements={result.chart.placements}
                axisUnit="arcmin"
                annotations={chartAnnotations}
              />
              <PlotPanel
                title="Retinal Image Simulation"
                x={result.chart.x}
                y={result.chart.y}
                values={result.retinal_image}
                colorScale={[
                  [0, "#000000"],
                  [1, "#ffffff"],
                ]}
                smooth
                axisUnit="arcmin"
                annotations={chartAnnotations}
                xRange={[result.chart.x[0], result.chart.x[result.chart.x.length - 1]]}
                yRange={[result.chart.y[0], result.chart.y[result.chart.y.length - 1]]}
              />
            </div>
          ) : (
            <div className="empty-state">Simulation result is not available yet.</div>
          )}
        </section>
      </main>
    </div>
  );
}

function buildSimulationRequest(controls: Controls) {
  return {
    optics: {
      wavelength_nm: controls.wavelengthNm,
      pupil_diameter_mm: controls.pupilDiameterMm,
      image_samples: controls.imageSamples,
      target_fov_arcmin: controls.targetFovArcmin,
      pupil_samples: null,
    },
    prescription: {
      sphere_diopters: controls.sphere,
      cylinder_diopters: controls.cylinder,
      axis_degrees: controls.axis,
    },
    additional_coefficients: zernikeControls
      .map((control) => ({
        mode: { n: control.n, m: control.m },
        coefficient_um: controls.zernike[control.key],
        source: "manual",
      }))
      .filter((coefficient) => Math.abs(coefficient.coefficient_um) > 1e-6),
    styles_crawford: {
      enabled: controls.sceEnabled,
      rho: controls.sceRho,
    },
    acuities: landoltAcuities,
    grid_size: 3,
  };
}

function updateControl(
  setControls: React.Dispatch<React.SetStateAction<Controls>>,
  key: keyof Controls,
  value: number,
) {
  setControls((current) => ({
    ...current,
    [key]: value,
  }));
}

function SliderRow(props: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  onChange: (value: number) => void;
}) {
  return (
    <label className="slider-row">
      <div className="slider-meta">
        <span>{props.label}</span>
        <strong>
          {props.value.toFixed(props.step >= 1 ? 0 : 2)}
          {props.unit ? ` ${props.unit}` : ""}
        </strong>
      </div>
      <input
        type="range"
        min={props.min}
        max={props.max}
        step={props.step}
        value={props.value}
        onChange={(event) => props.onChange(Number(event.target.value))}
      />
    </label>
  );
}

function SelectRow(props: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: Array<{ label: string; value: string }>;
}) {
  return (
    <label className="select-row">
      <span>{props.label}</span>
      <select value={props.value} onChange={(event) => props.onChange(event.target.value)}>
        {props.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function StatCard(props: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <article className="stat-card">
      <div className="stat-icon">{props.icon}</div>
      <div>
        <p>{props.label}</p>
        <strong>{props.value}</strong>
      </div>
    </article>
  );
}

function PlotPanel(props: PanelProps) {
  const zMatrix = useMemo(() => toMatrix(props.values, props.x.length, props.y.length), [props.values, props.x.length, props.y.length]);
  const layout = useMemo<Partial<Layout>>(
    () => ({
      title: {
        text: props.title,
        font: { family: "Manrope, sans-serif", size: 14, color: "#18243f" },
        x: 0.02,
      },
      margin: { l: 56, r: 20, t: 42, b: 48 },
      paper_bgcolor: "rgba(255,255,255,0)",
      plot_bgcolor: "#fffaf2",
      autosize: true,
      dragmode: false,
      xaxis: {
        title: { text: props.axisUnit },
        showgrid: true,
        gridcolor: "rgba(24, 36, 63, 0.12)",
        zeroline: false,
        scaleanchor: "y",
        scaleratio: 1,
        ...(props.xRange ? { range: props.xRange } : {}),
      },
      yaxis: {
        title: { text: props.axisUnit },
        showgrid: true,
        gridcolor: "rgba(24, 36, 63, 0.12)",
        zeroline: false,
        ...(props.yRange ? { range: props.yRange } : {}),
      },
      annotations: (props.annotations ?? []).map((annotation) => ({
        x: annotation.x,
        y: annotation.y,
        text: annotation.text,
        showarrow: false,
        xanchor: "center",
        yanchor: "top",
        font: { color: "#d92d20", size: 12, family: "Manrope, sans-serif" },
        bgcolor: "rgba(255,255,255,0.72)",
        bordercolor: "rgba(255,255,255,0.9)",
        borderpad: 2,
      })),
    }),
    [props.annotations, props.axisUnit, props.title],
  );

  const data = useMemo<Data[]>(() => {
    const heatmapValues = props.logScale
      ? zMatrix.map((row) => row.map((value) => Math.log10(1 + value * 1_000_000)))
      : zMatrix;

    return [
      {
        type: "heatmap",
        x: props.x,
        y: props.y,
        z: heatmapValues,
        colorscale: props.colorScale,
        reversescale: props.reverseScale ?? false,
        zsmooth: props.smooth ? "best" : false,
        showscale: false,
        hovertemplate: `${props.axisUnit}: (%{x:.2f}, %{y:.2f})<br>value=%{z:.4g}<extra></extra>`,
      },
    ];
  }, [props.axisUnit, props.colorScale, props.logScale, props.reverseScale, props.smooth, props.x, props.y, zMatrix]);

  return (
    <article className="raster-panel">
      <Plot
        data={data}
        layout={layout}
        config={{
          displayModeBar: false,
          responsive: true,
          staticPlot: true,
        }}
        className="plot-panel"
        useResizeHandler
      />
    </article>
  );
}

function LandoltChartPanel(props: {
  title: string;
  xRange: [number, number];
  yRange: [number, number];
  placements: SimulationResult["chart"]["placements"];
  axisUnit: string;
  annotations?: Array<{ x: number; y: number; text: string }>;
}) {
  const traces = useMemo<Data[]>(() => {
    return props.placements.map((placement) => buildLandoltRingTrace(placement));
  }, [props.placements]);

  const layout = useMemo<Partial<Layout>>(
    () => ({
      title: {
        text: props.title,
        font: { family: "Manrope, sans-serif", size: 14, color: "#18243f" },
        x: 0.02,
      },
      margin: { l: 56, r: 20, t: 42, b: 48 },
      paper_bgcolor: "rgba(255,255,255,0)",
      plot_bgcolor: "#ffffff",
      autosize: true,
      dragmode: false,
      showlegend: false,
      xaxis: {
        title: { text: props.axisUnit },
        showgrid: false,
        zeroline: false,
        scaleanchor: "y",
        scaleratio: 1,
        range: props.xRange,
      },
      yaxis: {
        title: { text: props.axisUnit },
        showgrid: false,
        zeroline: false,
        range: props.yRange,
      },
      annotations: (props.annotations ?? []).map((annotation) => ({
        x: annotation.x,
        y: annotation.y,
        text: annotation.text,
        showarrow: false,
        xanchor: "center",
        yanchor: "top",
        font: { color: "#d92d20", size: 12, family: "Manrope, sans-serif" },
        bgcolor: "rgba(255,255,255,0.72)",
        bordercolor: "rgba(255,255,255,0.9)",
        borderpad: 2,
      })),
    }),
    [props.annotations, props.axisUnit, props.title, props.xRange, props.yRange],
  );

  return (
    <article className="raster-panel">
      <Plot
        data={traces}
        layout={layout}
        config={{
          displayModeBar: false,
          responsive: true,
          staticPlot: true,
        }}
        className="plot-panel"
        useResizeHandler
      />
    </article>
  );
}

function toMatrix(values: number[], width: number, height: number) {
  const matrix: number[][] = [];
  for (let row = 0; row < height; row += 1) {
    const start = row * width;
    matrix.push(values.slice(start, start + width));
  }
  return matrix;
}

function createLinearAxis(samples: number, start: number, end: number) {
  if (samples <= 1) {
    return [start];
  }

  const step = (end - start) / (samples - 1);
  return Array.from({ length: samples }, (_, index) => start + step * index);
}

function createSymmetricAxis(samples: number, span: number) {
  return createLinearAxis(samples, -span / 2, span / 2);
}

function buildLandoltRingTrace(placement: SimulationResult["chart"]["placements"][number]): Data {
  const gapWidth = 1 / placement.acuity;
  const outerRadius = placement.outer_radius_arcmin;
  const innerRadius = (outerRadius * 3) / 5;
  const halfGap = gapWidth / 2;
  const outerAngle = Math.asin(Math.min(halfGap / outerRadius, 1));
  const innerAngle = Math.asin(Math.min(halfGap / innerRadius, 1));
  const outerArc = sampleArc(outerRadius, outerAngle, 2 * Math.PI - outerAngle, 100, false);
  const innerArc = sampleArc(innerRadius, 2 * Math.PI - innerAngle, innerAngle, 100, true);

  const xOuterTop = Math.sqrt(Math.max(outerRadius * outerRadius - halfGap * halfGap, 0));
  const xInnerTop = Math.sqrt(Math.max(innerRadius * innerRadius - halfGap * halfGap, 0));
  const xOuterBottom = xOuterTop;
  const xInnerBottom = xInnerTop;

  const localPoints = [
    ...outerArc,
    { x: xOuterBottom, y: -halfGap },
    { x: xInnerBottom, y: -halfGap },
    ...innerArc,
    { x: xInnerTop, y: halfGap },
    { x: xOuterTop, y: halfGap },
  ];

  const angle = (placement.gap_angle_degrees * Math.PI) / 180;
  const x: number[] = [];
  const y: number[] = [];

  for (const point of localPoints) {
    const rotatedX = point.x * Math.cos(angle) - point.y * Math.sin(angle) + placement.x_arcmin;
    const rotatedY = point.x * Math.sin(angle) + point.y * Math.cos(angle) + placement.y_arcmin;
    x.push(rotatedX);
    y.push(rotatedY);
  }

  return {
    type: "scatter",
    mode: "lines",
    x,
    y,
    fill: "toself",
    fillcolor: "#000000",
    line: {
      color: "#000000",
      width: 0.5,
      shape: "linear",
    },
    hoverinfo: "skip",
  };
}

function sampleArc(radius: number, start: number, end: number, count: number, descending: boolean) {
  const points: Array<{ x: number; y: number }> = [];
  for (let index = 0; index <= count; index += 1) {
    const t = index / count;
    const angle = descending ? start + (end - start) * t : start + (end - start) * t;
    points.push({
      x: radius * Math.cos(angle),
      y: radius * Math.sin(angle),
    });
  }
  return points;
}

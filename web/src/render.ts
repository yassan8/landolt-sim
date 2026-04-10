export type ColorMode = "wavefront" | "grayscale" | "grayscale-log";

type RasterPanelArgs = {
  canvas: HTMLCanvasElement;
  width: number;
  height: number;
  values: number[];
  title: string;
  colorMode: ColorMode;
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
  axisUnit: string;
};

const MARGIN_LEFT = 56;
const MARGIN_RIGHT = 18;
const MARGIN_TOP = 36;
const MARGIN_BOTTOM = 42;

export function drawRasterPanel({
  canvas,
  width,
  height,
  values,
  title,
  colorMode,
  xMin,
  xMax,
  yMin,
  yMax,
  axisUnit,
}: RasterPanelArgs) {
  const context = canvas.getContext("2d");
  if (!context) {
    return;
  }

  const deviceScale = window.devicePixelRatio || 1;
  const cssWidth = canvas.clientWidth || 420;
  const cssHeight = canvas.clientHeight || 360;
  canvas.width = Math.floor(cssWidth * deviceScale);
  canvas.height = Math.floor(cssHeight * deviceScale);
  context.setTransform(deviceScale, 0, 0, deviceScale, 0, 0);

  context.clearRect(0, 0, cssWidth, cssHeight);
  context.fillStyle = "#fffaf2";
  context.fillRect(0, 0, cssWidth, cssHeight);

  const plotWidth = cssWidth - MARGIN_LEFT - MARGIN_RIGHT;
  const plotHeight = cssHeight - MARGIN_TOP - MARGIN_BOTTOM;

  const offscreen = document.createElement("canvas");
  offscreen.width = width;
  offscreen.height = height;
  const imageContext = offscreen.getContext("2d");
  if (!imageContext) {
    return;
  }

  const image = imageContext.createImageData(width, height);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;

  for (let index = 0; index < values.length; index += 1) {
    let normalized = (values[index] - min) / span;
    if (colorMode === "grayscale-log") {
      normalized = Math.log1p(normalized * 4_000) / Math.log1p(4_000);
    }

    const [r, g, b] = mapColor(normalized, colorMode);
    const pixelIndex = index * 4;
    image.data[pixelIndex] = r;
    image.data[pixelIndex + 1] = g;
    image.data[pixelIndex + 2] = b;
    image.data[pixelIndex + 3] = 255;
  }

  imageContext.putImageData(image, 0, 0);
  context.drawImage(offscreen, MARGIN_LEFT, MARGIN_TOP, plotWidth, plotHeight);

  context.strokeStyle = "#18243f";
  context.lineWidth = 1;
  context.strokeRect(MARGIN_LEFT, MARGIN_TOP, plotWidth, plotHeight);

  context.fillStyle = "#18243f";
  context.font = '600 14px Manrope, sans-serif';
  context.fillText(title, MARGIN_LEFT, 20);
  context.font = '12px Manrope, sans-serif';
  context.fillStyle = "#475467";
  context.fillText(axisUnit, cssWidth - MARGIN_RIGHT - 46, cssHeight - 12);

  drawAxis(context, plotWidth, plotHeight, xMin, xMax, yMin, yMax);
}

function drawAxis(
  context: CanvasRenderingContext2D,
  plotWidth: number,
  plotHeight: number,
  xMin: number,
  xMax: number,
  yMin: number,
  yMax: number,
) {
  const ticks = 4;
  context.strokeStyle = "rgba(24, 36, 63, 0.16)";
  context.fillStyle = "#667085";
  context.font = '11px Manrope, sans-serif';

  for (let index = 0; index <= ticks; index += 1) {
    const x = MARGIN_LEFT + (plotWidth * index) / ticks;
    const y = MARGIN_TOP + (plotHeight * index) / ticks;

    context.beginPath();
    context.moveTo(x, MARGIN_TOP);
    context.lineTo(x, MARGIN_TOP + plotHeight);
    context.stroke();

    context.beginPath();
    context.moveTo(MARGIN_LEFT, y);
    context.lineTo(MARGIN_LEFT + plotWidth, y);
    context.stroke();

    const xValue = xMin + ((xMax - xMin) * index) / ticks;
    const yValue = yMax - ((yMax - yMin) * index) / ticks;
    context.fillText(formatTick(xValue), x - 10, MARGIN_TOP + plotHeight + 18);
    context.fillText(formatTick(yValue), 10, y + 4);
  }
}

function formatTick(value: number) {
  if (Math.abs(value) >= 100) {
    return value.toFixed(0);
  }
  if (Math.abs(value) >= 10) {
    return value.toFixed(1);
  }
  return value.toFixed(2);
}

function mapColor(value: number, colorMode: ColorMode): [number, number, number] {
  const normalized = Math.max(0, Math.min(1, value));
  if (colorMode === "grayscale" || colorMode === "grayscale-log") {
    const channel = Math.round(normalized * 255);
    return [channel, channel, channel];
  }

  const fourValue = 4 * normalized;
  const red = Math.round(255 * clamp(Math.min(fourValue - 1.5, -fourValue + 4.5), 0, 1));
  const green = Math.round(255 * clamp(Math.min(fourValue - 0.5, -fourValue + 3.5), 0, 1));
  const blue = Math.round(255 * clamp(Math.min(fourValue + 0.5, -fourValue + 2.5), 0, 1));
  return [red, green, blue];
}

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

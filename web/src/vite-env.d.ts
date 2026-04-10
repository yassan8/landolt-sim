declare module "/pkg/landolt_sim.js" {
  export default function init(): Promise<void>;
  export function sca_to_zernike_js(input: unknown): unknown;
  export function simulate_retinal_image_js(input: unknown): unknown;
}

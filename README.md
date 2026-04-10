# Landolt-Sim

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Wasm](https://img.shields.io/badge/target-WebAssembly-rebeccapurple.svg)](https://webassembly.org/)
[![JIS Standard](https://img.shields.io/badge/Standard-JIS%20T%207309-red.svg)](https://www.jisc.go.jp/)

**Landolt-Sim** は、眼の光学的な収差（波面収差）が網膜像の質に与える影響を、JIS規格準拠のランドルト環を用いて精密にシミュレートするWebアプリケーションです。

Pythonによるプロトタイプで検証された高度な光学ロジックを、Rust + WebAssembly によってブラウザ上でのリアルタイム演算として実現しました。

## 🌟 プロジェクトの核心

本プロジェクトは、単なる画像の加工ではなく、以下の物理プロセスを忠実に再現しています：

1.  **SCAからZernikeへの変換**: 球面(S)・乱視(C)・軸(A)の処方値から、OSA標準に基づいた2次ゼルニケ係数を算出。
2.  **高次収差の付加**: コマ収差や球面収差など、個人の眼特有の複雑な歪みを波面収差としてモデリング。
3.  **物理光学PSFの生成**: 瞳孔関数からFFT（高速フーリエ変換）を用いて、点像分布関数(PSF)を計算。
4.  **JIS準拠チャート**: JIS T 7309に基づいた厳密な幾何形状（外径:内径:切欠き = 5:3:1）のランドルト環グリッドを生成。
5.  **網膜像の合成**: 生成されたチャートとPSFを巡回畳み込みすることで、実際の見え方をシミュレート。

## 🧬 光学スペック

- **波長 ($\lambda$)**: 555 nm (視感度の高い基準波長)
- **瞳孔径 ($D$)**: 可変 (標準 6.0 mm)
- **視野角 (FOV)**: $\pm 120$ arcmin (240分角)
- **サンプリング**: 最大 2048 x 2048 の高解像度FFT計算

## 🛠 技術スタック

- **Core Engine**: Rust (Zernike多項式の生成、FFT演算、画像コンボリューション)
- **Parallelism**: `rustfft` による最適化されたフーリエ変換
- **Frontend**: React + TypeScript (UI層)
- **Communication**: WebAssembly による高速なメモリ共有

## 📸 シミュレーションフロー


*波面収差(左上)からPSF(右上)を導出し、ランドルト環(左下)と合成して最終的な網膜像(右下)を得るまで*

## 🚀 開発者向け情報

Python版のプロトタイプコードは `prototype/` ディレクトリに保管されています。Rust版への移植にあたっては、`numpy` と `matplotlib` のロジックを Rust の `nalgebra` および `Wasm-side rendering` へと移行しています。

### 現在の実装状況

- Rust の Wasm クレートをルートに初期化済み
- `ScaPrescription`, `ZernikeCoefficient`, `WavefrontRequest`, `WavefrontResult` などの基礎データ構造を追加
- SCA (`S, C, Ax`) から OSA 標準 2 次 Zernike 係数 `C₂⁻², C₂⁰, C₂²` への変換を実装
- OSA 正規化 Zernike 多項式と波面収差マップ生成を実装
- Styles-Crawford 効果の Gaussian apodization `A(r) = 10^{-\rho r^2}` を瞳孔振幅に適用する基盤を追加
- `rustfft` による瞳孔関数からの PSF 計算と FFT ベースの巡回畳み込みを実装
- JIS T 7309 準拠のランドルト環 3x3 チャート生成を実装
- `wasm-bindgen` 経由で `sca_to_zernike_js`, `generate_wavefront_js`, `simulate_retinal_image_js` を公開
- React + TypeScript + Vite のフロントエンドを追加し、4 パネル表示とリアルタイムスライダー UI を実装
- React + TypeScript + Vite のフロントエンドを追加し、Plotly ベースの 4 パネル表示とリアルタイムスライダー UI を実装
- グラフは 4 パネルすべて 1:1 アスペクト比で表示
- ランドルト環は左上から右下に向かって視力 0.10, 0.20, 0.30, 0.50, 0.70, 1.00, 1.20, 1.50, 2.00 を固定配置
- スライダー操作中は低解像度、停止後は選択解像度で再計算する高速化を実装
- 開発サーバーは HTTPS `localhost` で起動し、Chrome の secure context 要件に対応

### 次の実装ステップ

1. Canvas パネルへ視力ラベルや PSF のズーム表示を追加
2. 操作中は低解像度、停止後は高解像度に切り替える段階的レンダリングを追加
3. 個別 Zernike 係数セットの保存・読み込みを追加

### セットアップ

```bash
# Node 依存関係の解決
npm install

# Rust コアのテスト
cargo test

# Wasm + React 開発サーバー起動
npm run dev

# Chrome では以下を開く
# https://localhost:5173

# 本番ビルド
npm run build

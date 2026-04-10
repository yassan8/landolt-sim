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

### セットアップ

```bash
# クローンと依存関係の解決
git clone [https://github.com/your-username/landolt-sim.git](https://github.com/your-username/landolt-sim.git)
npm install

# Rust(Wasm)のビルド
wasm-pack build --target web

# アプリケーションの起動
npm run dev

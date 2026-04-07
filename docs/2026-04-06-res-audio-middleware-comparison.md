---
tags:
  - research
  - consideration
  - audio
  - tech-stack
  - kira
  - fmod
  - csound
  - libpd
  - supercollider
  - fundsp
  - tone-js
  - tauri
---
# オーディオミドルウェア再調査

depends-on:
- [技術スタック選定](./2026-04-05-dec-tech-stack.md)

## 調査目的

前回の技術選定では SuperCollider + TidalCycles を採用したが、配布の複雑さ（scsynth, GHCi, SuperDirt の3つの外部依存）が課題。音楽表現力を維持しつつ、より配布しやすい構成がないか再調査する。

優先度: 1. 音楽表現力 > 2. 配布しやすさ > 3. クロスプラットフォーム

## 候補一覧サマリー

| 候補 | GitHub Stars | 最新バージョン | 最終更新 | 配布依存 | ライセンス |
|------|-------------|---------------|---------|---------|-----------|
| A. kira | ~980 | 0.12.0 | 2026-02頃 | なし（pure Rust） | MIT/Apache-2.0 |
| B. FMOD (fmod-rs) | ~50（fmod-oxide） | バインディング複数あり | 2024頃 | FMOD DLL同梱必須 | プロプライエタリ（Indie無料） |
| C. Csound (csound-rs) | ~16 | 0.1.8 | 2020頃（停滞） | Csound DLL同梱必須 | LGPL |
| D. libpd-rs | （未公開） | 0.2.0 | ビルド失敗あり | libpd同梱必須 | BSD |
| E. scsynth バンドル | N/A | SC 3.14 | 2025頃 | scsynth.exe同梱 | GPL-3.0 |
| F. fundsp + midly | ~133（fundsp） | 0.23.0 | 2025-02頃 | なし（pure Rust） | MIT/Apache-2.0 |
| G. Tauri + Tone.js | ~14,700（Tone.js） | Tone.js 14.x / Tauri 2.x | 2026-03 | WebView2（Windows標準） | MIT |

## A. Rust + kira

### 概要

ゲームオーディオ向け Rust ライブラリ。バックエンド非依存で、サウンド再生・ミキシング・エフェクト・クロック同期を提供する（[kira GitHub](https://github.com/tesselode/kira)）。

### 音楽表現力

- **シンセサイズ**: なし。プリレンダリングされた音声ファイル（WAV/OGG/MP4等）の再生が主機能。リアルタイム波形生成は対象外（[kira docs.rs](https://docs.rs/kira/latest/kira/)）
- **パターン/シーケンス**: クロックシステムで「N tick 目に再生開始」という形でタイミング制御が可能。BPM指定可能。ただし TidalCycles のようなパターン変形（`every`, `fast`, `slow`）に相当する機能はない（[kira clock docs](https://docs.rs/kira/latest/kira/clock/index.html)）
- **エフェクト**: ミキサーにエフェクトチェーンを構築可能。リバーブ、ディレイ等の組み込みエフェクトあり（未検証）
- **アダプティブ音楽**: Tween によるパラメータのスムース遷移、クロック同期による楽曲切り替えが可能。ゲームの状態に応じたBGM遷移に向いている

### 配布

pure Rust。追加依存なし。1バイナリに静的リンク可能。

### 評価

サンプル再生ベースのアダプティブ音楽（ゲーム的な「レイヤー切り替え」）には最適。しかしリアルタイムシンセサイズやパターンベースのジェネレーティブ音楽には機能が不足。**本プロジェクトの「キー入力に応じてパターンが変化する」要件にはシンセ機能の欠如が致命的。**

---

## B. Rust + FMOD (fmod-rs / fmod-oxide)

### 概要

ゲーム業界標準のオーディオミドルウェア。FMOD Studio でアダプティブ音楽のオーサリングが可能（[FMOD 公式](https://www.fmod.com/studio/)）。

### Rust バインディングの状態

複数の Rust バインディングが存在するが、いずれも成熟度が低い:
- **fmod-rs** (CAD97/fmod-rs): FMOD Core + Studio バインディング。thread-safe API（[fmod-rs GitHub](https://github.com/CAD97/fmod-rs)）
- **fmod-oxide** (melody-rs/fmod-oxide): よりRust的なバインディング。FMOD 2.0.2+ 対応（[fmod-oxide GitHub](https://github.com/melody-rs/fmod-oxide)）
- **libfmod** (lebedec/libfmod): FFIラッパー。FMOD 2.02.22 対応（[libfmod GitHub](https://github.com/lebedec/libfmod)）

### 音楽表現力

- **シンセサイズ**: FMOD自体にシンセ機能はない。サンプルベースの再生が前提
- **パターン/シーケンス**: FMOD Studio のイベントシステムでアダプティブ音楽を設計可能。パラメータ駆動で楽曲が変化する仕組み。ただしパターン変形のようなジェネレーティブ機能はなく、**事前にFMOD Studioでオーサリングしたものを再生する方式**
- **エフェクト**: 業界標準レベルのエフェクトチェーン

### ライセンス

プロプライエタリ。Indie ライセンスは開発予算 $600k 未満、会社売上 $200k 未満で無料（[FMOD ライセンス](https://www.fmod.com/)）。

### 配布

FMOD のネイティブ DLL/dylib を同梱する必要がある。バイナリサイズは数MB増加（未検証）。

### 評価

アダプティブ音楽の「業界標準」だが、あくまでサンプルベース。FMOD Studio での事前オーサリングが前提で、**リアルタイムにパターンを生成・変形する用途には向かない**。Rust バインディングも複数乱立しており安定性に不安。

---

## C. Rust + Csound (csound-rs)

### 概要

Csound は1985年から続く音声合成言語。組み込み可能なライブラリ版（libcsound）がある（[Csound GitHub](https://github.com/csound/csound)）。

### Rust バインディングの状態

- **csound-rs**: v0.1.8。GitHub stars 16。最終更新は2020年頃で**事実上メンテナンス停滞**（[csound-rs GitHub](https://github.com/neithanmo/csound-rs)）
- docs.rs でビルド失敗の報告あり（[csound 0.1.8 docs.rs](https://docs.rs/crate/csound/latest)）
- 動的リンク（libcsound64）のみ対応。静的リンク不可

### 音楽表現力

- **シンセサイズ**: 極めて強力。数百のオペコードで任意の音声合成が可能
- **パターン/シーケンス**: Csound のスコア言語はイベントリスト形式。TidalCycles のような「パターンの高階変形」は**言語レベルでは未対応**。Strudel（TidalのWeb版）との実験的統合は存在する（[Strudel + Csound](https://strudel.cc/learn/csound/)）が、Csound単体のスコア記述力は Tidal に大きく劣る
- **エフェクト**: 非常に豊富。リバーブ、フィルタ、グラニュラー合成等

### 配布

Csound ランタイム（DLL/dylib, ~20-30MB（未検証））の同梱が必要。

### 評価

シンセサイズ能力は最高レベルだが、**Rust バインディングが事実上放棄状態**。パターン記述力は Tidal に遠く及ばない。配布依存も大きい。現状では採用困難。

---

## D. Rust + libpd-rs (Pure Data embedded)

### 概要

Pure Data (Pd) を Rust プロセスに組み込むライブラリ（[libpd-rs GitHub](https://github.com/alisomay/libpd-rs)）。

### 最新状態

- v0.2.0 が最新だが、**docs.rs でのビルドが失敗している**（[libpd-rs 0.2.0 docs.rs](https://docs.rs/crate/libpd-rs/latest)）
- libpd-sys (FFI層) は v0.3.4（[libpd-sys docs.rs](https://docs.rs/crate/libpd-sys/latest)）
- メンテナンス頻度は低い

### .pd ファイルをテキストで書く方式

Pd パッチは内部的にはテキスト形式（`.pd` ファイル）で記述可能。ビジュアルエディタなしでもパッチを作成・編集できる。ただし:
- 構文は座標情報を含む特殊フォーマットで、人間が直接書くのは非常に煩雑（未検証）
- 事前に作成した .pd ファイルをアプリにバンドルする方式なら実用的

### 音楽表現力

- **シンセサイズ**: Pd のフル機能が使える。SC に匹敵する合成能力
- **パターン/シーケンス**: Pd 自体にパターン言語はない。シーケンスはデータフローで構築する必要がある
- **エフェクト**: 豊富

### 配布

libpd をスタティックリンクすれば1バイナリに近づけるが、.pd パッチファイルの同梱が必要。バイナリサイズは数MB増加（未検証）。

### 評価

技術的には面白いが、**ビルド失敗問題が未解決**、パターン記述力不足、.pd ファイルの手書きが非現実的、という3重の問題がある。

---

## E. Rust + scsynth バンドル（Tidal なし）

### 概要

scsynth.exe のみをアプリにバンドルし、Rust から OSC で直接制御する方式。sclang, IDE, TidalCycles, GHCi は全て不要（[SC Client vs Server](https://doc.sccode.org/Guides/ClientVsServer.html)）。

### Rust からの制御

- **rosc** クレート: pure Rust の OSC ライブラリ（[rosc crates.io](https://crates.io/crates/rosc)）
- **rosc_supercollider**: SC 拡張 OSC 対応版（[rosc_supercollider crates.io](https://crates.io/crates/rosc_supercollider)）
- **sorceress**: Rust から SC を制御する高レベルクレート。v0.2.0、ただし4年以上更新なし（[sorceress GitHub](https://github.com/ooesili/sorceress)）
- SC の Node Messaging Protocol で SynthDef の読み込み・ノード操作・パラメータ変更が全て可能

### 音楽表現力

- **シンセサイズ**: scsynth のフル能力。数百の UGen が利用可能
- **パターン/シーケンス**: **自前実装が必要**。Tidal のパターンエンジンに相当するものを Rust で書く必要がある。これが最大の課題
- **エフェクト**: scsynth のフルエフェクトチェーン

### 配布

- scsynth.exe + プラグイン DLL の同梱が必要。サイズは ~50-100MB（未検証）
- **GPL-3.0 ライセンス**: scsynth をバンドルするとアプリ全体が GPL になる（[SC GPL 議論](https://scsynth.org/t/live-mixed-music-and-gpl/9577)）。これは重大な制約

### 評価

シンセ能力は最高だが、**GPL 汚染**と**パターンエンジンの自前実装コスト**が問題。配布サイズも大きい。現在の SC+Tidal 構成から Tidal を除いただけで、配布問題は scsynth の同梱で部分的にしか解決しない。

---

## F. Rust + fundsp + midly (pure Rust)

### 概要

fundsp は Rust ネイティブの DSP ライブラリ。インライン記法でオーディオグラフを記述できる（[fundsp GitHub](https://github.com/SamiPerttu/fundsp)）。

### 最新状態

- v0.23.0（37バージョン公開済み）。2025-02 頃更新。アクティブにメンテナンスされている（[fundsp crates.io](https://crates.io/crates/fundsp)）
- GitHub stars ~133

### 音楽表現力

- **シンセサイズ**: 波形生成、フィルタ、エンベロープ、グラニュラー合成等を pure Rust で実現。コンビネータ記法が強力:
  ```rust
  // 例: FM合成 + ローパスフィルタ
  sine_hz(440.0) * sine_hz(5.0) >> lowpass_hz(1000.0, 1.0)
  ```
  （[fundsp README](https://github.com/SamiPerttu/fundsp/blob/master/README.md)）
- **パターン/シーケンス**: `Sequencer` コンポーネントでノードの動的追加・削除が可能。開始/停止時刻、フェードイン/アウトを指定できる（[fundsp docs.rs](https://docs.rs/fundsp/latest/fundsp/)）。ただし **TidalCycles のようなパターン変形DSLは存在しない**。シーケンスロジックは全て自前実装
- **エフェクト**: リバーブ、ディレイ、フィルタ、コンプレッサー等。SC ほどではないが実用的
- **midi_fundsp**: MIDI入力と fundsp を組み合わせるクレートが存在（[midi_fundsp lib.rs](https://lib.rs/crates/midi_fundsp)）

### 配布

**完全に1バイナリ**。外部依存ゼロ。バイナリサイズは通常の Rust アプリ + 数百KB 程度（未検証）。

### 評価

配布の観点では最高。シンセ能力も実用レベル。最大の課題は**パターンエンジンを自前で実装する必要がある**こと。ただし、TidalCycles の全機能は不要で「キー入力に応じてパターンパラメータを変える」程度なら、シンプルなステップシーケンサー + パラメータ変調で十分な可能性がある。

---

## G. Tauri + Tone.js + Web Audio

### 概要

Web Audio API ベースの音楽フレームワーク Tone.js を Tauri でデスクトップアプリ化する構成（[Tone.js 公式](https://tonejs.github.io/)、[Tauri 公式](https://v2.tauri.app/)）。

### 音楽表現力

- **シンセサイズ**: Tone.js に各種シンセサイザー（FM, AM, Mono, Poly 等）が組み込み。Web Audio API の OscillatorNode, BiquadFilterNode 等も利用可能
- **パターン/シーケンス**: `Tone.Transport` でBPM同期のスケジューリング、`Tone.Sequence`, `Tone.Pattern`, `Tone.Loop` でパターンベースのシーケンスが可能（[Tone.js Transport Wiki](https://github.com/tonejs/tone.js/wiki/Transport)）。**7候補中、パターンシーケンスの組み込みサポートが最も充実**
- **エフェクト**: リバーブ、ディレイ、ディストーション、フィルタ等。DAW レベルのエフェクトチェーン

### レイテンシ

Web Audio API のレイテンシは `lookAhead`（デフォルト100ms）+ `updateInterval` で構成される。`lookAhead` を0にすれば低レイテンシ化可能だがグリッチのリスクが増す（[Tone.js Performance Wiki](https://github.com/Tonejs/Tone.js/wiki/Performance)）。BGM 用途なら 10-50ms のレイテンシは許容範囲。

### 配布

- Tauri 2.0 でインストーラー作成。バイナリサイズは最小 ~600KB（Tauri本体）+ フロントエンドアセット（[Tauri App Size](https://v2.tauri.app/concept/size/)）
- Windows は WebView2 を使用（Windows 10/11 標準搭載）。オフラインインストーラーだと +127MB（[Tauri Windows Installer](https://v2.tauri.app/distribute/windows-installer/)）
- NSIS / WiX でインストーラー生成可能（[Tauri Distribute](https://v2.tauri.app/distribute/)）

### 評価

パターンシーケンス機能が最も充実しており、配布も容易。シンセ能力は SC に劣るが BGM 用途には十分。**最大の懸念は Rust + JS の2言語構成になること**と、Web Audio API の制約（AudioWorklet のスレッドモデル等）。

---

## 比較マトリクス

### 音楽表現力

| 候補 | シンセサイズ | パターン/シーケンス | エフェクト | 総合 |
|------|------------|-------------------|----------|------|
| A. kira | ×（再生のみ） | △（クロック同期） | ○ | **D** |
| B. FMOD | ×（再生のみ） | △（イベントベース） | ◎ | **C** |
| C. Csound | ◎ | △（スコア言語） | ◎ | **B** |
| D. libpd | ◎ | △（データフロー） | ◎ | **B** |
| E. scsynth | ◎ | ×（自前実装） | ◎ | **B-** |
| F. fundsp | ○ | △（Sequencer） | ○ | **C+** |
| G. Tone.js | ○ | ◎（Transport/Sequence/Pattern） | ○ | **A-** |

### 配布しやすさ

| 候補 | 外部依存 | バイナリ形態 | サイズ目安 | 総合 |
|------|---------|------------|----------|------|
| A. kira | なし | 1バイナリ | ~5MB | **A** |
| B. FMOD | FMOD DLL | バイナリ + DLL | ~10MB | **B** |
| C. Csound | Csound DLL | バイナリ + DLL | ~30MB | **C** |
| D. libpd | libpd + .pd | バイナリ + パッチ | ~10MB | **C+** |
| E. scsynth | scsynth + plugins | バイナリ + 多数 | ~100MB | **D** |
| F. fundsp | なし | 1バイナリ | ~5MB | **A** |
| G. Tone.js | WebView2 | インストーラー | ~3-10MB | **A-** |

### クロスプラットフォーム

| 候補 | Windows | macOS | Linux | 備考 |
|------|---------|-------|-------|------|
| A. kira | ○ | ○ | ○ | cpal バックエンド |
| B. FMOD | ○ | ○ | ○ | ネイティブ対応 |
| C. Csound | ○ | ○ | ○ | 各OS用ビルド必要 |
| D. libpd | ○ | ○ | ○ | ビルド問題あり |
| E. scsynth | ○ | ○ | ○ | 各OS用バイナリ必要 |
| F. fundsp | ○ | ○ | ○ | pure Rust |
| G. Tone.js | ○ | ○ | ○ | WebView依存 |

---

## 推奨順位

### 第1推奨: F. Rust + fundsp（パターンエンジン自作）

**理由**: 1バイナリ配布が可能で、シンセサイズ能力も実用的。パターンエンジンは自前実装だが、本プロジェクトの要件（「キー入力速度/パターンに応じてBGMパラメータが変わる」）は TidalCycles のフル機能は不要。ステップシーケンサー + パラメータ変調で十分実現可能。

- 配布: 完全1バイナリ、依存ゼロ
- 表現力: シンセ○、パターンは自作だが要件に対しては十分
- リスク: パターンエンジンの設計・実装コスト。ただし段階的に拡張可能

### 第2推奨: G. Tauri + Tone.js

**理由**: パターン/シーケンス機能が最も充実しており、「音楽アプリ」としての完成度を最も早く高められる。Tauri による配布も容易。2言語（Rust + JS/TS）構成がデメリットだが、音楽ロジックを JS 側に寄せれば Rust 側はシンプルになる。

- 配布: インストーラー ~3-10MB、WebView2 は Windows 標準
- 表現力: パターンシーケンス◎、シンセ○
- リスク: 2言語構成、Web Audio のレイテンシ（BGM用途なら許容）

### 第3推奨: A. Rust + kira + fundsp 組み合わせ

**理由**: kira のアダプティブ音楽機能（クロック同期、Tween、ミキサー）と fundsp のシンセ機能を組み合わせる。kira の `StreamingSoundData` で fundsp の出力をストリーミング入力できる可能性がある（未検証）。

- 配布: 1バイナリ
- 表現力: 両方の強みを活かせる可能性
- リスク: 2ライブラリの統合が未検証

### 第4推奨: E. Rust + scsynth バンドル

**理由**: シンセ能力は最高だが、GPL 汚染・配布サイズ・パターンエンジン自作の3重苦。音楽表現力を最重視する場合のみ。

### 非推奨

- **B. FMOD**: サンプル再生ベースで本プロジェクトの要件に合わない。事前オーサリング前提
- **C. Csound**: Rust バインディングが停滞。ビルド問題
- **D. libpd**: ビルド失敗問題、パターン記述力不足

---

## 次のアクション（提案）

1. **fundsp の PoC**: fundsp で簡単なシーケンサー + シンセを実装し、音楽表現力が要件を満たすか検証
2. **kira + fundsp 統合の検証**: StreamingSoundData 経由で fundsp 出力を kira に流せるか確認
3. **Tauri + Tone.js の PoC**（並行）: JS/TS でパターンシーケンスの prototype を作り、表現力とレイテンシを体感

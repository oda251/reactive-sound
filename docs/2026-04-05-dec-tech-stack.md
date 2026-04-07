---
tags:
  - research
  - consideration
  - decision
  - tech-stack
  - supercollider
  - tidalcycles
  - rust
  - egui
---
# 技術スタック選定

depends-on:
- [プロジェクト概要](./2026-04-05-dec-project-overview.md)

## 決定事項

| レイヤー | 採用技術 | バージョン目安 |
|---------|---------|---------------|
| 音声合成 | SuperCollider (scsynth + SuperDirt) | SC 3.13+ |
| 作曲/パターン | TidalCycles | 1.9+ |
| アプリケーション | Rust | 1.75+ |
| GUI | egui (eframe) | 0.27+ |
| OSC通信 | rosc クレート | （未検証） |
| キーボード検知 | rdev or device_query クレート | （未検証） |

## SuperCollider

### 採用理由

- リアルタイム音声合成のデファクトスタンダード（[SuperCollider 公式](https://supercollider.github.io/)）
- scsynth は OSC (UDP) で外部から制御可能。ヘッドレス運用に適している（[SC Server Architecture](https://doc.sccode.org/Guides/ClientVsServer.html)）
- TidalCycles のバックエンドとして SuperDirt が scsynth 上で動作する（[SuperDirt GitHub](https://github.com/musikinformatik/SuperDirt)）

### scsynth vs sclang

- **scsynth**: 音声合成サーバー。OSC で直接制御可能。本プロジェクトではこちらを使う
- **sclang**: SuperCollider の言語インタプリタ。SuperDirt の起動に必要だが、直接的なプログラミングは不要

## TidalCycles

### 採用理由

- パターンベースのライブコーディング言語。リズム・メロディの記述力が高い（[TidalCycles 公式](https://tidalcycles.org/)）
- `fast`, `slow`, `every` などの関数でパターンをリアルタイムに変形できる（[Tidal Pattern Transformations](https://tidalcycles.org/docs/reference/alteration)）
- 音声合成（SC）とパターン記述（Tidal）は関心の分離ができている

### 制御方式

- TidalCycles は GHCi（Haskell REPL）上で動作する
- エディタプラグイン（VS Code, Vim等）は GHCi の **stdin にHaskellコードを流し込む** ことで制御している（[tidal-vim の実装](https://github.com/tidalcycles/vim-tidal)）
- 本プロジェクトでも同様に、Rust から `std::process::Command` で GHCi プロセスを起動し、stdin 経由で Tidal コードを送信する

### 検討事項

- Tidal は本来「人間がリアルタイムにコードを書く」用途で設計されている。プログラムからの自動制御は本来の用途外だが、stdin 経由でのコード送信は技術的に確立されている（未検証）
- GHCi プロセスのライフサイクル管理（起動・再起動・エラーハンドリング）が必要

## Rust

### 採用理由

- プロセス管理、OSC通信、キーボードフック、GUI を1つの言語で統合できる
- `std::process::Command` で外部プロセス（scsynth, GHCi）の起動・stdin制御が可能
- `rosc` クレートで OSC 通信が可能（[rosc crates.io](https://crates.io/crates/rosc)）（未検証）
- システムトレイ常駐アプリの実装実績がある（`tray-item` クレート等）（未検証）

## egui

### 採用理由

- Pure Rust で完結。Web技術への依存なし（[egui GitHub](https://github.com/emilk/egui)）
- スライダー、ドラッグ値、カスタムウィジェットなど、オーディオパラメータ操作に必要なUI部品が揃っている
- オーディオツールでの採用実績（[eframe examples](https://github.com/emilk/egui/tree/master/examples)）
- 将来的により高機能な GUI が必要になった場合、iced や Tauri+Web への移行パスがある

### Tauri を採用しなかった理由

- Tauri は Rust バックエンド + WebView（HTML/JS）フロントエンドの構成。Pure Rust ではない
- 本プロジェクトでは Web 技術を使う必然性がなく、egui で十分

## 検討した代替スタック

### libpd-rs（Pure Data 組み込み）

- Pd を Rust プロセスに直接組み込める。アーキテクチャが最もシンプル（1プロセス、IPC不要）（[libpd-rs GitHub](https://github.com/alisomay/libpd-rs)）
- **不採用理由**: Pd はビジュアルプログラミングが前提。テキストベースでパターンを記述したいという要件に合わない。また TidalCycles のパターン言語を使いたいという目的がある

### Sonic Pi

- Ruby ベースのライブコーディング環境。OSC `/run-code` エンドポイントで外部制御可能（[Sonic Pi OSC](https://sonic-pi.net/)）
- **不採用理由**: GUI プロセスの起動が必須でヘッドレス非対応。制御トークンをログファイルからパースする必要があり不安定（未検証）

### Overtone (Clojure)

- Clojure で SC を制御するライブコーディング環境（[Overtone GitHub](https://github.com/overtone/overtone)）
- **不採用理由**: JVM 依存が増える。開発が停滞している

### FoxDot / Renardo (Python)

- Python ベースで SC を制御（[Renardo GitHub](https://github.com/e-lie/renardo)）
- **不採用理由**: Python + SC で結局マルチプロセス構成。エコシステムが小さい

### Rust ネイティブ音声 (cpal + fundsp)

- Rust のみで音声合成まで完結（[fundsp docs.rs](https://docs.rs/fundsp)）
- **不採用理由**: 音楽ロジック（シーケンシング、スケール、パターン）を全て自前実装する必要がある。初心者には非現実的

### Tauri + Tone.js

- Web Audio API ベース。Tauri で デスクトップアプリ化
- **不採用理由**: レイテンシ（10-50ms）がネイティブ（~5ms）より大きい。BGM用途なら許容範囲だが、Web Audio の表現力は SC に劣る（未検証）

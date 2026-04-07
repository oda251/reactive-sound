# Reactive BGM

アーキテクチャと設計方針は README.md を参照。

## 開発ガイドライン

- engine クレートは FC/IS 構成。core/ に副作用を入れない。shell/ に純粋ロジックを入れない。
- 層の境界は `InputEvent`, `Score`, `ScoreProvider` で型安全に定義。直接の具体型依存を避ける。
- オーディオコールバック（hot path）ではヒープ確保・ブロッキングロックを避ける。
- Faust DSP は `engine/dsp/*.dsp` に定義し、build.rs で自動コンパイル。
- 設計ドキュメントは `docs/` 配下に documentation スキルのフォーマットで管理する。

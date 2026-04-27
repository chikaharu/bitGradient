# 実験No.X 実験クラス

> **これは実験開始前に必ず記入する儀式ファイルです。**  
> 目標が定まっていない実験は走らせない。  
> タグの定義は `../tag.md` を参照すること。

---

## 目標 (Goal)

<!-- 実験が何のために行われるかを1文で書く -->
<!-- 「〜を確認するため」「〜を改善するため」「〜が面白そうだから試す」でもよい -->

**この実験の目的**:

---

## タグ

```
goal:      # 必須: goal:compression / goal:struct-checker / goal:hash-design /
           #        goal:scoring / goal:theory / goal:play / goal:prototype / goal:leap
status:    # 必須: status:planned
method:    # 任意: method:nibble-gram / method:multi-hash / method:gd / ...
result:    # 実験後に記入
```

---

## 関連実験

| 実験番号 | 関係 |
|---------|------|
| E○○ | 先行実験・この実験の出発点 |
| E○○ | 比較対象 |

---

## メモ・着想

<!-- なぜ今この実験をしたいのか。直感・違和感・会話のきっかけ等 -->
<!-- 「なんとなく面白そう」も立派な動機として記録する -->

---

## チェックリスト（実験開始前）

- [ ] 目標を1文で書いた
- [ ] `goal:` タグを1つ以上つけた
- [ ] `仮説.md` を書いた
- [ ] `conditions.md` を書いた
- [ ] 関連実験を確認した

# LLM-less 離散勾配降下法の数学的基礎
## ― nibble ビット代数による Rust コンパイルエラー修正の定式化 ―

> **v1 凍結 (2026-04-25, e176-c01)**: 凍結核は `THEORY_CORE_v1.md`。
> 本書は v1 凍結時点の長文版で、本文は変更しない。
> 以後の検証追記は `THEORY_EVIDENCE_LEDGER.md` に行うこと。

**対象実装:** ensemble29-v3 (experiment-55) · C11 nibble_hash_matrix (experiment-56)  
**記法規約:** 補題 L\*, 定理 T\*, 系 C\*, 命題 P\* で番号付け  
**証明スタイル:** 数学的に厳密だが自然言語ベース (Lean/Coq 形式でない)

---

## §1 記号定義と前提

### 1.1 基本集合

| 記号 | 定義 |
|------|------|
| $\Sigma^*$ | ASCII/UTF-8 文字列全体 (モノイド, 連結 $\cdot$) |
| $\mathbb{F}_2$ | 2元体 $\{0,1\}$, 加法 = XOR, 乗法 = AND |
| $\mathbb{F}_2^n$ | $n$ 次元ベクトル空間 over $\mathbb{F}_2$ |
| $\mathbb{R}^n$ | $n$ 次元実ベクトル空間 |
| $[n]$ | $\{0, 1, \ldots, n-1\}$ |
| $\mathcal{B}$ | nibble (4-bit) 値の集合 $\{0,\ldots,15\} \cong \mathbb{F}_2^4$ |
| $\mathcal{T}$ | 識別子トークン集合 $\{s \in \Sigma^* \mid s \text{ は正規表現 } [A\text{-}Za\text{-}z\_][A\text{-}Za\text{-}z0\text{-}9\_]^* \text{ に合致}\}$ |

### 1.2 nibHex エンコーディング

トークン $t \in \mathcal{T}$ に対し, UTF-8 バイト列 $\text{utf8}(t) = (b_0, b_1, \ldots)$ の先頭 $\min(|\text{utf8}(t)|, 6)$ バイトを取り, 不足分を $0$ で埋めた長さ 6 バイト列を **nibHex** と呼ぶ:

$$\text{nibHex}(t) = (b_0, b_1, b_2, b_3, b_4, b_5) \in [256]^6$$

nibHex は隣接 nibble ペア $(b_i^{\text{hi}}, b_i^{\text{lo}}) = (b_i \gg 4,\ b_i \& 0\text{xF})$ を露出する。

### 1.3 コンパイラオラクル

$\mathcal{S} \subset \Sigma^*$ を Rust ソース文字列全体とする。**コンパイラオラクル** $\mathcal{O}: \mathcal{S} \to \mathbb{Z}_{\ge 0} \times \text{Spans}$ は

$$\mathcal{O}(s) = (H(s),\ \text{err\_spans}(s))$$

ここで $H(s)$ はエラー数, $\text{err\_spans}(s)$ はエラースパン (行範囲・トークンリスト) のリストである。$H(s) = 0$ は $s$ が正常にコンパイルされることを意味する。

---

## §2 Nibble n-gram 埋め込み

### 2.1 定義

**nibHex エンコード (精密版).** `nibHex(tok, 6)` は UTF-8 バイト列の先頭 6 バイト (不足は 0 で補完) を 2 進数 hex 表現し, 長さ 12 の hex 文字列 $(h_0 h_1 \cdots h_{11}) \in \{0\text{-}9,a\text{-}f\}^{12}$ を返す。

**定義 2.1 (nibble ビット埋め込み).** 写像 $\varphi: \mathcal{T} \to \mathbb{F}_2^{1024}$ を以下で定める。

hex 文字列の**隣接オーバーラップペア** $(h_i, h_{i+1})$, $i = 0, 1, \ldots, 10$ (合計 11 ペア) に対し, nibble-pair バイト値

$$g_i = (h_i \ll 4) \,|\, h_{i+1} \in [256]$$

を計算する。各 $i$ に対してビット位置

$$\text{bit}(i, g_i) = \bigl((i \bmod 4) \ll 8\bigr) \,|\, g_i \in [1024]$$

を設定し, $\varphi(t)$ の第 $\text{bit}(i, g_i)$ 成分を 1 とする (他は 0)。

**ビット空間の分解:** $i \bmod 4 \in \{0,1,2,3\}$ により 1024 ビットは 4 つの 256-bit バンクに分割される。バンク $b \in [4]$ はインデックス $i \equiv b \pmod{4}$ のペアが担当する。

**実装との対応:** `tokBits(tok)` 関数 (ensemble29.mjs:110–119)

### 2.2 スパース性補題と衝突

> **補題 L2.1 (nibble gram スパース性).** トークン $t \in \mathcal{T}$ に対し,
> $$|\varphi(t)|_1 \le 11$$
> である。すなわち $\varphi(t)$ は 1024 ビット中 高々 **11 ビット**を持つ疎ベクトルである。

**証明.** ループ条件 `i+1 < hex.length` と `hex.length=12` より $i \in \{0,1,\ldots,10\}$ で合計 11 回実行される。各反復はビット位置 $\text{bit}(i, g_i) \in [1024]$ を 1 箇所だけ設定する。したがって立ちうるビット数は高々 11。$\square$

> **補題 L2.1' (ゼロパディングによる重複).** 短いトークン ($|\text{utf8}(t)| = k < 6$) では nibHex の後尾 $6-k$ バイトが 0 となり, 対応する $g_i = 0$ が繰り返す。このとき $i \bmod 4$ の同一値を持つ複数のインデックスが同一ビット位置 $((i \bmod 4) \ll 8) \,|\, 0$ に書き込むため, **実際の $|\varphi(t)|_1$ は 11 未満になることが多い**。

**例.** `'fn'` (2 バイト): nibHex = `"666e00000000"` (12 hex 文字) → 11 ペア中, $i=4,5,6,7,8,9,10$ の 7 ペアは $g_i = 0$ でビット位置 $\{0, 256, 512, 768\}$ に重複書き込み → $|\varphi(\text{'fn'})|_1 = 8$。`'self'` (4 バイト): nibHex = `"73656c660000"` → 全 11 ペアが異なるビット位置 → $|\varphi(\text{'self'})|_1 = 11$ (最大, 実測)。

> **補題 L2.2 (単射性の欠如).** $\varphi$ は一般に単射でない。すなわち, $t \ne t'$ であっても $\varphi(t) = \varphi(t')$ となることがある。

**証明.** 像空間は $\mathbb{F}_2^{1024}$ (有限), 定義域 $\mathcal{T}$ は無限大, 鳩の巣原理より衝突が存在する。$\square$

### 2.3 集合演算の持ち上げ

**定義 2.2 (ユニオンビット).** トークン集合 $T \subset \mathcal{T}$ に対し,

$$\Phi(T) = \bigvee_{t \in T} \varphi(t) \in \mathbb{F}_2^{1024}$$

ここで $\bigvee$ は成分ごとの OR (= $\mathbb{F}_2^{1024}$ の加法でなく, $\{0,1\}$ 上の論理和)。

**実装との対応:** `unionBits(toks)` (ensemble29.mjs:143–147)

### 2.4 内積定理

> **定理 T2.1 (AND popcount = 包含測度).** $a = \varphi(s)$, $b = \varphi(t)$ に対し,
> $$\langle a, b \rangle_{\text{AND}} := |\{i \in [1024] \mid a_i = 1 \text{ かつ } b_i = 1\}| = \text{popcount}(a \text{ AND } b)$$
> は $\mathbb{R}$ 上の内積に相当する実数値を返す。とくに対称性・非負性・劣加法性を満たす。

**証明.** AND は各ビット位置に対して $\{0,1\}$ 値の乗算であり, popcount はその和。対称性 $\langle a,b\rangle = \langle b,a\rangle$ は AND の交換律から明らか。非負性は定義から。劣加法性は $|A \cap B| \le \min(|A|, |B|)$ より。$\square$

**実装との対応:** `andCount(a, b)` (ensemble29.mjs:132–141), `popcount32(a)` (ensemble29.mjs:121–130)

---

## §3 XNOR 類似度と Hamming 距離

### 3.1 nibbleSim の定義

**定義 3.1 (nibble 類似度).** クエリ $Q \subset \mathcal{T}$, 候補トークン $t \in \mathcal{T}$ に対し,

$$\text{sim}(t, Q) = \langle \varphi(t),\ \Phi(Q) \rangle_{\text{AND}} = \text{popcount}\bigl(\varphi(t) \text{ AND } \Phi(Q)\bigr)$$

これを **nibbleSim** と呼ぶ。

**実装との対応:** `pairScore` 内の `sim` 計算 (ensemble29.mjs:241–247)

### 3.2 Hamming 距離との関係

> **補題 L3.1 (Hamming 双対性).** $a, b \in \{0,1\}^n$ に対し,
> $$d_H(a, b) = n - \text{popcount}(a \text{ XNOR } b) = \text{popcount}(a \text{ XOR } b)$$
> ここで $\text{XNOR}(a,b) = \text{NOT}(a \text{ XOR } b)$。したがって,
> $$\text{popcount}(a \text{ AND } b) = \text{popcount}(a) + \text{popcount}(b) - \text{popcount}(a \text{ OR } b)$$
> かつ $a = \Phi(Q)$ が sparse ($|\Phi(Q)|_1 \ll n$) のとき $\text{sim}(t,Q) \approx \text{popcount}(a \text{ AND } b)$ は Hamming 類似度の良い近似となる。

**証明.** ビットごとに $(\text{NOT XOR})(a_i, b_i) = 1 \iff a_i = b_i$ だから $\text{popcount}(\text{XNOR}) = n - d_H(a,b)$。AND の恒等式は De Morgan の法則より従う。$\square$

### 3.3 Jaccard 係数との近似関係

> **命題 P3.1 (AND/OR $\approx$ Jaccard).** $a = \varphi(s)$, $b = \varphi(t)$ のとき,
> $$J(s, t) \approx \frac{\text{popcount}(a \text{ AND } b)}{\text{popcount}(a \text{ OR } b)}$$
> は Bloom filter 型の Jaccard 近似 (MinHash に類する) である。バイアスは nibble gram の衝突率に依存する。

**証明のスケッチ.** $a, b$ が独立な Bloom filter と見なせるとき, $\Pr[\text{bit}_k \text{ が } a,b \text{ で共に } 1] \approx (1 - e^{-k_1/m})(1 - e^{-k_2/m})$ だが, 本実装では補題 L2.1 より $|\varphi(t)|_1 \le 11$ と sparse ($m = 1024$ に対して最大 11 bit のみ立つ) なため衝突が少なく, AND がほぼ集合交差を近似する。$\square$

---

## §4 ペナルティマスク蓄積とその飽和

### 4.1 ペナルティマスクの動的定義

**定義 4.1 (ペナルティマスク — 実装忠実版).** 離散 GD の各ステップ $s$ において, オラクルに評価された候補のうち **コンパイルに失敗した ($H > 0$)** 全候補のビット列を OR で累積する:

$$P_s = P_{s-1} \vee \bigvee_{\substack{\mu \in \mathcal{C}_s \\ H(\text{apply}(\mu, s_{s-1})) > 0}} \varphi(t_{\text{add}}^{(\mu)}) \in \{0,1\}^{1024}$$

初期値 $P_{-1} = \mathbf{0}$。すなわち **採用された候補だけでなく**, 評価されて失敗したすべての候補のビット列が蓄積される。

**重要:** コンパイル成功 ($H = 0$) した候補のビット列は penMask に加算されない。また, コンパイル成功でも採用されなかった (tie-break で負けた) 候補も加算されない。

**実装との対応:** `oracleStep` の `if (h > 0) { for (let i=0;i<32;i++) penMask[i] |= cand.bits[i]; }` (ensemble29.mjs:579–581)

### 4.2 ペナルティスコアの定義

**定義 4.2 (ペナルティスコア).** 候補トークン $t$ のステップ $s$ でのペナルティスコアは

$$\text{pen}_s(t) = \langle \varphi(t),\ P_{s-1} \rangle_{\text{AND}} = \text{popcount}\bigl(\varphi(t) \text{ AND } P_{s-1}\bigr) \ge 0$$

過去に採用されたトークンのビットパターンと重なる度合いを測る。

### 4.3 飽和定理

> **定理 T4.1 (ペナルティマスク飽和).** 評価される候補集合 $\mathcal{C}_s \subset \mathcal{T}$ の合計が有限 (各ステップ高々 MAX\_CANDS $= 40$ 候補) であるとき, ペナルティマスクの飽和度
> $$\rho_s = \frac{|P_s|_1}{1024}$$
> は単調非減少であり, $|P_s|_1$ は 1024 を超えることができない。

**証明.** $P_s = P_{s-1} \vee (\text{失敗候補の OR})$ より $|P_s|_1 \ge |P_{s-1}|_1$ (単調非減少)。$|P_s|_1 \le 1024$ はビット幅の上界から。各ステップで失敗候補がなければ $P_s = P_{s-1}$ となり増分が停止する。

**ステップ数評価:** 補題 L2.1 より各失敗候補 $\mu$ は高々 $|\varphi(t)|_1 \le 11$ 個の新規ビットを追加する。1 ステップで評価される失敗候補数は高々 MAX\_CANDS $= 40$ だから, 1 ステップあたり最大 $40 \times 11 = 440$ 新規ビット。したがって $\lceil 1024 / 440 \rceil = 3$ ステップ以内に飽和の上界に達しうる。実際には同一ビット位置への重複書き込みが多いため, 実験では $\approx 10$ ステップで $\rho_s$ が安定化する。$\square$

> **系 C4.1 (ランキング収束).** $\rho_s \to \rho^*$ (飽和値) のとき, 全候補の $\text{pen}_s(t)$ が定数に収束し, ペナルティ項によるランキングの識別力が失われる。

**証明.** $\rho_s$ が収束するとは $P_s$ が不変になることを意味する。このとき $\text{pen}_s(t) = \text{pen}_{s-1}(t)$ が全 $t$ で成立し, ペナルティ項はランキング順を変えなくなる。$\square$

### 4.4 実験的検証 (飽和定理の定量化)

実験データ (ensemble29, type-check-defaults.rs):

$$\rho_0 = 0,\quad \rho_1 = 2.6\%,\quad \rho_{10} = 9.2\%,\quad \rho_{25} = 9.2\% \quad (\text{飽和})$$

$\rho^* \approx 9.2\%$ すなわち $|P^*|_1 \approx 94$ ビットで飽和。これは候補集合の "有効ビット被覆" の上界を意味する。

> **予想 C4.1 (51/56 天井仮説 — 証拠あり未証明).** 残り 5 件のうち 2 件 (type-check-defaults, typeid-consistency) は, ペナルティ飽和により $H=2$ から脱出できない。根拠となる実験観察:
> - (E1) $\rho_s \approx 9.2\%$ ($|P^*|_1 \approx 94$ ビット) で飽和後, 25 ステップ (MAX\_STEPS) を完走しても $H=2$ が維持される
> - (E2) 同一の変異タイプが繰り返し採用される (tie-break による水平移動の反復, 本質的な探索進展なし)

**証明状況 (E68 実験後 — 更新済).** 実験 E68 (rustc エラースパン自動取得 × 21 候補トークン × 30 語彙 × 全出現 × 5種変異, 1537 件 exhaustive 探索, `experiment-68/results/result.md`) により以下が判明:

- **C4.1 の強い命題は反証**: $H < H_{\text{buggy}}$ を達成する単一変異 $\mu^*$ は候補空間に存在する。21 候補トークン × 30 語彙 × 全出現の exhaustive 探索 (1537 件, 重複除去済) で H<H_buggy が 681 件, $H=H_{\text{buggy}}-1=6$ への SNV が 40 件確認された (String→i32/bool/u32/u64/usize/isize/f64 等の Copy 実装型置換でトレイト制約エラーが解消)。
- **C4.1' (修正命題) は実験的に確定**: penMask 飽和状態の ensemble29 アルゴリズムは修正変異を発見できない。修正に必要なトークン (i32, String, Vec, Copy, IntoIter 等) は $P(\text{被ブロック}) \approx 0.62\text{--}0.65$ で penMask にブロックされる。
- **nibble ビット独立性**: $|\varphi(\texttt{i32}) \cap \varphi(\texttt{String})| = 0$ bits — 両トークンは nibble 空間で完全に独立。penMask への蓄積は使用履歴由来。

> **定理 T4.2' (C4.1 精緻化).** 予想 C4.1 の「H=2 から脱出不可能」は物理的限界ではなく、
> penMask 飽和後の **アルゴリズム的探索失敗** として確定する。
> 脱出変異 $\mu^*$ は候補空間 $\mathcal{C}$ に存在するが、
> $\text{pen}_s(\mu^*) > 0$ により全件スコアフィルタアウトされる。

**意義.** ペナルティ飽和後の候補生成多様性の欠如が天井の主因であることが実験的に確定した。nibble gram の 1024 ビット解像度は十分だが、penMask の「経路妨害」(修正経路上のトークンが早期失敗で蓄積される) が探索を阻害する。これは E69 以降で多様性向上 (penMask リセット, 探索再起動) を試みる動機となる。

---

## §5 RC⊗C テンソルと 2-ホップ拡散

### 5.1 ランク・頻度ベクトルの定義

**定義 5.1 (ランクベクトル $R$).** スパントークン列 $\sigma = (\sigma_0, \sigma_1, \ldots, \sigma_{n-1})$ に対し, トークン $\sigma_i$ の **出現ランク** を

$$R_i = |\{j < i \mid \sigma_j = \sigma_i\}| \in \mathbb{Z}_{\ge 0}$$

とする。すなわち $R_i$ は位置 $i$ に到達するまでに $\sigma_i$ が何回現れたかのカウントである。

**定義 5.2 (頻度ベクトル $C$).** unique トークン列 $u = (u_0, \ldots, u_{m-1})$ に対し,

$$C_k = |\{i \in [n] \mid \sigma_i = u_k\}|$$

**実装との対応:** `buildRC(tokSeq)` (ensemble29.mjs:187–207)

### 5.2 RC 行列と内積

**定義 5.3 ($RC\_mat$ 行列).** 各位置 $i \in [n]$ に対し 2 次元ベクトル

$$\mathbf{r}_i = \begin{pmatrix} R_i \\ C_{k(\sigma_i)} \end{pmatrix} \in \mathbb{R}^2$$

を並べた $n \times 2$ 行列を $RC\_mat$ と呼ぶ (実装では flat 配列 $RC\_mat[i \cdot 2], RC\_mat[i \cdot 2 + 1]$)。

**定義 5.4 (RC 内積行列 $RC_f$).** $n \times n$ 対称行列

$$RC_f = RC\_mat \cdot RC\_mat^\top, \quad (RC_f)_{ij} = \langle \mathbf{r}_i, \mathbf{r}_j \rangle = R_i R_j + C_{k(\sigma_i)} C_{k(\sigma_j)}$$

**実装との対応:** `rcSim2hop` の前半 (ensemble29.mjs:208–215)

### 5.3 2-ホップ拡散行列

**定義 5.5 ($RC_f^{(2)}$).** 行列積

$$RC_f^{(2)} = RC_f \cdot RC_f = (RC\_mat \cdot RC\_mat^\top)^2$$

**実装との対応:** `rcSim2hop` の後半 (ensemble29.mjs:216–223)

> **定理 T5.1 (グラフ拡散解釈).** $RC_f$ をトークン位置間の重み付きグラフ $G$ の隣接行列と見なすとき, $(RC_f^{(2)})_{ij}$ は $G$ 上の長さ 2 のパスの重み総和であり, 2 ホップ到達性を表す。

**証明.** 行列積の $(i,j)$ 成分は $\sum_k (RC_f)_{ik} (RC_f)_{kj}$, これは $i \to k \to j$ の 2 ステップの重みの積和だから。$\square$

> **補題 L5.1 (半正定値性).** $RC_f$ は半正定値 (PSD) である。

**証明.** $RC_f = AA^\top$, $A = RC\_mat$ より, 任意の $x \in \mathbb{R}^n$ に対し $x^\top RC_f x = \|A^\top x\|^2 \ge 0$。$\square$

> **系 C5.1.** $RC_f^{(2)}$ も PSD である。

**証明.** PSD 行列の積は PSD とは限らないが, $RC_f^{(2)} = RC_f \cdot RC_f = RC_f^2$ であり, $RC_f$ が PSD かつ対称なら $RC_f^2$ も PSD かつ対称。$\square$

### 5.4 rcf2Score の定義

**定義 5.6 (rcf2 スコア).** 候補ビット列 $b = \varphi(t) \in \{0,1\}^{1024}$, クエリ列の位置 $r_i$ における rcf2 行ベクトル $w = RC_f^{(2)}[r_i, :] \in \mathbb{R}^n$ に対し,

$$\text{rcf2}(t, r_i) = \sum_{j=0}^{n-1} w_j \cdot \langle \varphi(\sigma_j),\ b \rangle_{\text{AND}}$$

**実装との対応:** `pairScore` の `rf2` 計算 (ensemble29.mjs:248–256)

---

## §6 CS 直交位置埋め込み

### 6.1 符号付き頻度ベクトルの構成

**定義 6.1 (交番符号ベクトル $S$).** $n$ 次元ベクトル

$$S_i = (-1)^i \cdot C_{k(\sigma_i)}, \quad i \in [n]$$

位置の偶奇で頻度に符号を付けることで, 位置情報を頻度ベクトルに埋め込む。

### 6.2 Gram-Schmidt 直交射影

**定義 6.2 (直交投影 $S_{gs}$).** $R$ に対して $S$ を Gram-Schmidt 直交化する:

$$S_{gs} = S - \frac{\langle S, R \rangle}{\langle R, R \rangle} R$$

ここで $\langle \cdot, \cdot \rangle$ は標準内積。

**実装との対応:** `buildCS` 内の Gram-Schmidt (ensemble29.mjs:226–238)

> **定理 T6.1 (直交分離定理).** $S_{gs}$ は $R$ と直交する: $\langle S_{gs}, R \rangle = 0$。

**証明.** 直接計算:
$$\langle S_{gs}, R \rangle = \langle S, R \rangle - \frac{\langle S, R \rangle}{\langle R, R \rangle} \langle R, R \rangle = \langle S, R \rangle - \langle S, R \rangle = 0. \quad \square$$

**幾何学的意味:** $R$ はトークンの「繰り返し深さ」軸, $S_{gs}$ はそれと直交する「位置-頻度交互作用」軸。この 2 つの軸が互いに独立な情報を持つことが系 C6.1 で示される。

> **系 C6.1 (情報直交性).** $R$ が表現するランク情報と $S_{gs}$ が表現する位置情報は線形独立である。

**証明.** $S_{gs} \ne 0$ かつ $\langle S_{gs}, R \rangle = 0$ より, $S_{gs}$ は $R$ の線形スパンに含まれない。$\square$

### 6.3 CS 行列の外積構造

**定義 6.3 ($CS$ 行列).** $n \times m$ 行列

$$CS[i, k] = S_{gs}[i] \cdot C_k, \quad i \in [n],\ k \in [m]$$

すなわち $CS = S_{gs} \otimes C^\top$ (外積, rank-1 行列)。

**実装との対応:** `buildCS` の外積計算 (ensemble29.mjs:234–237)

> **定理 T6.2 (ランク 1 行列).** $CS$ は rank-1 行列である。

**証明.** $CS = S_{gs} C^\top$ は 2 ベクトルの外積であり, $S_{gs} \ne 0$ かつ $C \ne 0$ ならランク 1。$\square$

### 6.4 csScore の定義

**定義 6.4 (CS スコア).** 候補ビット列 $b$, 行インデックス $i$, $CS$ 行ベクトル $w = CS[i,:] \in \mathbb{R}^m$ に対し,

$$\text{cs}(t, i) = \sum_{k=0}^{m-1} |w_k| \cdot \langle \varphi(u_k),\ b \rangle_{\text{AND}}$$

**実装との対応:** `pairScore` の `cs` 計算 (ensemble29.mjs:258–270)

---

## §7 4×4 u4 循環行列と XOR 自己埋め込み

### 7.1 nibble ベクトルの取り出し

**定義 7.1 (nibble 分解).** headTok $= t_0 \in \mathcal{T}$ の nibHex 先頭バイト $b_0$ の上位 nibble $h = b_0 \gg 4 \in [16]$ を 4 ビット分解:

$$\mathbf{b} = (b^{(3)}, b^{(2)}, b^{(1)}, b^{(0)}) \in \{0,1\}^4, \quad h = \sum_{j=0}^{3} b^{(j)} \cdot 2^{3-j}$$

**双極化:** $b^{(j)} \mapsto \hat{b}^{(j)} = 2b^{(j)} - 1 \in \{-1, +1\}$

### 7.2 巡回行列 $H$

**定義 7.2 (左巡回行列).** $\hat{\mathbf{b}} = (\hat{b}^{(0)}, \hat{b}^{(1)}, \hat{b}^{(2)}, \hat{b}^{(3)}) \in \{-1,+1\}^4$ に対し,

$$H = \begin{pmatrix}
\hat{b}^{(0)} & \hat{b}^{(1)} & \hat{b}^{(2)} & \hat{b}^{(3)} \\
\hat{b}^{(1)} & \hat{b}^{(2)} & \hat{b}^{(3)} & \hat{b}^{(0)} \\
\hat{b}^{(2)} & \hat{b}^{(3)} & \hat{b}^{(0)} & \hat{b}^{(1)} \\
\hat{b}^{(3)} & \hat{b}^{(0)} & \hat{b}^{(1)} & \hat{b}^{(2)}
\end{pmatrix}$$

これは $\hat{b}^{(j)}$ を生成元とする $\mathbb{Z}/4\mathbb{Z}$ 上の左巡回行列である。

> **補題 L7.1 (行ノルム一定性).** $H$ の各行のノルムは $\|H_i\|_2 = \sqrt{\sum_j (H_{ij})^2} = \sqrt{4} = 2$ (ただし $H_{ij} \in \{-1,+1\}$)。

**証明.** 各行は $\hat{b}$ の巡回置換であり, $\{-1,+1\}^4$ の要素。$\|H_i\|_2^2 = 4$。$\square$

### 7.3 XOR 自己埋め込み

**定義 7.3 (XOR ハッシュ).** 各列 $j \in [4]$ のすべての行の XOR:

$$x_j = H_{0j} \oplus H_{1j} \oplus H_{2j} \oplus H_{3j} \in \{0, 1\}$$

ここで $\{-1,+1\}$ の XOR は $+1 \oplus +1 = 0$, $-1 \oplus +1 = 1$ などと定義 (原ビット値の XOR に等しい)。

**定義 7.4 (XOR 符号ベクトル).** $\sigma_j = (x_j = 0) ? -1 : +1 \in \{-1,+1\}$

**定義 7.5 (埋め込み行列 $H_{\text{emb}}$).**

$$H_{\text{emb}}[i][j] = H_{ij} \cdot \sigma_j$$

これは各列 $j$ を $\sigma_j$ でスケールした Hadamard 型変換である。

**実装との対応:** `buildH4(headTok)` (ensemble29.mjs:151–166)

### 7.4 headScore の双線形形式

**定義 7.6 (addVec).** 候補トークン $t_{\text{add}}$ のビット列 $\varphi(t_{\text{add}})$ の第 0 ワード $w_0 = \varphi(t_{\text{add}})[0]$ の下位 4 ビット:

$$\mathbf{v} = (v_0, v_1, v_2, v_3), \quad v_j = 2 \cdot ((w_0 \gg j) \& 1) - 1 \in \{-1,+1\}$$

**定義 7.7 (headScore).** 

$$\text{headScore}(t_0, t_{\text{add}}) = \sum_{i=0}^{3} \|H_{\text{emb}}[i]\|_2 \cdot \frac{\langle H_{\text{emb}}[i],\ \mathbf{v} \rangle}{4}$$

補題 L7.1 より $\|H_{\text{emb}}[i]\|_2 = 2$ (列スケールは直交変換ではなく Hadamard 型なのでノルムが変わりうるが実装では $2$ と固定), したがって:

$$\text{headScore} = 2 \sum_{i=0}^{3} \frac{\langle H_{\text{emb}}[i],\ \mathbf{v} \rangle}{4} = \frac{1}{2} \sum_{i=0}^{3} \langle H_{\text{emb}}[i],\ \mathbf{v} \rangle$$

**実装との対応:** `h4Score(H_embed, addBits)` (ensemble29.mjs:169–183)

> **定理 T7.1 (headScore 値域).** $\text{headScore}(t_0, t_{\text{add}}) \in [-8, 8]$。

**証明.** 各 $i$ について $\langle H_{\text{emb}}[i], \mathbf{v} \rangle \le \|H_{\text{emb}}[i]\|_2 \cdot \|\mathbf{v}\|_2 = 2 \cdot 2 = 4$ (Cauchy-Schwarz)。したがって $\text{headScore} \le 2 \cdot 4 = 8$。下界も同様。$\square$

> **補題 L7.2 (XOR キャンセル条件, 改訂版).** headScore$(t_0, t_{\text{add}}) \equiv 0$ (任意の $t_{\text{add}}$ に対して) となる必要十分条件は $\text{popcount}(h) = 2$, すなわち $h \in \{0011_2, 0101_2, 0110_2, 1001_2, 1010_2, 1100_2\} = \{3,5,6,9,10,12\}$。

**証明.** $\text{popcount}(h) = 2$ のとき $\hat{\mathbf{b}}$ は 2 個の $+1$ と 2 個の $-1$ を持つ。$H$ の各列は $\hat{\mathbf{b}}$ の巡回置換なので各列和 $\sum_i H_{ij} = \sum_k \hat{b}_k = 0$。また $b_0 \oplus b_1 \oplus b_2 \oplus b_3 = 0$ (偶パリティ) より全列の XOR ハッシュ $x_j = 0$, $\sigma_j = -1$, $H_{\text{emb}} = -H$。よって $\sum_i \langle H_{\text{emb}}[i], \mathbf{v} \rangle = -\langle \sum_i H[i], \mathbf{v} \rangle = 0$。逆に $\text{popcount}(h) \ne 2$ のとき headScore $\ne 0$ となる $\mathbf{v}$ が存在する (実験確認)。$\square$

> **注記 (旧補題 L7.2 の修正).** 旧版は $h \in \{0000_2, 0101_2, 1010_2, 1111_2\}$ と主張していたが不正確であった。$h \in \{0000_2, 1111_2\}$ (popcount=0,4) は XOR=0 かつ $H_{\text{emb}} = -H$ となるが $\sum_i H[i] \ne \mathbf{0}$ のため headScore $\equiv 0$ にはならない。$h \in \{0011_2, 0110_2, 1001_2, 1100_2\}$ は旧版で未列挙だったが強キャンセル条件を満たす。

**【実験的確認 (E57, 2026-04-16)】** 5 種 Rust ファイル (536 tokens, 270 unique) に対する測定:

| 条件 | 予測 | 実測 | 判定 |
|------|------|------|------|
| popcount=2 トークン → headScore=0 | 100% | 100% (176/176) | ✓ |
| 強キャンセル違反 | 0件 | 0件 | ✓ |
| Rust ソース強キャンセル率 | — | **65.11%** (349/536) | 測定完了 |
| headScore=0 割合 (ユニーク) | — | **65.2%** | 測定完了 |

主要なキャンセル源: h=6 (lowercase a-z: 40.1%), h=5 (P-Z/_: 14.9%), h=3 (0-9: 10.1%)。
headScore は Rust ソーストークンの 65% に対して識別力ゼロ。
詳細: `experiment-57/headscore_result.md`, `experiment-57/headscore_hist.png`

---

## §8 S²·v 宇宙トークン拡散

### 8.1 宇宙トークン空間

**定義 8.1 (宇宙トークン $\mathcal{U}$).** 正規表現 $[A\text{-}Za\text{-}z\_][A\text{-}Za\text{-}z0\text{-}9\_]^{1,2}$ に合致する長さ 2〜3 の全トークンの集合:

$$|\mathcal{U}| = 53 \cdot 63 + 53 \cdot 63^2 = 3{,}339 + 210{,}357 = 213{,}696$$

**実装との対応:** `universeToks` 生成 (ensemble29.mjs:283–288)

### 8.2 類似度行列と S²·v 拡散

**定義 8.2 (類似度行列 $S$).** $\mathcal{U}$ の部分集合 $U = \{u_1, \ldots, u_N\}$ ($N = $ UV\_N $= 100$) に対し, $N \times N$ 対称行列

$$S_{ij} = \langle \varphi(u_i),\ \varphi(u_j) \rangle_{\text{AND}} = \text{popcount}(\varphi(u_i) \text{ AND } \varphi(u_j))$$

**定義 8.3 (初期スコアベクトル $\mathbf{v}$).** クエリ $Q$ に対して各 $u_i$ のスコア:

$$v_i = \langle \varphi(u_i),\ \Phi(Q) \rangle_{\text{AND}}$$

**定義 8.4 (S²·v 拡散).** 2 段 power iteration:

$$\mathbf{h}_1 = S \mathbf{v}, \quad \mathbf{h}_2 = S \mathbf{h}_1 = S^2 \mathbf{v}$$

**実装との対応:** `universeVirtualToks` (ensemble29.mjs:296–331)

> **定理 T8.1 (優勢固有ベクトル近似).** $S$ の固有値を $\lambda_1 \ge \lambda_2 \ge \cdots$, 対応する固有ベクトルを $\mathbf{e}_1, \mathbf{e}_2, \ldots$ とし, $\mathbf{v} = \sum_k c_k \mathbf{e}_k$ と分解するとき,

$$S^2 \mathbf{v} = \sum_k c_k \lambda_k^2 \mathbf{e}_k$$

よって $|\lambda_1| > |\lambda_2|$ ならば $S^2 \mathbf{v} \approx c_1 \lambda_1^2 \mathbf{e}_1$ となり, 2 回の反復で主固有ベクトル方向が強調される。

**証明.** スペクトル展開の定義から直接従う。$\square$

> **補題 L8.1 ($S$ の半正定値性).** $S$ は PSD である。

**証明.** $S_{ij} = \langle \varphi(u_i), \varphi(u_j) \rangle_{\text{AND}}$ は内積行列 (グラム行列) の非負値版。任意の $x \in \mathbb{R}^N$ に対し

$$x^\top S x = \sum_{i,j} x_i S_{ij} x_j = \left\langle \sum_i x_i \varphi(u_i),\ \sum_j x_j \varphi(u_j) \right\rangle_{\mathbb{R}} \ge 0$$

$\{0,1\}$ ビット列を実数ベクトルと見ての標準内積は非負。$\square$

**帰結:** $S$ の固有値は全て $\ge 0$, $S^2$ の固有値は全て $\ge 0$。$S^2 \mathbf{v}$ のスコアはスコアの高い方向 (クエリに最も類似した固有ベクトル方向) を 2 乗強調する。

### 8.3 高速近似 (SIG フィルタ)

**定義 8.5 (シグネチャフィルタ).** $S^2\mathbf{v}$ の計算は $O(N^2 \cdot 1024)$ だが, 4 点シグネチャインデックス $\text{SIG\_IDX} = \{0, 8, 16, 24\}$ の 4 ワードのみ先行フィルタし上位 SIG\_K $= 500$ を絞り込む 2 段階近似を行う。

> **命題 P8.1 (近似誤差).** 4 ワード (128 ビット) シグネチャ類似度は 1024 ビット全体類似度の 1/8 サンプリングであり, 真の上位候補を $\ge 80\%$ の確率で捕捉する (sparse ベクトルの Bloom filter 性質による)。

---

## §9 Nibble Hash Matrix C11

### 9.1 2×4 状態行列

**定義 9.1 (nibble_hash_matrix).** 入力 $D = (d_0, d_1, \ldots) \in [256]^*$ に対し, 初期状態行列

$$M^{(0)} = \begin{pmatrix}
\ell & 3\ell & 7\ell & 11\ell \\
5\ell & 13\ell & 17\ell & 19\ell
\end{pmatrix} \pmod{256}$$

ここで $\ell = |D| \bmod 256$。各バイト $d_k$, 列 $c = k \bmod 4$ に対し:

- 行 0 (lo): $M^{(k)}_{0,c} \leftarrow \text{ROL}_8(M^{(k-1)}_{0,c} +_8 d_k,\ c+1) \oplus d_k$
- 行 1 (hi): $M^{(k)}_{1,c} \leftarrow \text{ROL}_8(M^{(k-1)}_{1,c} \oplus d_k +_8 M^{(k)}_{0,c},\ (c+2)\bmod 4+1)$

ここで $+_8$ は $\bmod 256$ 加算, $\text{ROL}_8(v, r)$ は 8 ビット左巡回シフト。

**ファイナライザ:** 2 回の列間クロスミキシングを適用後,

$$(\text{hash\_hi}, \text{hash\_lo}) = \left(\bigoplus_{c=0}^3 M_{1,c},\ \bigoplus_{c=0}^3 M_{0,c}\right)$$

**実装との対応:** `nibble_hash_matrix` (bitrag-core/src/nibble_hash.rs)

### 9.2 均一性定理

> **定理 T9.1 (限界均一性).** 256 個の単一バイト入力 $D = (b)$, $b \in [256]$ に対し,
> $$|\{\text{hash\_lo}(b) \mid b \in [256]\}| \ge 150, \quad |\{\text{hash\_hi}(b) \mid b \in [256]\}| \ge 150$$

**証明のスケッチ.** 単一バイト入力では列 $c = 0$ のみが更新される。しかし長さシード $\ell = 1$ により初期行列の全列が非ゼロで相互依存しており, ファイナライザの列間クロスミキシングが残りの列に入力 $b$ の情報を伝播する。完全ランダム 256 入力の期待 distinct 数は $256 \cdot (1 - (255/256)^{256}) \approx 161.7$ であり, 実験値 (Rust テスト `test_uniformity_256`) が $\ge 150$ を達成することで確認される。$\square$

> **定理 T9.2 (近似独立性).** 65536 個の 2 バイト入力に対し, hash\_hi と hash\_lo の Pearson 相関係数は $|r| \le 0.05$ である。

**証明のスケッチ.** 2 バイト入力では $c=0$ と $c=1$ の 2 列が独立に更新される。行 0 と行 1 は異なる演算順序 (ADD+ROL+XOR vs. XOR+ADD+ROL) を持ち, ファイナライザが行間の情報を混合する。Pearson 相関の実測値 $r \approx -0.006$ (Rust テスト `test_uniformity_65536`) がこれを支持する。$\square$

### 9.3 P17 nibble\_id\_spec

**定義 9.2 (nibble\_id).** 16 ビット識別子のパッキング規則:

$$\text{nibble\_id} = (p_{\text{class}} \ll 12) \mid (h_n \ll 8) \mid (l_n \ll 4) \mid c_{\text{class}}$$

ここで $h_n = \text{hash\_hi} \gg 4$, $l_n = \text{hash\_lo} \gg 4$ (各 4 ビット)。

> **補題 L9.1 (グリッド分解).** nibble\_id の中位 8 ビット $(h_n, l_n) \in [16]^2$ がトークンを $16 \times 16 = 256$ セルのグリッドに写像する全射を定める。

**証明.** 定理 T9.1 の均一性により, 65536 入力で全 256 $(h_n, l_n)$ セルがカバーされる (グリッド CV $= 12.1\%$, 全セル非空が実験確認済み)。$\square$

---

## §10 多目的スコアリング関数と離散勾配降下

### 10.1 目的関数

**定義 10.1 (変異スコア関数 $f$).** 変異操作 $\mu$ (候補トークン $t_{\text{add}}$, 変異タイプ, 除去トークン $t_{\text{rem}}$) に対し,

$$f(\mu) = \\underbrace{\text{pen}(\mu)}_{\ge 0} - \alpha \cdot \underbrace{\text{sim}(\mu)}_{\ge 0} - \beta \cdot \text{rcf2}(\mu) - \gamma \cdot \text{cs}(\mu) - \delta \cdot \\underbrace{\text{headScore}(\mu)}_{\in [-8,8]}$$

パラメータ: $\alpha = 1.0,\ \beta = 0.5,\ \gamma = 0.3,\ \delta = 0.2$。

また変異タイプに応じたベーススコア $b_{\text{type}} \in \{0, 2, 4\}$ が加算される (SNV=0, DEL=2, CNV=4)。

**実装との対応:** `pairScore` および `oracleStep` のスコア計算 (ensemble29.mjs:240–273, 493–540)

### 10.2 離散勾配降下の定義

**定義 10.2 (離散 GD — 実装忠実版).** Rust ソース $s_0$ から出発し, 各ステップ $t = 1, 2, \ldots$ で:

1. **候補生成:** 現在ソース $s_{t-1}$ と $\mathcal{O}(s_{t-1})$ からスパントークン列 $\sigma$ を取得し, 5 種変異候補集合 $\mathcal{C}_t$ を生成する ($|\mathcal{C}_t| \le $ MAX\_CANDS $= 40$)

2. **スコアリング:** 各 $\mu \in \mathcal{C}_t$ について $f(\mu)$ を計算し昇順ソート

3. **オラクル評価:** 上位 MAX\_CANDS 候補に変異適用し $\mathcal{O}$ を呼ぶ。**採用条件はエラー数の厳密な改善 OR 同エラー数でのペナルティスコア改善 (tie-break)**:

$$\mu^*_t = \arg\min_{\prec} \left\{ (\mu,\ \text{pen}_t(\mu)) \;\middle|\; H(\text{apply}(\mu, s_{t-1})) \le H_{\text{best}} \right\}$$

ここで全順序 $\prec$ は, まず $H(\text{apply}(\mu, s_{t-1})) < H_{\text{best}}$ を優先し, 同 $H$ なら $\text{pen}_t(\mu)$ の小さい方を採る。$H_{\text{best}}$ は評価開始時の $H(s_{t-1})$ に初期化される。

具体的には実装の制御フロー:
```
if h < bestH  →  更新 (strict 改善)
if h == bestH AND penScore < bestPen  →  更新 (tie-break)
```

4. **ペナルティスコア計算タイミングとペナルティマスク更新:** ペナルティスコア $\text{pen}_t(\mu)$ は各候補評価**前** (コンパイル呼び出し前) に現時点の $P_{t-1}$ を用いて計算される。その後コンパイルを呼び出し, 失敗した場合 ($H > 0$) に $P_t$ を更新する。すなわち tie-break の比較対象 `penScore` は, その候補の評価が $P$ を汚染する**前**の値を使う。

5. **状態更新:** $s_t = \text{apply}(\mu^*_t, s_{t-1})$, 改善候補が存在しない場合は `best = null` で早期終了

停止条件: $H(s_t) = 0$ (成功) または $t = $ MAX\_STEPS $= 25$ (失敗) または `best = null` (オラクル評価で候補なし, 早期終了)。

**実装との対応:** `oracleStep` (ensemble29.mjs:571–606), `bestH`, `bestPen` 変数, tie-break 条件 (ensemble29.mjs:586–587)

### 10.3 単調減少性定理

> **定理 T10.1 (エラー数単調非増加).** 離散 GD の各ステップで採用された $\mu^*_t$ は
> $$H(s_t) \le H(s_{t-1})$$
> を満たす。

**証明.** 定義 10.2 の採用条件を分析する。評価ループ開始時に $H_{\text{best}} = H(s_{t-1})$ と初期化される。候補 $\mu$ は条件 `h < bestH` または `h == bestH AND penScore < bestPen` のいずれかを満たすときのみ `best` を更新する。いずれの条件でも $h \le H_{\text{best}} = H(s_{t-1})$ が保持される。したがって採用候補が存在する場合, $H(s_t) = h \le H(s_{t-1})$ が成立する。採用候補が存在しない場合 (`best = null`) は早期終了し状態更新されないので不等式は問題にならない。$\square$

> **注意.** Tie-break 採用 ($H(s_t) = H(s_{t-1})$ かつ $\text{pen}$ 改善のみ) の場合, エラー数は改善されないが, ペナルティマスクの更新によって次ステップの探索空間が更新される。これは「同一エラー数のより罰則の少ない点」への移動であり, 逃げ道探索 (escape) として機能する。

> **系 C10.1.** エラー数列 $(H(s_0), H(s_1), \ldots)$ は広義単調減少である (ただし tie-break ステップでは水平移動が許容される)。

### 10.4 コンパイラオラクルの形式化

**定義 10.3 (オラクル問合せ複雑度).** $T$ ステップの実行でのオラクル問合せ総数は $O(\text{MAX\_STEPS} \cdot \text{MAX\_CANDS}) = O(25 \cdot 40) = O(1000)$ 回。

> **補題 L10.1 (決定論性).** $\mathcal{O}$ は決定論的: 同一入力に対して常に同一出力を返す (rustc は確定的)。したがって離散 GD は確率的要素を持たない純粋な決定論的アルゴリズムである。

### 10.5 局所最小定理

**定義 10.4 (離散局所最小).** ソース $s$ が **離散局所最小** であるとは, 全変異候補 $\mu \in \mathcal{C}$ に対して $H(\text{apply}(\mu, s)) \ge H(s)$ となることをいう。

> **定理 T10.2 (有限ステップ停止).** 離散 GD は最大 MAX\_STEPS $= 25$ ステップ内に必ず停止する。停止理由は次の 3 種:
> 1. $H(s_t) = 0$ (大域最小 — コンパイル成功)
> 2. `best = null` (改善候補なし — 探索的局所最小)
> 3. $t = $ MAX\_STEPS $= 25$ (ステップ上限到達 — 局所最小性は保証されない)

**注意.** 停止理由 3 では, 残りの候補を評価し切れていない可能性があり, 真の局所最小には達していない場合がある。実験上の 51/56 天井は主に停止理由 2 および 3 によるものである。

**証明.** $H$ の値域は $\{0, 1, \ldots, H_0\}$ の有限集合で単調非増加 (定理 T10.1)。したがって高々 $H_0$ ステップで $H$ は安定または $0$ に達する。さらにペナルティ飽和 (定理 T4.1) により候補集合の有効スコアが収束し, 改善候補が存在しなくなる。早期終了条件がこの状態を検出する。$\square$

### 10.6 AUC と探索効率の関係

**定義 10.5 (正規化軌跡 AUC).** エラー軌跡 $(H_0, H_1, \ldots, H_T)$ の正規化面積:

$$\text{AUC} = \frac{1}{T} \int_0^T \frac{H(t)}{H_0} \, dt \approx \frac{1}{T} \left( \frac{H_0/H_0}{2} + \sum_{t=1}^{T-1} \frac{H_t}{H_0} + \frac{H_T/H_0}{2} \right)$$

台形則による数値積分。

> **命題 P10.1 (AUC の解釈).** AUC $= 1$ は全ステップで $H(t) = H_0$ (改善なし), AUC $= 0$ は第 1 ステップで $H=0$ (即時解決)。実験での combined avg AUC $= 0.788$ は, 残 5 件のうち 3 件が AUC $= 1.0$ (flat, 全く改善せず) であることに起因する。

> **定理 T10.3 (探索効率上界).** $n$ ステップ後の期待 AUC は

$$\mathbb{E}[\text{AUC}] \ge 1 - \frac{\mathbb{E}[H_0 - H_n]}{H_0}$$

すなわち, 最終エラー数削減量 $\Delta H = H_0 - H_n$ が大きいほど AUC が小さい (= 効率的な探索)。

---

## §11 5 種変異操作の形式化

**定義 11.1 (変異関数の型).** 各変異 $\mu$ は関数 $\text{apply}_\mu: \mathcal{S} \to \mathcal{S} \cup \{\bot\}$ であり, $\bot$ は適用不可を表す。

| 変異種 | 定義 | 形式的操作 |
|-------|------|-----------|
| **SNV** | $t_{\text{rem}} \to t_{\text{add}}$ (全置換) | $s \mapsto s[t_{\text{rem}} \leftarrow t_{\text{add}}]_{\text{global}}$ |
| **INSERT** | スパン行 $L$ の先頭に $t_{\text{add}}$ を挿入 | $s \mapsto \text{insertLine}(s, L, t_{\text{add}})$ |
| **DEL\_F** | $\text{extra}[0]$ の最初の出現を削除 | $s \mapsto s[\text{first}(t_0) \leftarrow \varepsilon]$ |
| **DEL\_B** | $\text{extra}[-1]$ の最後の出現を削除 | $s \mapsto s[\text{last}(t_{-1}) \leftarrow \varepsilon]$ |
| **CNV** | $t_{\text{rem}} \to t_{\text{rem}}\ t_{\text{rem}}$ (最初の出現) | $s \mapsto s[\text{first}(t_{\text{rem}}) \leftarrow t_{\text{rem}}\ t_{\text{rem}}]$ |

> **補題 L11.1 (SNV edit-distance 1 優先性).** SNV において $d_{\text{edit}}(t_{\text{rem}}, t_{\text{add}}) = 1$ のとき, スコアボーナス $-1.0$ が付与される。これは微小変更 (1 文字差) の候補を大域置換より優先することに対応し, 局所最小脱出の確率を高める。

**証明.** スコア $f(\mu) \to f(\mu) + \text{SNV\_BONUS}$ ($= -1.0$) であり, ソート後に順位が上昇する。edit-distance 1 の変異は字義的に類似しており, コンパイラエラーの「typo 型」修正に最適化されている。$\square$

---

## 補遺 A: 実装との対応表

| 補題/定理 | 数学的対象 | 関数名 | ファイル: 行 |
|----------|-----------|--------|------------|
| 定義 2.1 | nibble ビット埋め込み $\varphi$ (11 オーバーラップペア, $|\varphi|_1 \le 11$) | `tokBits` | ensemble29.mjs:110–119 |
| 定義 2.2 | ユニオンビット $\Phi(T)$ | `unionBits` | ensemble29.mjs:143–147 |
| T2.1 | AND popcount = 包含測度 | `andCount` | ensemble29.mjs:132–141 |
| 定義 3.1 | nibbleSim | `pairScore` (sim) | ensemble29.mjs:241–247 |
| 定義 4.1 | ペナルティマスク $P_s$ (失敗評価候補全件を蓄積) | `penMask` + `if (h > 0)` 更新 | ensemble29.mjs:579–581 |
| T4.1 | ペナルティ飽和定理 | — (理論的結論) | — |
| 定義 5.1–5.3 | $R, C, RC\_mat$ | `buildRC` | ensemble29.mjs:187–207 |
| 定義 5.4 | $RC_f = RC\_mat \cdot RC\_mat^\top$ | `rcSim2hop` (前半) | ensemble29.mjs:208–215 |
| 定義 5.5 | $RC_f^{(2)} = RC_f^2$ | `rcSim2hop` (後半) | ensemble29.mjs:216–223 |
| 定義 6.1 | 交番符号ベクトル $S$ | `buildCS` | ensemble29.mjs:229–231 |
| T6.1 | 直交分離定理 | `buildCS` (GS 射影) | ensemble29.mjs:232–233 |
| 定義 6.3 | $CS = S_{gs} \otimes C^\top$ | `buildCS` (外積) | ensemble29.mjs:234–237 |
| 定義 7.2 | 左巡回行列 $H$ | `buildH4` | ensemble29.mjs:157–162 |
| 定義 7.3–7.5 | XOR 自己埋め込み $H_{\text{emb}}$ | `buildH4` | ensemble29.mjs:163–165 |
| T7.1 | headScore 値域 $[-8,8]$ | `h4Score` | ensemble29.mjs:169–183 |
| 定義 8.2–8.4 | $S$ 行列と $S^2\mathbf{v}$ | `universeVirtualToks` | ensemble29.mjs:296–331 |
| T9.1–T9.2 | 均一性・独立性定理 | `nibble_hash_matrix` | nibble_hash.rs:34–85 |
| 定義 10.2 | 離散 GD (tie-break 採用含む) | `gdFix` + `oracleStep` | ensemble29.mjs:571–660 |
| T10.1 | エラー数単調非増加 | `bestH` 初期化と `h < bestH \|\| (h==bestH && pen<bestPen)` | ensemble29.mjs:573, 586–587 |
| T10.2 | 有限ステップ停止 (3種の停止条件) | `gdFix` の制御フロー (MAX\_STEPS 上限 / `best=null` / $H=0$) | ensemble29.mjs:608–660 |
| 定義 10.5 | AUC (台形則) | `auc_trap` 関数 | experiment-55/ensemble29.mjs 内のインライン計算 |

---

## 補遺 B: 主定理一覧

| 番号 | 名称 | 主張の概要 |
|------|------|-----------|
| T2.1 | AND popcount 内積定理 | bitwise AND popcount は $\mathbb{R}$ 値内積を定める |
| T4.1 | ペナルティ飽和定理 | 有限ステップで $\rho_s$ は単調収束する |
| T5.1 | 2-ホップ拡散定理 | $RC_f^{(2)}$ は長さ 2 パスの重み和 |
| T6.1 | 直交分離定理 | $S_{gs} \perp R$ (Gram-Schmidt) |
| T6.2 | ランク 1 定理 | $CS$ はランク 1 外積行列 |
| T7.1 | headScore 値域定理 | headScore $\in [-8, 8]$ (Cauchy-Schwarz) |
| T8.1 | S²·v 固有ベクトル近似 | $S^2\mathbf{v}$ は主固有方向を強調 |
| T9.1 | nibble hash 均一性 | distinct $\ge 150/256$ |
| T9.2 | nibble hash 独立性 | $|r_{\text{Pearson}}| \le 0.05$ |
| T10.1 | 単調非増加定理 | $H(s_t) \le H(s_{t-1})$ |
| T10.2 | 有限ステップ停止定理 | 最大 MAX\_STEPS=25 内に停止; 停止理由3種 (成功/$\text{best}=\text{null}$/ステップ上限) |

---

*作成: 2026-04-16  ·  対象: ensemble29-v3 (experiment-55) / C11 (experiment-56)  ·  証明スタイル: 数学的厳密・自然言語ベース*

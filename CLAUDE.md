# 開発方針＆開発環境ルール(RS-CSS)

作業ドライブは`F:\runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。

## リポジトリ改称(2026-09-13)

`RCSS`→`RS-CSS`へGitHub上でrename済み。`aruaru.pro`向けのフロント基盤
整備に合わせた`RFrontEnd`傘下のネーミング統一の一環(`RS-HTML`・
`RS-GraphQL`・`RS-Node.js`も同時に改称)。crate名(`rcss3`)は依存関係の
広さ(`RS-BootStrap`・`RS-React`から利用)に対して名前変更の効果が薄い
ため今回は見送り、path参照のみ更新した。以下の記述内の`RCSS`表記は
改称前の履歴として残す。

## このプロジェクトの構想(2026-07-18新設)

`RHTML5`/`RCSS3`/`RTypeScript`/`RBootStrap`という4プロジェクト構想の1つ。
詳細な全体構想・開発順序・マイルストーンは[`rhtml5`](https://github.com/aon-co-jp/rhtml5)
のCLAUDE.mdを参照(構想はrhtml5側に集約して記録)。

**設計判断**: `rcss3`は特定のDOM実装(`rhtml5::Element`等)に直接
依存しない。要素とのマッチングは`ElementLike`トレイト(`tag_name()`/
`classes()`/`id()`)経由で行う——`cssparser`(Servo由来)がDOM非依存で
あるのと同じ設計判断。これにより将来的に別のDOM実装と組み合わせる
自由度を保つ。

## 現状(第一段、2026-07-18)

- `src/selector.rs`: `SimplePart`(`Tag`/`Class`/`Id`/`Universal`)、
  `CompoundSelector`(単純セレクタの組み合わせ、結合子は未対応)、
  `ElementLike`トレイト、`matches()`、`specificity()`
  (標準的な(id数,class数,tag数)モデル)。
- `src/parser.rs`: コメント除去→`セレクタ { 宣言 }`ブロック分割→
  宣言(`property: value;`)パースの逐次スキャナ。カンマ区切りの
  複数セレクタ対応。
- `src/cascade.rs`: マッチする全ルールを(specificity, ソース順)で
  昇順ソートし、宣言を順に適用(後勝ち)する`compute_style()`
  (2026-07-18: 第3引数に祖先チェーン`ancestors: &[&E]`を追加、
  子孫結合子のマッチングに使う)。`style_to_string()`でHTML `style`
  属性用の文字列に変換。
- `src/selector.rs`(2026-07-18更新): **子孫結合子(スペース区切り、
  例: `div p`)対応**。`Selector = Vec<CompoundSelector>`型、
  `parse_selector()`・`selector_specificity()`(各コンパウンドの
  specificityの合計)・`matches_selector()`(祖先チェーンを右から左へ
  消費して判定)を追加。
- **2026-07-18追記: 子結合子(`>`)・隣接兄弟結合子(`+`)対応完了**。
  `Selector`型を`Vec<CompoundSelector>`から`Vec<SelectorSegment>`
  (各segmentが「1つ左隣とどう関係するか」を表す`Combinator`
  (`Descendant`/`Child`/`AdjacentSibling`)を保持)へ変更(破壊的変更、
  依存クレート`RReact`/`RBootStrap`も追従修正済み)。`parse_selector`は
  `>`/`+`の前後にスペースが無い表記(`div>p`)にも対応。
  `matches_selector`/`compute_style`に`preceding_siblings`引数を追加
  (`+`のマッチングに使う、直前の兄弟から順に並べた配列)。
  **正直なスコープの限界**: `+`はセレクタの最も右側の結合でのみ判定
  可能(例: `li + li`)。`div + p span`のように`+`がそれより左側
  (祖先チェーン側)に現れる場合、祖先の兄弟情報がそもそも呼び出し側
  から渡されないため判定不能——安全側に倒して常に不一致を返す
  (`selector.rs`冒頭コメント参照)。
- **2026-07-19追記: 一般兄弟結合子(`~`)対応完了**。`Combinator`に
  `GeneralSibling`を追加、`parse_selector`は`~`の前後にスペースが
  無い表記(`.a~.b`)にも対応。マッチングは`+`と違い
  `preceding_siblings`を1件目だけでなく全件スキャンし、いずれか一致
  すれば真(直前である必要はない——CSS仕様通り)。
  **正直なスコープの限界(`+`と共通)**: `matches_selector`が受け取る
  `preceding_siblings`はtarget要素自身のものだけなので、`~`もセレクタ
  最右の結合でのみ判定可能。`div ~ p span`のように左側(祖先チェーン側)
  に現れる場合は`+`と同じ理由で判定不能——安全側に倒して不一致を返す
  (`selector.rs`冒頭コメント参照)。依存クレート`RReact`/`RBootStrap`は
  `Combinator`を網羅的にmatchしていない(構築のみ)ため、変更不要
  ・両クレートとも無変更でビルド/テスト green を確認済み。
- **未対応(次段階)**: `@media`等のat-rule、`!important`、
  カスケードレイヤー(`@layer`)、CSS変数、レイアウト計算
  (flexbox/grid)、深い位置での`+`/`~`結合子(上記参照)。
- **検証**: `cargo test`で32件全green(既存27件+一般兄弟結合子の
  パース2件・マッチング3件)。警告0件。依存先の`RBootStrap`
  (22件、無変更)・`RReact`(`dom_bridge`フィーチャ有効時16件、
  無効時10件、無変更)も影響なし(`Combinator`を網羅的にmatchしておらず
  構築のみのため)を確認済み。

## 次にすべきこと

1. 「RHTML5+RCSS3を使った最小のPoem SSRエンドポイント」マイルストーン
   (`rhtml5`側CLAUDE.md参照)
2. `@media`等のat-rule、`!important`
3. 深い位置での`+`/`~`結合子対応(祖先の兄弟情報を渡せるAPI設計の検討)

## 関連プロジェクト

- [rhtml5](https://github.com/aon-co-jp/rhtml5) — 対になるDOM実装、
  全体構想の詳細はこちらに集約
- [rreact](https://github.com/aon-co-jp/RReact) — `dom_bridge`
  フィーチャで`rhtml5::Element`が`rcss3::ElementLike`を実装するアダプタ
  (`ElementRef`)を提供し、RHTML→RCSS→RReactのEnd-to-Endパイプラインを
  実装済み(2026-07-18、詳細はRReact側CLAUDE.md参照)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本

## HANDOFF

- **2026-07-19(続き) `!important`/`@media`をcompute_styleへ実配線
  (未完成の発見と修正)**: `audiocafe-tokyo-rust`ユーザーからの
  エコシステム完成度向上要望を受けて監査したところ、2つの問題が
  見つかった。(1) 前回セッションが`!important`/`@media`のパーサー・
  データモデル(`src/media.rs`新設・`parser.rs`更新・`lib.rs`の
  re-export)を完成させていたにもかかわらず、**一度もコミット・push
  されていなかった**(このファイルの「未対応」記載もそのため古いままに
  なっていた)。(2) さらに、そのデータモデル自体は完成していても、
  実際にスタイルを計算する`cascade::compute_style`が
  `Declaration.important`/`Rule.media`を一切参照しておらず、
  パースはされるが計算結果に何の影響も与えない「配線されていない」
  状態だった。両方を修正: `compute_style`に2パス適用
  (非important→important)による優先順位と、`MediaContext::default()`
  (screen・1920px)に対する`@media`フィルタリングを実装。新規テスト
  6件追加、既存分と合わせ42件全green・警告0件。依存クレート
  `RBootStrap`は`Declaration`/`Rule`の新フィールドに追従しておらず
  ビルドが壊れていたため、`important: false`/`media: None`を追加して
  修正した(RBootStrap側CLAUDE.md参照)。`RReact`の`compute_style`
  呼び出し(`dom_bridge.rs`)はシグネチャ変更が無いため無修正で
  ビルド・テストとも影響なしを確認済み。
  次にすべきこと: 深い位置での`+`/`~`結合子対応、カスケードレイヤー
  (`@layer`)、CSS変数、レイアウト計算(flexbox/grid)。

- **2026-07-19 一般兄弟結合子(`~`)対応**: `Combinator::GeneralSibling`
  追加、`parse_selector`が`~`を認識、`matches_selector`は
  `preceding_siblings`を全件スキャンして「直前でなくてもよい」
  マッチングを実装(`+`との違い)。深い位置での`~`は`+`と同じ理由で
  スコープ外(安全側に倒して不一致)。テスト27件→32件(全green、
  警告0件)。`RBootStrap`/`RReact`は`Combinator`を網羅的にmatchして
  いないため無変更・影響なし(両クレートとも既存テスト数のまま
  green確認済み)。次にすべきこと: `@media`/`!important`対応、
  深い位置での`+`/`~`結合子対応(API設計の見直しが必要)。

- **2026-07-18 子孫結合子(スペース区切りセレクタ)対応**: `Selector`
  型(`Vec<CompoundSelector>`)・`parse_selector`・`selector_specificity`・
  `matches_selector`(祖先チェーンを右から左へ消費するアルゴリズム)を
  追加。`Rule.selectors`の型を`Vec<CompoundSelector>`から
  `Vec<Selector>`へ変更(破壊的変更、`parser.rs`/`cascade.rs`両方を
  追従修正)。`compute_style`に祖先チェーン引数を追加(既存呼び出しは
  `&[]`で後方互換)。テストは13件→20件(全green、警告0件)。
  併せて、次のステップとして明記されていた「`rhtml5::Element`への
  `ElementLike`アダプタ」は、orphan ruleの制約により本クレートではなく
  `rreact`クレート側の`dom_bridge`フィーチャで実装した(利用側クレートで
  実装する、という本ファイルの元々の方針通り)。
  次にすべきこと: 子結合子(`>`)・隣接兄弟結合子(`+`)対応、
  `@media`/`!important`対応。

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**

//! スタイルシート(`Vec<Rule>`)を実際のCSSテキストへシリアライズする。
//!
//! **経緯**: `RBootStrap`(現`RBootstrap`)側のCLAUDE.mdに、
//! 「`Rule.media`は正しく設定されるようになったが、それを『本物のCSS
//! ファイル』として出力する経路が無い」という設計上の限界が明記されて
//! いた——レスポンシブ`@media`が謳われていても、実際にブラウザへ渡せる
//! `.css`テキストを生成する手段がこのクレートに存在しなかった、という
//! 「配線されていない」ギャップ。本モジュールはその欠落を埋める。
//!
//! **スコープ(正直な開示)**: 入力はパーサーが生成した`Rule`列を想定し、
//! 出力は「セレクタ再構築→宣言→`@media`によるグルーピング」という
//! 素直な整形のみ。CSSの正規化(値の再パース・圧縮・重複排除)は行わない
//! ——`Declaration.value`はパース時の生文字列をそのまま出力する。
//! 同一の`MediaQuery`を持つ複数の`Rule`は出現順を保ったまま同じ
//! `@media`ブロックにまとめる(`PartialEq`で比較、入れ子の`@media`は
//! 生成しない——本クレートの`Rule.media`自体がフラットな設計のため)。

use crate::media::{MediaQuery, MediaType};
use crate::parser::{Declaration, Rule};
use crate::selector::{Combinator, Selector, SimplePart};

fn simple_part_to_string(part: &SimplePart) -> String {
    match part {
        SimplePart::Tag(name) => name.clone(),
        SimplePart::Class(name) => format!(".{name}"),
        SimplePart::Id(name) => format!("#{name}"),
        SimplePart::Universal => "*".to_string(),
    }
}

fn combinator_prefix(combinator: Combinator) -> &'static str {
    match combinator {
        Combinator::Descendant => " ",
        Combinator::Child => " > ",
        Combinator::AdjacentSibling => " + ",
        Combinator::GeneralSibling => " ~ ",
    }
}

/// 1つの`Selector`(結合子で連結されたコンパウンドセレクタ列)を
/// `div.foo > p`のようなCSSテキストへ戻す。
pub fn selector_to_string(selector: &Selector) -> String {
    let mut out = String::new();
    for (i, segment) in selector.iter().enumerate() {
        if i > 0 {
            out.push_str(combinator_prefix(segment.combinator));
        }
        for part in &segment.compound {
            out.push_str(&simple_part_to_string(part));
        }
    }
    out
}

fn declaration_to_string(decl: &Declaration, indent: &str) -> String {
    let important = if decl.important { " !important" } else { "" };
    format!("{indent}{}: {}{important};", decl.property, decl.value)
}

/// 1つの`Rule`(カンマ区切りセレクタ・宣言列、`media`は含まない)を
/// `{indent}selector1, selector2 {\n  prop: value;\n}\n`形式へ。
fn rule_body_to_string(rule: &Rule, indent: &str) -> String {
    let selectors = rule.selectors.iter().map(selector_to_string).collect::<Vec<_>>().join(", ");
    let mut out = format!("{indent}{selectors} {{\n");
    let inner_indent = format!("{indent}  ");
    for decl in &rule.declarations {
        out.push_str(&declaration_to_string(decl, &inner_indent));
        out.push('\n');
    }
    out.push_str(indent);
    out.push_str("}\n");
    out
}

fn media_type_to_string(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Screen => "screen",
        MediaType::Print => "print",
    }
}

/// `MediaQuery`を`parse_media_query`が受理する構文へ戻す
/// (`screen and (min-width: 768px)`のような文字列)。全フィールドが
/// `None`の場合(`@media { ... }`という空条件になってしまう)は
/// `"all"`を返す——空文字列より安全側の出力(構文として無効にしない)。
pub fn media_query_to_string(query: &MediaQuery) -> String {
    let mut parts = Vec::new();
    if let Some(media_type) = query.media_type {
        parts.push(media_type_to_string(media_type).to_string());
    }
    if let Some(min) = query.min_width_px {
        parts.push(format!("(min-width: {min}px)"));
    }
    if let Some(max) = query.max_width_px {
        parts.push(format!("(max-width: {max}px)"));
    }
    if let Some(w) = query.width_px {
        parts.push(format!("(width: {w}px)"));
    }
    if parts.is_empty() {
        "all".to_string()
    } else {
        parts.join(" and ")
    }
}

/// スタイルシート全体(`Vec<Rule>`)を実際の`.css`テキストへ変換する。
/// 出現順を保ったまま、連続する同一`@media`条件の`Rule`を1つの
/// `@media`ブロックへグルーピングする(隣接していない同条件の`Rule`は
/// あえてマージしない——出現順の意味を保つため、`RBootStrap`側の
/// 「`col-md-4`は`min-width:768px`」のような単体`Rule`をそのまま
/// 素直に出力する用途を優先した設計)。
pub fn stylesheet_to_string(rules: &[Rule]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < rules.len() {
        let rule = &rules[i];
        match &rule.media {
            None => {
                out.push_str(&rule_body_to_string(rule, ""));
            }
            Some(query) => {
                // 連続する同一@media条件のRuleを1ブロックにまとめる。
                let mut j = i + 1;
                while j < rules.len() && rules[j].media.as_ref() == Some(query) {
                    j += 1;
                }
                out.push_str(&format!("@media {} {{\n", media_query_to_string(query)));
                for r in &rules[i..j] {
                    out.push_str(&rule_body_to_string(r, "  "));
                }
                out.push_str("}\n");
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_stylesheet;

    #[test]
    fn serializes_a_simple_rule_round_trip() {
        let rules = parse_stylesheet("p { color: red; font-size: 12px; }");
        let css = stylesheet_to_string(&rules);
        assert_eq!(css, "p {\n  color: red;\n  font-size: 12px;\n}\n");
        // 再パースしても同じ内容に戻ることを確認(ラウンドトリップ)。
        let reparsed = parse_stylesheet(&css);
        assert_eq!(reparsed, rules);
    }

    #[test]
    fn serializes_combinators_back_to_css_syntax() {
        let rules = parse_stylesheet("div > p { color: blue; } li + li { color: green; } .a ~ .b { color: red; }");
        let css = stylesheet_to_string(&rules);
        assert!(css.contains("div > p {"));
        assert!(css.contains("li + li {"));
        assert!(css.contains(".a ~ .b {"));
    }

    #[test]
    fn serializes_important_declarations() {
        let rules = parse_stylesheet(".a { color: red !important; }");
        let css = stylesheet_to_string(&rules);
        assert_eq!(css, ".a {\n  color: red !important;\n}\n");
    }

    #[test]
    fn groups_rules_under_a_media_block() {
        let rules = parse_stylesheet("@media screen and (min-width: 768px) { .col-md-4 { width: 33%; } }");
        let css = stylesheet_to_string(&rules);
        assert_eq!(css, "@media screen and (min-width: 768px) {\n  .col-md-4 {\n    width: 33%;\n  }\n}\n");
        // ラウンドトリップ: 再パースしてmedia条件が保持されることを確認。
        let reparsed = parse_stylesheet(&css);
        assert_eq!(reparsed, rules);
    }

    #[test]
    fn consecutive_rules_with_the_same_media_query_share_one_block() {
        let rules = parse_stylesheet(
            "@media (min-width: 768px) { .a { color: red; } .b { color: blue; } }",
        );
        let css = stylesheet_to_string(&rules);
        // 1つの@mediaブロックにまとまり、2回出力されないことを確認。
        assert_eq!(css.matches("@media").count(), 1);
        assert!(css.contains(".a {"));
        assert!(css.contains(".b {"));
    }

    #[test]
    fn top_level_and_media_rules_can_be_mixed() {
        let rules = parse_stylesheet("p { color: red; } @media print { p { color: black; } }");
        let css = stylesheet_to_string(&rules);
        let reparsed = parse_stylesheet(&css);
        assert_eq!(reparsed, rules);
    }

    #[test]
    fn media_query_to_string_falls_back_to_all_when_empty() {
        assert_eq!(media_query_to_string(&MediaQuery::default()), "all");
    }
}

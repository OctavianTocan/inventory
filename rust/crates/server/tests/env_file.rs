//! Frozen synthetic env-parser expectations, preserving the fixed generated corpus and sorted map order.

#[path = "../../core/tests/common/mod.rs"]
mod common;

use mm_core::{Object, Value};
use mymind::env_file;

const DRIVER: &str = "rust/crates/server/tests/drivers/env.mjs";

fn sorted(vars: std::collections::HashMap<String, String>) -> Value {
    let mut pairs: Vec<_> = vars.into_iter().collect();
    pairs.sort();
    Value::Object(pairs.into_iter().map(|(name, value)| (name, Value::from(value))).collect::<Object>())
}

#[test]
fn env_files_parse_as_node_parses_them() {
    let mut inputs: Vec<String> = [
        "A=1", "export  B = x # c", "C=\"a\\nb\" junk", "D='q'", "E=`t`", "=v", "  F  =  g  ", "G", "H=\"unterminated\nI=2",
        "J=a#b", "K=\"x\"#y", "\tL=1", "M=\t2\t", "N= \"spaced\"", "#c\nO=1", "P=1\r\nQ=2", "R=\"\"", "S='it's'",
        "T=\"line\nnext\"", "export=1", "U==x", "export V", "export X=\nY=1", "Z=\n", "\n\n  \nA=1\n\n#x\nB=2", "Ä=ö # ü",
    ]
    .map(str::to_owned)
    .into();
    let alphabet = ["A", "B", "=", "#", "\"", "'", "`", " ", "\t", "\n", "\r", "\\n", "export ", "x", "é"];
    let mut random = common::Random(0x5eed_e17f);
    for _ in 0..20_000 {
        let length = 1 + random.below(40) as usize;
        inputs.push(random.text(&alphabet, length));
    }
    let calls: Vec<(&str, Vec<Value>)> = inputs.iter().map(|input| ("parseEnv", vec![Value::from(input.as_str())])).collect();
    let answers = common::calls(DRIVER, &calls);
    assert_eq!(inputs.len(), 20_026, "synthetic env-parser case count");
    assert_eq!(answers.len(), inputs.len(), "env-parser answer count");
    for (input, answer) in inputs.iter().zip(answers) {
        common::assert_same(&Ok(sorted(env_file::parse(input))), &answer, &format!("{input:?}"));
    }
}

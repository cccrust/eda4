use v2f_core::Device;

#[test]
fn test_pnr_from_synth_json() {
    let json = r#"{
        "creator": "v2f-synth v0.3",
        "modules": {
            "top": {
                "ports": {
                    "a": { "direction": "input", "bits": [1] },
                    "y": { "direction": "output", "bits": [2] }
                },
                "cells": {},
                "netnames": {}
            }
        }
    }"#;
    let asc = v2f_pnr::run_pnr(json, Device::HX8K);
    assert!(asc.contains(".device"));
    assert!(asc.contains("HX8K"));
}

#[test]
fn test_pnr_asc_contains_sym() {
    let json = r#"{
        "creator": "v2f-synth v0.3",
        "modules": {
            "top": {
                "ports": {
                    "a": { "direction": "input", "bits": [3] }
                },
                "cells": {
                    "$1": {
                        "type": "$_AND_",
                        "parameters": {},
                        "connections": { "A": [3], "B": [4], "Y": [5] }
                    }
                },
                "netnames": {
                    "a": { "bits": [3], "hide_name": 0 }
                }
            }
        }
    }"#;
    let asc = v2f_pnr::run_pnr(json, Device::HX1K);
    assert!(asc.contains(".sym"));
}

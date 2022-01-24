#[cfg(feature = "_fuzz")]
mod test {
    use fuzzcheck::fuzz_test;

    use puck_liveview::{
        html::id::IdGen,
        prelude::{BodyNode, IntoWrappedBodyNode},
    };

    #[allow(unused)]
    fn test_diff((before, after): &(BodyNode, BodyNode)) -> bool {
        let before = before.clone();
        let after = after.clone();
        let mut before = before.wrap().into_element(&mut IdGen::new());
        let after = after.wrap().into_element(&mut IdGen::new());

        let mut before2 = before.clone();
        let cs = before2.diff(Some(&after));

        cs.apply(&mut before);

        before == after
    }

    #[test]
    fn fuzz_diffing() {
        let res = fuzz_test(test_diff).default_options().launch();
        assert!(!res.found_test_failure);
    }

    fn test_regression(data: &str) {
        let data: (BodyNode, BodyNode) = serde_json::from_str(data).unwrap();

        assert!(test_diff(&data));
    }

    #[test]
    fn many_cov_hits() {
        test_regression(r#"[{"A":{"attrs":{},"text":""}},{"Label":{"text":"","attrs":{}}}]"#);
        test_regression(
            r#"[{"Img":{"attrs":{"9":"6","p":"","F":""}}},{"H2":{"text":"","attrs":{"p":"","C":"","9":""}}}]"#,
        );
    }

    #[test]
    fn max_cov_hits() {
        test_regression(
            r#"[
            {
                "H2": {
                    "text": "",
                    "attrs": {
                        "73": "",
                        "F": "k",
                        "1CB66bp7saGlSaY1j": "p3AX",
                        "g54Wr": "",
                        "t": "Y",
                        "O": "Z7",
                        "Iaq6": "",
                        "H": "",
                        "46": "7",
                        "S3": "",
                        "d": "",
                        "FeB": "",
                        "e6": "1",
                        "eo": "",
                        "2": "",
                        "1": "3",
                        "11": "V",
                        "o": "",
                        "N": "h",
                        "8": "A",
                        "03": "",
                        "i": "",
                        "pa": "",
                        "312RPqrkF6nFuk3vdhZ8FOL": "8FGWGKcAn6e",
                        "de4": "q",
                        "L": "",
                        "K": "",
                        "88": "",
                        "NX2T": "",
                        "m0": "",
                        "m": "",
                        "V8": "",
                        "El": "e2",
                        "0": "",
                        "1N": "b",
                        "w01": "6",
                        "f": "",
                        "b3": "",
                        "9": "uni",
                        "sf": "",
                        "nR": "",
                        "4h5sjgcV5juESPjm2I5VilqD": "7q35G8Y0AR4C6A",
                        "6b8": "",
                        "D": "",
                        "z": "s",
                        "kY": "",
                        "Oc": "",
                        "6": "",
                        "Y": "",
                        "aN": "",
                        "Z": "",
                        "r": "",
                        "M": "",
                        "e": "iiP",
                        "C": "9",
                        "B": "",
                        "0J": "",
                        "3orR4w7pOO35q": "",
                        "Us": "",
                        "5": "",
                        "7": "",
                        "1ry962G8klM5v41qDpfvQ093BGfRonf9Ij1p": "zp8K3Zf6JoiLg",
                        "I": "",
                        "l": "",
                        "c": "",
                        "er": "",
                        "X3": "",
                        "w": "J5Y",
                        "R": "7",
                        "U": "",
                        "n": "",
                        "8N": "",
                        "Ne": "",
                        "oK": "",
                        "p": "wkNqQ",
                        "u": "",
                        "g": "",
                        "b": "F",
                        "4OB": "",
                        "Q": "",
                        "V": "",
                        "A": "",
                        "6VDM": "3wN1s",
                        "h": "N",
                        "48": "",
                        "G": "5whqU",
                        "N3": "",
                        "X": "",
                        "s": "",
                        "68": "6",
                        "S": "",
                        "a": "",
                        "Awrv1Eb128Wc": "m1v",
                        "k": "",
                        "y7": "",
                        "jtq": "",
                        "x": "",
                        "q": "e",
                        "ep": "",
                        "x0MhH": "",
                        "v": "",
                        "4": "",
                        "01": "4FTb1",
                        "P": "X",
                        "k40": "L",
                        "E": "",
                        "T": "",
                        "y": "K",
                        "jsdTvQrrmK74022": "OTDJlkZM71v4lNW8",
                        "GUw": "OcUB9G"
                    }
                }
            },
            {
                "Form": {
                    "children": [
                        {
                            "A": {
                                "attrs": {},
                                "text": ""
                            }
                        },
                        {
                            "Label": {
                                "text": "",
                                "attrs": {}
                            }
                        },
                        {
                            "Input": {
                                "attrs": {}
                            }
                        },
                        {
                            "Br": null
                        },
                        {
                            "Form": {
                                "children": [
                                    {
                                        "Div": {
                                            "children": [],
                                            "attrs": {}
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "P": {
                                "attrs": {},
                                "text": "",
                                "children": [
                                    {
                                        "Form": {
                                            "children": [],
                                            "attrs": {}
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "NoScript": {
                                "text": ""
                            }
                        },
                        {
                            "Div": {
                                "children": [
                                    {
                                        "Form": {
                                            "children": [
                                                {
                                                    "NoScript": {
                                                        "text": ""
                                                    }
                                                }
                                            ],
                                            "attrs": {}
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "P": {
                                "attrs": {},
                                "text": "",
                                "children": [
                                    {
                                        "P": {
                                            "attrs": {},
                                            "text": "",
                                            "children": [
                                                {
                                                    "Div": {
                                                        "children": [],
                                                        "attrs": {}
                                                    }
                                                }
                                            ]
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "P": {
                                "attrs": {},
                                "text": "",
                                "children": []
                            }
                        },
                        {
                            "Form": {
                                "children": [
                                    {
                                        "Div": {
                                            "children": [],
                                            "attrs": {}
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "P": {
                                "attrs": {},
                                "text": "",
                                "children": [
                                    {
                                        "Form": {
                                            "children": [],
                                            "attrs": {}
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "P": {
                                "attrs": {},
                                "text": "",
                                "children": []
                            }
                        },
                        {
                            "Div": {
                                "children": [
                                    {
                                        "Form": {
                                            "children": [
                                                {
                                                    "Div": {
                                                        "children": [],
                                                        "attrs": {}
                                                    }
                                                }
                                            ],
                                            "attrs": {}
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "Div": {
                                "children": [
                                    {
                                        "Div": {
                                            "children": [],
                                            "attrs": {}
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "Div": {
                                "children": [],
                                "attrs": {}
                            }
                        },
                        {
                            "Form": {
                                "children": [],
                                "attrs": {}
                            }
                        },
                        {
                            "Div": {
                                "children": [],
                                "attrs": {}
                            }
                        },
                        {
                            "Form": {
                                "children": [
                                    {
                                        "Div": {
                                            "children": [
                                                {
                                                    "P": {
                                                        "attrs": {},
                                                        "text": "",
                                                        "children": []
                                                    }
                                                }
                                            ],
                                            "attrs": {}
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "Div": {
                                "children": [
                                    {
                                        "P": {
                                            "attrs": {},
                                            "text": "",
                                            "children": []
                                        }
                                    }
                                ],
                                "attrs": {}
                            }
                        },
                        {
                            "A": {
                                "attrs": {},
                                "text": ""
                            }
                        },
                        {
                            "P": {
                                "attrs": {},
                                "text": "",
                                "children": [
                                    {
                                        "Form": {
                                            "children": [],
                                            "attrs": {}
                                        }
                                    }
                                ]
                            }
                        }
                    ],
                    "attrs": {
                        "3": "",
                        "9": "",
                        "Ye": "",
                        "G": "L",
                        "n": "e3N",
                        "sZ": "H",
                        "220H": "",
                        "P": ""
                    }
                }
            }
        ]"#,
        );
    }
}

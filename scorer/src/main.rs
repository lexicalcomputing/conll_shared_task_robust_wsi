#![allow(non_snake_case)]
use polars::prelude::*;

fn usage() {
    eprintln!("usage: scorer INFILE [-c <CLUSTER_COL>] [-f <CLUSTER_FILE>]");
    eprintln!("");
    eprintln!(" INFILE is a tab-separated file, with a header and no quoting");
    eprintln!("     containing at least:");
    eprintln!("         - 'head' column");
    eprintln!("         - two or more columns with prefix 'sense' with the sense annotation");
    eprintln!("");
    eprintln!(" When -c CLUSTER_COL is present, the WSI system output is read from the");
    eprintln!("         column named CLUSTER_COL (default='cluster')");
    eprintln!("");
    eprintln!(" When -f CLUSTER_FILE is present, the WSI system output will be read from");
    eprintln!("         CLUSTER_FILE, a tab-separated file with a header a no quoting, with");
    eprintln!("         rows in the same order as in INFILE.");
    eprintln!("         The CLUSTER_COL option applies to CLUSTER_FILE as well.");
    eprintln!("");
    eprintln!(" The cluster values are arbitrary strings, only equality is considered, except");
    eprintln!(" for the 'sense' columns, for which values ending with 'x' are ignored.");
    eprintln!("");
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    args.next();  // skip argv[0], the name of the binary
    let infile = if let Some(infile) = args.next() {
        infile
    } else {
        usage();
        return Err("error: no input file".into());
    };

    let mut cluster_file = None;
    let mut cluster_colname = "cluster".to_string();

    loop {
        match args.next().as_deref() {
            Some("-f") => {
                cluster_file = Some(args.next()
                    .ok_or("-f expects CLUSTER_FILE argument")?);
            },
            Some("-c") => {
                cluster_colname = args.next()
                    .ok_or("-f expects CLUSTER_COL argument")?;
            },
            Some(_) => {
                usage();
                return Err("error: unexpected commandline arguments".into());
            },
            None => { break; }
        }
    }

    let csvopts = CsvReadOptions::default()
        .with_parse_options(CsvParseOptions::default()
            .with_separator(b'\t')
            .with_quote_char(None)
        ).with_has_header(true);
    let df_infile = csvopts.clone()
        .try_into_reader_with_file_path(Some(infile.into()))?
        .finish()?;

    let df = if cluster_file.is_some() {
        let mut df_clusterfile = csvopts
            .with_columns(Some(vec![cluster_colname.clone().into()].into()))
            .try_into_reader_with_file_path(Some(cluster_file.unwrap().into()))?
            .finish()?;
        df_clusterfile.rename(&cluster_colname, (cluster_colname.clone() + "__clusterfile__").into())?;
        cluster_colname = cluster_colname + "__clusterfile__";
        polars::functions::concat_df_horizontal(&[df_infile, df_clusterfile], true)?
    } else {
        df_infile
    };

    let gr = df.group_by(["head"])?;

    let f = |df: DataFrame| {
        let head = df["head"].try_str().unwrap().get(0).unwrap();
        let mut sense_cols = vec![];
        for col in df.get_columns() {
            if col.name().starts_with("sense") && *col.name() != cluster_colname {
                sense_cols.push(col.try_str().unwrap());
            }
        }

        let cluster_col = df.column(&cluster_colname)?.try_str().unwrap();

        let mut vals1 = vec![];
        let mut vals2 = vec![];

        let mut _total_pairs = 0;
        let mut TP = 0u64; let mut FP = 0u64;  // true positive, false positive
        let mut TN = 0u64; let mut FN = 0u64;  // true negative, false negative
        let mut UP = 0u64; let mut UN = 0u64;  // unknown positive, unknown negative
        let mut TPw = 0f64; let mut FPw = 0f64;  // true positive, false positive
        let mut TNw = 0f64; let mut FNw = 0f64;  // true negative, false negative

        for i in 0..df.height() {
            vals1.clear(); vals1.extend(sense_cols.iter().map(|ca| ca.get(i).unwrap()));
            for j in 0..df.height() {
                vals2.clear(); vals2.extend(sense_cols.iter().map(|ca| ca.get(j).unwrap()));

                let total_senses = vals1.len();
                let mut valid_senses = 0;
                let mut matching_senses = 0;
                for sense_ix in 0..vals1.len() {
                    let s1 = vals1[sense_ix];
                    let s2 = vals2[sense_ix];
                    if s1.ends_with('x') || s2.ends_with('x') {
                        // unset sense, cannot be matching
                    } else {
                        valid_senses += 1;
                        if s1 == s2 {
                            matching_senses += 1;
                        } else {

                        }
                    }
                }

                let same_cluster = cluster_col.get(i).unwrap() == cluster_col.get(j).unwrap();
                let threshold = 0.25f64;

                if valid_senses as f64 / total_senses as f64 > 0.5 {
                    let matching_ratio = matching_senses as f64 / valid_senses as f64;
                    let w = 2.*(0.5 - matching_ratio).abs();
                    if same_cluster {
                        if matching_ratio >= 1.0f64 - threshold {
                            TP += 1;
                        } else if matching_ratio <= threshold {
                            FP += 1;
                        } else {
                            UP += 1;
                        }

                        if matching_ratio > 0.5 {
                            TPw += w;
                        } else {
                            FPw += w;
                        }
                    } else {
                        if matching_ratio >= 1.0f64 - threshold {
                            FN += 1;
                        } else if matching_ratio <= threshold {
                            TN += 1;
                        } else {
                            UN += 1;
                        }

                        if matching_ratio > 0.5 {
                            FNw += w;
                        } else {
                            TNw += w;
                        }
                    }

                    _total_pairs += 1;
                }
            }
        }
        eprintln!("processed {}", head);
        let Precision = TP as f64/(TP+FP) as f64;
        let Recall = TP as f64/(TP+FN) as f64;
        let F1 = 2.*(Precision*Recall)/(Precision+Recall);
        let RI = (TP+TN) as f64/(TP+TN+FP+FN) as f64;
        let sRI = 2.*(TP as f64*TN as f64-FP as f64*FN as f64)/
            ((TN as f64+FN as f64)*(TP as f64+FP as f64) +
             (TN as f64+FP as f64)*(TP as f64+FN as f64));
        let wsRI = 2.*(TPw*TNw-FPw*FNw)/
            ((TNw+FNw)*(TPw+FPw) + (TNw+FPw)*(TPw+FNw));

        df!("head" => &[head],
            "RI" => &[RI],
            "sRI" => &[sRI],
            "wsRI" => &[wsRI],
            "TP" => &[TP], "FP" => &[FP],
            "TN" => &[TN], "FN" => &[FN],
            "UP" => &[UP], "UN" => &[UN],
            "Precision" => &[Precision],
            "Recall" => &[Recall],
            "F1" => &[F1],
            "Instances" => &[df.height() as u64])
    };
    let mut aggregated = gr.apply(f)?;
    println!("mean RI: {}", aggregated.column("RI")?.mean_reduce().value());
    println!("mean sRI: {:?}", aggregated.column("sRI")?.mean_reduce().value());
    println!("mean wsRI: {:?}", aggregated.column("wsRI")?.mean_reduce().value());
    CsvWriter::new(&mut std::io::stdout())
        .include_header(true)
        .include_bom(false)
        .with_separator(b'\t')
        .with_quote_style(QuoteStyle::Never)
        .finish(&mut aggregated)?;
    Ok(())
}


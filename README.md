# CoNLL 2025 Shared Task: Robust WSI

This is a repository containing data for the Robust Word Sense Induction shared task.

For details, please visit the the [website](https://projects.sketchengine.eu/conll2025/).

## Sample Files

The `sample/` directory contains the annotated test sets for three words for each language.


The files are encoded as UTF-8 and use columnar format separated by TAB characters. No quoting is used and the first line describes the names of the columns. All the files have the same structure.

 - Column `head` represents the headword.
 - Columns starting with `sense` represent the "gold" annotations, one column per annotator. Value ending with an `x` means that the annotator has not marked this line in any way.
 - Column `text` contains the the sentence, within which the specific occurrence appears.
  
## The Scorer Program
To obtain a good performance, is written in `Rust`, the source code is in the `scorer/` directory, a prebuilt static binary for x86\_64 Linux is present in the `scorer/bin/` directory.

### Usage
Annotate the test set using your own WSI system and create a TSV file containing a column with the cluster labels. A header needs to be present. The default name for the cluster column is `cluster`. Other columns might be present as well. You can also place the column with the cluster labels into the file containing the gold data.

Then run the scorer and observe the output:

    ./bin/scorer GOLD_FILE -f CLUSTER_FILE

To change the name of the cluster column, use the `-c` option. If your labels are in the same file as the gold data is, omit the `-f` option.

### Compilation
To build the program yourself, install Rust using [https://rustup.rs/](https://rustup.rs/) and then run `cargo build --release` from the `scorer/` directory.

## Licensing

Shield: [![CC BY-NC-SA 4.0][cc-by-nc-sa-shield]][cc-by-nc-sa]

This work is licensed under a [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License][cc-by-nc-sa].

[![CC BY-NC-SA 4.0][cc-by-nc-sa-image]][cc-by-nc-sa]

[cc-by-nc-sa]: http://creativecommons.org/licenses/by-nc-sa/4.0/
[cc-by-nc-sa-image]: https://licensebuttons.net/l/by-nc-sa/4.0/88x31.png
[cc-by-nc-sa-shield]: https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg

## Contact

Do not hesitate to contact us at [conll2025@sketchengine.eu](mailto:conll2025@sketchengine.eu).


# High Energy (Physics) Generator Analyser

**Formats:**

- EPOS4+ (OSC1999A)
- PHSD (.dat)
- UrQMD (OSC1997A)

## Realisation

- Rust (2024)
- uses [rayon](https://docs.rs/rayon/latest/rayon/) for multithreading
- [printer.ipynb](https://github.com/YoitzWolf/hega-rs/printer.ipynb]) - printing results example

## To Build

build: `cargo build --release`

## To Use

`hega-rs.exe --help`

`genarg.bat` - example how to generate large FILENAMES argument for Windows

## Output

Output is defined by criteria list:

```rust
    let criteria = vec![
        StandardCriteria::FinEnergy,
        StandardCriteria::ECharge,
        StandardCriteria::BCharge,
        StandardCriteria::LCharge,
        StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
        StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
        StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
        
    ];
```

And output file name is defined with `-o` option

The result is csv file where each line describes an event:

```csv
    FinEnergy;	ECharge;	BCharge;	LCharge;	PseudorapidityFilterCnt(-0.5, 0.5);	PseudorapidityFilterCnt(-1.0, 1.0);	PseudorapidityFilterCnt(-1.5, 1.5)
    7000.000251383995;	2;	2;	0;	0;	0;	0
    7000.000251383995;	2;	2;	0;	0;	0;	0
    7000.000251383995;	2;	2;	0;	0;	0;	0
    ...
```

Columns are the same as criteria!

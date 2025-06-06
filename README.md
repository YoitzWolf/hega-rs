# High Energy (Physics) Generator Analyser

**Formats:**

- EPOS4+ (OSC1999A)
- PHSD (.dat)
- UrQMD (OSC1997A)

## Realisation

- Rust (2024)
- uses [rayon](https://docs.rs/rayon/latest/rayon/) for multithreading
- [printer.ipynb](https://github.com/YoitzWolf/hega-rs/blob/master/printer.ipynb]) - printing results example

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


## Custom Criteria

Now only scalar result values are implemented!

Custom criteria have to implement trait (see) `https://github.com/YoitzWolf/hega-rs/blob/master/src/anlz/generic.rs`:

```rust
pub trait ScalarCriteria<'a, S, T>: Sized + PartialEq + Debug + Clone + Send
where T: Particle<Decoder = S> + 'static,
    // S: 'a
{
    fn get_criteria_value(&self, p: &T, dec: &S) -> f64;// Clone + Send + Sync + 'a;

    fn name(&self) -> String;
}
```

after that it could be sent to analyzer in criteria vector:

```rust
    pub fn calculate_criteria
    (   
            &self,
            filter: impl (Fn(&Event::P, &<Event::P as Particle>::Decoder) -> bool) + Sync,
            criteria: Vec<impl Sync + ScalarCriteria<'a, <Event::P as Particle>::Decoder, Event::P> >,//Vec<T>,
            dec: &<Event::P as Particle>::Decoder
    ) -> ScalarAnalyzerResults
    where
        <Event as HEPEvent>::P: 'static ,
        <Event::P as Particle>::Decoder: Sync
```

Example is showed at `https://github.com/YoitzWolf/hega-rs/blob/master/src/custom_criteria.rs`

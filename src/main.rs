use std::{
    fs::File,
    io::{BufReader, Write},
};

use anlz::{HEPEventAnalyzer, StandardCriteria};
use fmt::{decoder::EposDict, generic::GenericDataContainer, phqmd::PHQMDBlock};

use rayon::prelude::*;

mod anlz;
mod fmt;

fn main() {
    let dict_lepto = EposDict::upload(
        BufReader::new(File::open("./dicts/EPOS_LEPTONS.particles.txt").unwrap()),
        fmt::decoder::DctCoding::PDG,
        None,
    );
    let dict_EPOS = EposDict::upload(
        BufReader::new(File::open("./dicts/EPOS.particles.txt").unwrap()),
        fmt::decoder::DctCoding::PDG,
        Some(dict_lepto.codes().cloned().collect())
    );
    drop(dict_lepto);

    // let dictBaryo = EposDict::upload(BufReader::new(File::open("./dicts/EPOS_BARYONS.particles.txt").unwrap()), fmt::decoder::DctCoding::PDG);

    // let f = File::open("C:\\Users\\yoitz\\Documents\\Reports\\s4Check\\sim\\eposSimBig\\eposSim\\epos_3\\results\\z-pp7000.data").unwrap();
    // let oscFile = fmt::oscar::OSCEposDataFile::upload(BufReader::new(f), &dict).unwrap();

    let start = SystemTime::now();

    let f = File::open("C:/Users/yoitz/PHQMD/pp7tdata/phsd.dat").unwrap();
    let mut phqmdFile = fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap();

    let f = File::open("C:/Users/yoitz/PHQMD/pp7tdata/phsd2.dat").unwrap();
    phqmdFile.push_back(
        fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
    );

    let f = File::open("C:/Users/yoitz/PHQMD/pp7tdata/phsd3.dat").unwrap();
    phqmdFile.push_back(
        fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
    );

    let f = File::open("C:/Users/yoitz/PHQMD/pp7tdata/phsd4.dat").unwrap();
    phqmdFile.push_back(
        fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
    );

    let f = File::open("C:/Users/yoitz/PHQMD/pp7tdata/phsd5.dat").unwrap();
    phqmdFile.push_back(
        fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
    );


    use std::time::SystemTime;
    let end = start.elapsed().unwrap();
    println!("READING DONE: {} s", end.as_secs_f64());


    // ANALYSER
    
    let start = SystemTime::now();
    let analyzer = HEPEventAnalyzer::new(phqmdFile.get_blocks());
    let v = analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, vec![
        StandardCriteria::FinEnergy,
        StandardCriteria::ECharge,
        StandardCriteria::BCharge,
        StandardCriteria::LCharge,
        StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
        StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
        StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
        
    ], &dict_EPOS);
    let headers = v.headers();
    let res = v.values();

    /*let res = phqmdFile
        .get_blocks()
        .par_iter()
        .map(|event| {
            event
                .event
                .iter()
                .fold((0., 0., 0), |(mut E, mut B, mut L), p| {
                    {
                        // LAST GEN
                        //ENERGY
                        E += p.E;
                        //BARYON CHARGE
                        if let Some(pd) = dict.get(&p.code) {
                            B += [
                                pd.ifl1.unwrap_or(0),
                                pd.ifl2.unwrap_or(0),
                                pd.ifl3.unwrap_or(0),
                            ]
                            .iter()
                            .map(|&x| {
                                if (x > 0) {
                                    1.
                                } else if (x < 0) {
                                    -1.
                                } else {
                                    0.
                                }
                            })
                            .sum::<f64>()
                                / 3.0;
                        } else if let Some(apd) = dict.get(&-(p.code)) {
                            B -= [
                                apd.ifl1.unwrap_or(0),
                                apd.ifl2.unwrap_or(0),
                                apd.ifl3.unwrap_or(0),
                            ]
                            .iter()
                            .map(|&x| {
                                if (x > 0) {
                                    1.
                                } else if (x < 0) {
                                    -1.
                                } else {
                                    0.
                                }
                            })
                            .sum::<f64>()
                                / 3.0;
                        }
                        // LEPTON CHARGE
                        if dictLepto.get(&p.code).is_some() {
                            L += 1;
                        } else if dictLepto.get(&-(p.code)).is_some() {
                            L -= 1;
                        }
                    }
                    (E, B, L)
                })
        })
        .collect::<Vec<_>>();*/

    // headers = "E[GeV];\tB;\tL\n".as_bytes()
    let end = start.elapsed().unwrap();
    println!("CALCULATION DONE: {} s", end.as_secs_f64());
    let mut f = File::create("./phsd.csv.stat").unwrap();
    f.write((headers.join(";\t") + "\n").as_bytes()).unwrap();
    res.iter().for_each(|vals| {
        f.write((vals.iter().map(ToString::to_string).collect::<Vec<_>>().join(";\t") + "\n").as_bytes())
            .unwrap();
    });
}

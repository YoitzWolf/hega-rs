mod anlz;
mod fmt;
mod custom_criteria;
use custom_criteria::*;


use std::{
    fs::File,
    io::{BufReader, Write},
};

use anlz::{HEPEventAnalyzer, Particle, ScalarCriteria, StandardCriteria};
use fmt::{decoder::EposDict, generic::GenericDataContainer, oscar::OSCEposBlock, phqmd::{PHQMDBlock, PHQMDParticle}};

use clap::{Parser, *};


#[derive(
    clap::ValueEnum, Clone, Debug, Default
)]
pub enum AcceptedTypes {
    #[default]
    EPOS,
    UrQMD_F19,
    PHQMD
}

/*
#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct FileInput {
    #[default]
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    filenames: Vec<String>,
    
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    dirname: String,
}*/


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Type of file
    ftype: AcceptedTypes,

    /// List of files, delimeter ','. Use "quotes" if path contains whitespaces
    #[clap(short, long, num_args = 1.., value_delimiter = ',')]
    filenames: Vec<String>,

    #[clap(short, long="output", default_value="results.csv.stat")]
    o: String
}


fn main() {
    let dict_EPOS;

    let args = Args::parse();

    match args.ftype {
        AcceptedTypes::EPOS => {
            let dict_lepto = EposDict::upload(
                BufReader::new(File::open("./dicts/EPOS_LEPTONS.particles.txt").unwrap()),
                fmt::decoder::DctCoding::EPOS,
                None,
            );
            dict_EPOS = EposDict::upload(
                BufReader::new(File::open("./dicts/EPOS.particles.txt").unwrap()),
                fmt::decoder::DctCoding::EPOS,
                Some(dict_lepto.codes().cloned().collect())
            );
            drop(dict_lepto);
            /* ----------------------------------------------------------------------------------------------- */

            
        },
        AcceptedTypes::PHQMD | AcceptedTypes::UrQMD_F19 => {
            let dict_lepto = EposDict::upload(
                BufReader::new(File::open("./dicts/EPOS_LEPTONS.particles.txt").unwrap()),
                fmt::decoder::DctCoding::PDG,
                None,
            );
            dict_EPOS = EposDict::upload(
                BufReader::new(File::open("./dicts/EPOS.particles.txt").unwrap()),
                fmt::decoder::DctCoding::PDG,
                Some(dict_lepto.codes().cloned().collect())
            );
            drop(dict_lepto);
            /* ----------------------------------------------------------------------------------------------- */
        },
    }

    // let dictBaryo = EposDict::upload(BufReader::new(File::open("./dicts/EPOS_BARYONS.particles.txt").unwrap()), fmt::decoder::DctCoding::PDG);

    // let f = File::open("C:\\Users\\yoitz\\Documents\\Reports\\s4Check\\sim\\eposSimBig\\eposSim\\epos_3\\results\\z-pp7000.data").unwrap();
    // let oscFile = fmt::oscar::OSCEposDataFile::upload(BufReader::new(f), &dict).unwrap();

    

    /*let f = File::open("C:/Users/yoitz/PHQMD/pp7tdata/phsd.dat").unwrap();
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
    );*/

    use std::time::SystemTime;

    // ANALYSER

    // println!("{:?}", args.filename);
    
    let start = SystemTime::now();
    let v = match args.ftype {
        AcceptedTypes::EPOS => {
            let criteria: Vec< &dyn ScalarCriteria<'_, _, _> > = vec![
                &StandardCriteria::FinEnergy,
                &StandardCriteria::ECharge,
                &StandardCriteria::BCharge,
                &StandardCriteria::LCharge,
                &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                &custom_criteria::MyExampleCriterias::StupidCriteria1
            ];
            let start = SystemTime::now();
            let eposFile = args.filenames.iter().fold(None, 
                |mut fo:Option<fmt::oscar::OSCEposDataFile>, x| {
                    println!(">> FILE READING [{}]", x);
                    let f = File::open(&x).unwrap();
                    if let Some(mut fo) = fo {
                        fo.push_back(
                            fmt::oscar::OSCEposDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
                        );
                        Some(fo)            
                    } else {
                        Some(fmt::oscar::OSCEposDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap())
                    }
                }
            ).unwrap();
            let end = start.elapsed().unwrap();
            println!("READING DONE: {} s", end.as_secs_f64());
            let analyzer = HEPEventAnalyzer::new(eposFile.get_blocks());
            analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict_EPOS)
        },
        AcceptedTypes::UrQMD_F19 => {
            let criteria: Vec< &dyn ScalarCriteria<'_, _, _> > = vec![
                &StandardCriteria::FinEnergy,
                &StandardCriteria::ECharge,
                &StandardCriteria::BCharge,
                &StandardCriteria::LCharge,
                &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                &custom_criteria::MyExampleCriterias::StupidCriteria1
            ];
            let start = SystemTime::now();
            let eposFile = args.filenames.iter().fold(None, 
                |mut fo:Option<fmt::oscar::OSC97UrQMDDataFile>, x| {
                    println!(">> FILE READING [{}]", x);
                    let f = File::open(&x).unwrap();
                    if let Some(mut fo) = fo {
                        fo.push_back(
                            fmt::oscar::OSC97UrQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
                        );
                        Some(fo)            
                    } else {
                        Some(fmt::oscar::OSC97UrQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap())
                    }
                }
            ).unwrap();
            let end = start.elapsed().unwrap();
            println!("READING DONE: {} s", end.as_secs_f64());
            let analyzer = HEPEventAnalyzer::new(eposFile.get_blocks());
            analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict_EPOS)
        },
        AcceptedTypes::PHQMD => {
            let criteria: Vec< &dyn ScalarCriteria<'_, _, _> > = vec![
                &StandardCriteria::FinEnergy,
                &StandardCriteria::ECharge,
                &StandardCriteria::BCharge,
                &StandardCriteria::LCharge,
                &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                &custom_criteria::MyExampleCriterias::StupidCriteria1
            ];
            let start = SystemTime::now();
            let phqmdFile = args.filenames.iter().fold(None, 
                |mut fo:Option<fmt::phqmd::PHQMDDataFile>, x| {
                    println!(">> FILE READING [{}]", x);
                    let f = File::open(&x).unwrap();
                    if let Some(mut fo) = fo {
                        fo.push_back(
                            fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap()
                        );
                        Some(fo)
                    } else {
                        Some(fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict_EPOS).unwrap())
                    }
                }
            ).unwrap();
            let end = start.elapsed().unwrap();
            println!("READING DONE: {} s", end.as_secs_f64());
            let analyzer = HEPEventAnalyzer::new(phqmdFile.get_blocks());
            analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, criteria, &dict_EPOS)
        },
    };
    let end = start.elapsed().unwrap();
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
    println!("TOTAL DONE: {} s", end.as_secs_f64());
    let mut f = File::create(args.o).unwrap();
    f.write((headers.join(";\t") + "\n").as_bytes()).unwrap();
    res.iter().for_each(|vals| {
        f.write((vals.iter().map(ToString::to_string).collect::<Vec<_>>().join(";\t") + "\n").as_bytes())
            .unwrap();
    });
}

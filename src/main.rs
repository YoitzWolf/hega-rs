mod anlz;
mod fmt;
mod custom_criteria;


use std::{
    collections::HashSet, f64::consts::PI, fs::File, io::{BufReader, Write}
};

use anlz::{DistributionCritetia, HEPEventAnalyzer, ScalarCriteria, StandardCriteria, StandardDistributionCriteria};
use fmt::{decoder::EposDict, generic::GenericDataContainer, oscar::OSCEposBlock, phqmd::PHQMDBlock};

use clap::{Parser, *};


const DEG_MIN: f64 = 0.0;
const DEG_MAX: f64 = PI + 0.1;
const DEG_CNT: usize = 360;

const NU_MIN: f64 = -15.0;
const NU_MAX: f64 = 15.0;
const NU_CNT: usize = 1000;



#[derive(
    clap::ValueEnum, Clone, Debug, Default
)]
pub enum AcceptedTypes {
    #[default]
    EPOS,
    UrQmdF19,
    PHQMD
}

#[derive(
    clap::ValueEnum, Clone, Debug, Default, PartialEq, Eq, Hash
)]
pub enum CalcTarget {
    #[default]
    Statistics,
    Distribution
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

    /// List of calculation targets
    #[clap(short, long, num_args = 1.., value_delimiter = ',', default_value="statistics")]
    target: Vec<CalcTarget>,

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
        AcceptedTypes::PHQMD | AcceptedTypes::UrQmdF19 => {
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
    let calc_target = args.target.iter().collect::<HashSet<_>>();
    println!(">>>> {:?}", calc_target);
    
    let start = SystemTime::now();
    
    let (scalar_results, distr_results) =  {
        match args.ftype {
            AcceptedTypes::EPOS => {
                let criteria: Vec< &dyn ScalarCriteria<'_, _, _> > = vec![
                    &StandardCriteria::FinEnergy,
                    &StandardCriteria::ECharge,
                    &StandardCriteria::BCharge,
                    &StandardCriteria::LCharge,
                    &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                    &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                    &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                ];
                let d_buf_criteria = vec![
                    StandardDistributionCriteria::new(
                        anlz::StandardDistributionCriteraDefiner::PdirTheta::<OSCEposBlock>,
                        DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string()
                    ),// as &dyn DistributionCritetia<'_, _, _>
                    StandardDistributionCriteria::new(
                        anlz::StandardDistributionCriteraDefiner::PNu::<OSCEposBlock>,
                        NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string()
                    )
                ];
                let d_criteria = d_buf_criteria.iter().map(
                    |x| {
                        x as  &dyn DistributionCritetia<'_, _, _>
                    }
                ).collect::<Vec<_>>();
                let start = SystemTime::now();
                let epos_file = args.filenames.iter().fold(None, 
                    |fo:Option<fmt::oscar::OSCEposDataFile>, x| {
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
                let analyzer = HEPEventAnalyzer::new(epos_file.get_blocks());
                
                let distr_res = if calc_target.contains(&CalcTarget::Distribution) {
                    analyzer.calculate_distribution_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, d_criteria, &dict_EPOS) 
                } else {Default::default()};
                let stat_res = if calc_target.contains(&CalcTarget::Statistics) {
                    analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict_EPOS)
                } else {Default::default()};

                (stat_res, distr_res)
            },
            AcceptedTypes::UrQmdF19 => {
                let criteria: Vec< &dyn ScalarCriteria<'_, _, _> > = vec![
                    &StandardCriteria::FinEnergy,
                    &StandardCriteria::ECharge,
                    &StandardCriteria::BCharge,
                    &StandardCriteria::LCharge,
                    &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                    &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                    &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                ];
                let d_buf_criteria = vec![
                    StandardDistributionCriteria::new(
                        anlz::StandardDistributionCriteraDefiner::PdirTheta::<OSCEposBlock>,
                        DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string()
                    ),// as &dyn DistributionCritetia<'_, _, _>
                    StandardDistributionCriteria::new(
                        anlz::StandardDistributionCriteraDefiner::PNu::<OSCEposBlock>,
                        NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string()
                    )
                ];

                let d_criteria = d_buf_criteria.iter().map(
                    |x| {
                        x as  &dyn DistributionCritetia<'_, _, _>
                    }
                ).collect::<Vec<_>>();
                let start = SystemTime::now();
                let urqmd_file = args.filenames.iter().fold(None, 
                    |fo:Option<fmt::oscar::OSC97UrQMDDataFile>, x| {
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
                let analyzer = HEPEventAnalyzer::new(urqmd_file.get_blocks());
                //analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict_EPOS)
                
                let distr_res = if calc_target.contains(&CalcTarget::Distribution) {
                    analyzer.calculate_distribution_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, d_criteria, &dict_EPOS) 
                } else {Default::default()};
                let stat_res = if calc_target.contains(&CalcTarget::Statistics) {
                    analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict_EPOS)
                } else {Default::default()};
                (stat_res, distr_res)
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
                ];
                let d_buf_criteria = vec![
                    StandardDistributionCriteria::new(
                        anlz::StandardDistributionCriteraDefiner::PdirTheta::<PHQMDBlock>,
                        DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string()
                    ),// as &dyn DistributionCritetia<'_, _, _>
                    StandardDistributionCriteria::new(
                        anlz::StandardDistributionCriteraDefiner::PNu::<PHQMDBlock>,
                        NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string()
                    )
                ];

                let d_criteria = d_buf_criteria.iter().map(
                    |x| {
                        x as  &dyn DistributionCritetia<'_, _, _>
                    }
                ).collect::<Vec<_>>();
                let start = SystemTime::now();
                let phqmd_file = args.filenames.iter().fold(None, 
                    |fo:Option<fmt::phqmd::PHQMDDataFile>, x| {
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
                let analyzer = HEPEventAnalyzer::new(phqmd_file.get_blocks());
                // analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, criteria, &dict_EPOS)
                let distr_res = if calc_target.contains(&CalcTarget::Distribution) {
                    analyzer.calculate_distribution_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, d_criteria, &dict_EPOS) 
                } else {Default::default()};
                let stat_res = if calc_target.contains(&CalcTarget::Statistics) {
                    analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, criteria, &dict_EPOS)
                } else {Default::default()};

                (stat_res, distr_res)
            },
        }
    };

    let end = start.elapsed().unwrap();

    // headers = "E[GeV];\tB;\tL\n".as_bytes()
    println!("TOTAL DONE: {} s", end.as_secs_f64());
    if calc_target.contains(&CalcTarget::Statistics) {
        let headers = scalar_results.headers();
        let res = scalar_results.values();
        let mut f = File::create(args.o.clone()).unwrap();
        f.write((headers.join(";\t") + "\n").as_bytes()).unwrap();
        res.iter().for_each(|vals| {
            f.write((vals.iter().map(ToString::to_string).collect::<Vec<_>>().join(";\t") + "\n").as_bytes())
                .unwrap();
        });
    }

    if calc_target.contains(&CalcTarget::Distribution) {
        let suff = args.o.clone();

        distr_results.iter().for_each(
            |((pref, size, bins, vals))| {
                let mut f = File::create(format!("{}-{}-{}", pref, size, suff)).unwrap();
                f.write(
                    format!(
                        "# distribution : {}; total-items={}\n lbin;\t rbin;\t value\n",
                        pref, size
                    ).as_bytes()
                ).unwrap();
                
                let s = bins.iter().zip(vals.iter()).map(
                    |((a, b), v)| {
                        format!("{};\t{};\t{}\n", a, b, v)
                    }
                ).reduce(
                    |x, y| {
                        x + &y
                    }
                ).unwrap();
                f.write(s.as_bytes()).unwrap();
            }
        );

        // f.write((headers.join(";\t") + "\n").as_bytes()).unwrap();
        // res.iter().for_each(|vals| {
        //     f.write((vals.iter().map(ToString::to_string).collect::<Vec<_>>().join(";\t") + "\n").as_bytes())
        //         .unwrap();
        // });
    }


}

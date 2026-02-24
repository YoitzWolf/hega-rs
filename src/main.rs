mod anlz;
mod fmt;
mod custom_criteria;
mod cli;
mod api_macro;

use api_macro::*;
use cli::*;
use std::{
    collections::HashSet, f64::consts::PI, fs::File, io::{BufReader, Write}
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use clap::{Parser, *};
use std::time::SystemTime;

use anlz::{fncs::lab_momentum, DistributionCritetia, HEPEventAnalyzer, ScalarCriteria, StandardCriteria, StandardDistributionCriteria};
use fmt::{decoder::EposDict, generic::GenericDataContainer, oscar::OSCEposBlock, phqmd::PHQMDBlock, qgsm::QGSMDataFile, hepmc::HepMCDataFile, hepmc::HepMCBlock};
use crate::{anlz::{HEPEvent, StandardDistributionCriteraDefiner}, fmt::{oscar::OSC97UrQMDDataFile, phqmd::PHQMDDataFile}};
use crate::anlz::ParticleListCompiler;
use crate::fmt::oscar::OSCEposDataFile;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEG_MIN: f64 = 0.0;
const DEG_MAX: f64 = PI + PI / 360.0;
const DEG_CNT: usize = 360;

const NU_MIN: f64 = -30.0;
const NU_MAX: f64 = 30.0;
const NU_CNT: usize = 2000;


fn main() {
    let args = cli::Args::parse();

    let dict = generate_dictionary(&args.ftype);

    // ANALYSER

    // println!("{:?}", args.filename);
    let calc_target = args.target.iter().collect::<HashSet<_>>();
    println!(">>>> {:?}", calc_target);
    
    let start = SystemTime::now();
    
    let (scalar_results, distr_results, list_resutls) =  {
        run_criteria_list!(
            &args,
            &dict,
            &calc_target,
            vec![
                &StandardCriteria::FinEnergy,
                &StandardCriteria::ECharge,
                &StandardCriteria::BCharge,
                &StandardCriteria::LCharge,
                &StandardCriteria::FinCnt,
                &StandardCriteria::FinChargedCnt,

                &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),

                &StandardCriteria::PseudorapidityFilterCnt(3.5, 5.8),
                &StandardCriteria::PseudorapidityFilterCnt(-5.8, -3.5),
                &StandardCriteria::PseudorapidityFilterCnt(4.4, 5.8),
                &StandardCriteria::PseudorapidityFilterCnt(-5.8, -4.4),
            ],
            vec![ ParticleListCompiler::new( {
                    let code = dict.get_particle_code("Proton").unwrap();
                    let mut set = HashSet::new();
                    set.insert(-code);
                    set
                }) ],
            [
                ( StandardDistributionCriteraDefiner::PdirTheta, DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string() ),
                ( StandardDistributionCriteraDefiner::PNu, NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string() ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [p])".to_string(), arg=( {
                    let code = dict.get_particle_code("Proton").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [~p])".to_string(), arg=( {
                    let code = dict.get_particle_code("Proton").unwrap();
                    vec![-code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [n])".to_string(), arg=( {
                    let code = dict.get_particle_code("Neutron").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [~n])".to_string(), arg=( {
                    let code = dict.get_particle_code("Neutron").unwrap();
                    vec![-code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [pi0])".to_string(), arg=( {
                    let code = dict.get_particle_code("pi0").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [pi+])".to_string(), arg=( {
                    let code = dict.get_particle_code("pi+").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [pi-])".to_string(), arg=( {
                    let code = dict.get_particle_code("pi-").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [K+])".to_string(), arg=( {
                    let code = dict.get_particle_code("K+").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [K-])".to_string(), arg=( {
                    let code = dict.get_particle_code("K-").unwrap();
                    vec![code]
                }, ) ),
                ( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [K0])".to_string(), arg=( {
                    let code = dict.get_particle_code("K0").unwrap();
                    vec![code]
                }, ) )
            ]
        )
    };

    let end = start.elapsed().unwrap();
    let sysprx = {
        //if args.lab {
        //    "Lab"
        //} else {
            ""
        //}
    }.to_string();
    // headers = "E[GeV];\tB;\tL\n".as_bytes()
    println!("TOTAL DONE: {} s", end.as_secs_f64());
    if calc_target.contains(&CalcTarget::Statistics) {
        let headers = scalar_results.headers();
        let res = scalar_results.values();
        let mut f = File::create(sysprx.clone() + &args.o.clone()).unwrap();
        f.write(
            format!(
                "# hega-rs ver.{} statistics: \n#{:?} in Lab: {}\n",
                VERSION,
                args.ftype,
                false // args.lab
            ).as_bytes()
        ).unwrap();
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
                let mut f = File::create(format!("{}{}-{}-{}", sysprx, pref, size, suff)).unwrap();
                f.write(
                    format!(
                        "# hega-rs ver.{} distribution : {}; total-items={}\n lbin;\t rbin;\t value\n#{:?} in Lab: {}\n",
                        VERSION,
                        pref, size,
                        args.ftype,
                        false // args.lab
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

    if calc_target.contains(&CalcTarget::ParticleList) {
        let suff = args.o.clone();
        list_resutls.iter().for_each(
            |list_res| {
                let pref = format!("Particles({:?})", {
                    let mut v = list_res.id_filter.iter().collect::<Vec<_>>();
                    v.sort();
                    v
                } );
                let mut f = File::create(format!("{}{}-{}", sysprx, pref, suff)).unwrap();
                f.write(
                    format!(
                        "# hega-rs ver.{} particle compilation : {}; total-items={}\n \t source\n#{:?} in Lab: {}\n",
                        VERSION,
                        pref, list_res.data.len(),
                        args.ftype,
                        false // args.lab
                    ).as_bytes()
                ).unwrap();
                f.write(
                    "id;\tmass;\tp;\tbeta;\n".as_bytes()
                ).unwrap();

                let s = list_res.data.iter().fold(
                    "".to_string(),
                    |x, d| {
                        x + &format!(
                            "{};\t{};\t{};\t{};\n", d.id, d.mass, d.p, d.beta
                        )
                    }
                );
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

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
use fmt::{decoder::EposDict, generic::GenericDataContainer, oscar::OSCEposBlock, phqmd::PHQMDBlock};
use crate::{anlz::{HEPEvent, StandardDistributionCriteraDefiner}, fmt::{oscar::OSC97UrQMDDataFile, phqmd::PHQMDDataFile}};
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
    
    let (scalar_results, distr_results) =  {
        run_criteria_list!(
            &args,
            &dict,
            &calc_target,
            vec![
                &StandardCriteria::FinEnergy,
                &StandardCriteria::ECharge,
                &StandardCriteria::BCharge,
                &StandardCriteria::LCharge,
                &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                &StandardCriteria::PseudorapidityFilterCnt(3.5, 5.8),
                &StandardCriteria::PseudorapidityFilterCnt(-5.8, 3.5),
                &StandardCriteria::PseudorapidityFilterCnt(4.4, 5.8),
                &StandardCriteria::PseudorapidityFilterCnt(-5.8, 4.4),
            ],
            ( StandardDistributionCriteraDefiner::PdirTheta, DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string() ),
            ( StandardDistributionCriteraDefiner::PNu, NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string() )
            //( StandardDistributionCriteraDefiner::PNu_selected, NU_MIN, NU_MAX, NU_CNT, "N(Nu, [p, ~p])".to_string(), arg=( {
            //    let code = dict.get_particle_code("Proton").unwrap();
            //    vec![code, -code]
            //}, ) )
            //( StandardDistributionCriteraDefiner::PdirTheta, DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string() ),
        )
        
        /*match args.ftype {
            AcceptedTypes::EPOS => {
                run_criteria_list_inner!(
                    { &args },
                    { &calc_target },
                    vec![
                        &StandardCriteria::FinEnergy,
                        &StandardCriteria::ECharge,
                        &StandardCriteria::BCharge,
                        &StandardCriteria::LCharge,
                        &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                        &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                        &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                    ],
                    {
                        vec![
                            /*StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PdirTheta::<
                                    <OSCEposDataFile<'_> as GenericDataContainer>::Block>,
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string()
                            ),// as &dyn DistributionCritetia<'_, _, _>
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu::<
                                    <OSCEposDataFile<'_> as GenericDataContainer>::Block>,
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string()
                            )*/
                        ]
                    },
                    &dict,
                    OSCEposDataFile<'_>
                )
            },
            AcceptedTypes::UrQmdF19 => {
                run_criteria_list_inner!(
                    { &args },
                    { &calc_target },
                    vec![
                        &StandardCriteria::FinEnergy,
                        &StandardCriteria::ECharge,
                        &StandardCriteria::BCharge,
                        &StandardCriteria::LCharge,
                        &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                        &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                        &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                    ],
                    {
                        vec![
                            /*StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PdirTheta::<
                                    <OSC97UrQMDDataFile<'_> as GenericDataContainer>::Block>,
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string()
                            ),// as &dyn DistributionCritetia<'_, _, _>
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu::<
                                    <OSC97UrQMDDataFile<'_> as GenericDataContainer>::Block>,
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string()
                            )*/
                        ]
                    },
                    &dict,
                    OSC97UrQMDDataFile<'_>
                )
                /*
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
                                        fmt::oscar::OSC97UrQMDDataFile::upload(BufReader::new(f), &dict).unwrap()
                                    );
                                    Some(fo)            
                                } else {
                                    Some(fmt::oscar::OSC97UrQMDDataFile::upload(BufReader::new(f), &dict).unwrap())
                                }
                            }
                        ).unwrap();
                        let end = start.elapsed().unwrap();
                        println!("READING DONE: {} s", end.as_secs_f64());

                        let events = {
                            let mut events = urqmd_file.borrow_blocks();
                            if args.lab {
                                events.par_iter_mut().for_each(
                                    |x|{
                                        x.event.iter_mut().for_each(
                                            |p| {
                                                let mp = lab_momentum(p, &dict);
                                                p.p = mp;
                                            }
                                        );
                                    }
                                );
                            }
                            events
                        };
                        let analyzer = HEPEventAnalyzer::new(&events);
                        // analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict)
                
                        let distr_res = if calc_target.contains(&CalcTarget::Distribution) {
                            analyzer.calculate_distribution_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, d_criteria, &dict) 
                        } else {Default::default()};
                        let stat_res = if calc_target.contains(&CalcTarget::Statistics) {
                            analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<OSCEposBlock>, criteria, &dict)
                        } else {Default::default()};
                        (stat_res, distr_res)
                     */
            },
            AcceptedTypes::PHQMD => {
                run_criteria_list_inner!(
                    { &args },
                    { &calc_target },
                    vec![
                        &StandardCriteria::FinEnergy,
                        &StandardCriteria::ECharge,
                        &StandardCriteria::BCharge,
                        &StandardCriteria::LCharge,
                        &StandardCriteria::PseudorapidityFilterCnt(-0.5, 0.5),
                        &StandardCriteria::PseudorapidityFilterCnt(-1.0, 1.0),
                        &StandardCriteria::PseudorapidityFilterCnt(-1.5, 1.5),
                    ],
                    {
                        vec![
                            /*StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PdirTheta::<
                                    <PHQMDDataFile<'_> as GenericDataContainer>::Block>,
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p)".to_string()
                            ),// as &dyn DistributionCritetia<'_, _, _>
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu::<
                                    <PHQMDDataFile<'_> as GenericDataContainer>::Block>,
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu)".to_string()
                            )*/
                        ]
                    },
                    &dict,
                    PHQMDDataFile<'_>
                )
                /*
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
                            ),
                            // ---------------------------------------
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu_selected::<PHQMDBlock>(vec![2212, -2212]),
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu, [p, ~p])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu_selected::<PHQMDBlock>(vec![211, -211]),
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu, [pi+, pi-])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu_selected::<PHQMDBlock>(vec![111, -111]),
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu, [pi0, ~pi0])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu_selected::<PHQMDBlock>(vec![11, -11]),
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu, [e, ~e])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu_selected::<PHQMDBlock>(vec![13, -13]),
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu, [mu, ~mu])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PNu_selected::<PHQMDBlock>(vec![2112, -2112]),
                                NU_MIN, NU_MAX, NU_CNT, "N(Nu, [n, ~n])".to_string()
                            ),
                            // ---------------------------------------
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PTheta_selected::<PHQMDBlock>(vec![2212, -2212]),
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p), [p, ~p])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PTheta_selected::<PHQMDBlock>(vec![211, -211]),
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p), [pi+, pi-])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PTheta_selected::<PHQMDBlock>(vec![111, -111]),
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p), [pi0, ~pi0])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PTheta_selected::<PHQMDBlock>(vec![11, -11]),
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p), [e, ~e])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PTheta_selected::<PHQMDBlock>(vec![13, -13]),
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p), [mu, ~mu])".to_string()
                            ),
                            StandardDistributionCriteria::new(
                                anlz::StandardDistributionCriteraDefiner::PTheta_selected::<PHQMDBlock>(vec![2112, -2112]),
                                DEG_MIN, DEG_MAX, DEG_CNT, "N(Theta_p), [n, ~n])".to_string()
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
                                        fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict).unwrap()
                                    );
                                    Some(fo)
                                } else {
                                    Some(fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict).unwrap())
                                }
                            }
                        ).unwrap();
                        let end = start.elapsed().unwrap();
                        println!("READING DONE: {} s", end.as_secs_f64());
                        let events = {
                            let mut events = phqmd_file.borrow_blocks();
                            if args.lab {
                                events.par_iter_mut().for_each(
                                    |x|{
                                        x.event.iter_mut().for_each(
                                            |p| {
                                                let mp = lab_momentum(p, &dict);
                                                p.p = mp;
                                            }
                                        );
                                    }
                                );
                            }
                            events
                        };
                        let analyzer = HEPEventAnalyzer::new(&events);
                        // analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, criteria, &dict)
                        let distr_res = if calc_target.contains(&CalcTarget::Distribution) {
                            analyzer.calculate_distribution_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, d_criteria, &dict) 
                        } else {Default::default()};
                        let stat_res = if calc_target.contains(&CalcTarget::Statistics) {
                            analyzer.calculate_criteria(anlz::IS_FINAL_FILTER::<PHQMDBlock>, criteria, &dict)
                        } else {Default::default()};

                        (stat_res, distr_res)
                         */
            },
            //AcceptedTypes::ROOT => todo!(),
        }*/
    };

    let end = start.elapsed().unwrap();


    let sysprx = {
        if args.lab {
            "Lab"
        } else {
            ""
        }
    }.to_string();

    // headers = "E[GeV];\tB;\tL\n".as_bytes()
    println!("TOTAL DONE: {} s", end.as_secs_f64());
    if calc_target.contains(&CalcTarget::Statistics) {
        let headers = scalar_results.headers();
        let res = scalar_results.values();
        let mut f = File::create(sysprx.clone() + &args.o.clone()).unwrap();
        f.write(
            format!(
                "# hega-rs ver.{} statistics: ",
                VERSION
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
                        "# hega-rs ver.{} distribution : {}; total-items={}\n lbin;\t rbin;\t value\n",
                        VERSION,
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

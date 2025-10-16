use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::time::SystemTime;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::cli::{self, Args, CalcTarget};
use crate::cli::AcceptedTypes;
use crate::fmt;
use crate::{standard_criteria, anlz::{fncs::lab_momentum, DistributionCritetia, HEPEventAnalyzer, ScalarCriteria, StandardCriteria, StandardDistributionCriteria}};
use crate::fmt::oscar::OSCEposDataFile;
use crate::fmt::{
    decoder::EposDict,
    generic::GenericDataContainer,
    oscar::{OSCEposBlock, OscarParticle},
    phqmd::{PHQMDBlock, PHQMDParticle}
};
use crate::anlz::HEPEvent;

use crate::fmt::{oscar::OSC97UrQMDDataFile, phqmd::PHQMDDataFile};
use crate::anlz::generic::*;


pub fn get_decoder(at: &cli::AcceptedTypes) -> fmt::decoder::DctCoding {
    match at {
        cli::AcceptedTypes::EPOS => {
            fmt::decoder::DctCoding::EPOS
        },
        cli::AcceptedTypes::PHQMD | cli::AcceptedTypes::UrQmdF19 => {
            fmt::decoder::DctCoding::PDG
        },
    }
}

pub fn generate_dictionary(x: &AcceptedTypes) -> EposDict {
    let decoder = get_decoder(x);
    let dict_lepto = EposDict::upload(
            BufReader::new(File::open("./dicts/EPOS_LEPTONS.particles.txt").unwrap()),
            decoder.clone(),
            None,
    );
    let dict = EposDict::upload(
        BufReader::new(File::open("./dicts/EPOS.particles.txt").unwrap()),
        decoder,
        Some(dict_lepto.codes().cloned().collect())
    );
    drop(dict_lepto);
    dict
}

#[macro_export]
macro_rules! run_criteria_list_inner {
    ($args:expr, $calc_target:expr, $criteria_vec:expr, $d_buf_criteria:expr, $dict:expr, $DataFile:ty) => {
        {
            let criteria: Vec< &dyn ScalarCriteria<'_, _, _> > = $criteria_vec;
            let start = SystemTime::now();
            let files = $args.filenames.iter().fold(None,
                |fo:Option<$DataFile>, x| {
                    println!(">> FILE READING [{}]", x);
                    let f = File::open(&x).unwrap();
                    let x = <$DataFile>::upload(BufReader::new(f), $dict).unwrap();
                    if let Some(mut fo) = fo {
                        fo.push_back( x );
                        Some(fo)            
                    } else {
                        Some( x )
                    }
                }
            ).unwrap();
            let distribution_critera_buf = $d_buf_criteria;
            let d_criteria: Vec<&_> = distribution_critera_buf.iter().to_owned().map(
                |x| {
                    x as &dyn DistributionCritetia<
                            '_, _, _ //<<$DataFile as GenericDataContainer>::Block as HEPEvent>::P
                        >
                }
            ).collect::<Vec<_>>();
            let end = start.elapsed().unwrap();
            println!("READING DONE: {} s", end.as_secs_f64());
            let events = {
                let mut events = files.borrow_blocks();
                if $args.lab {
                    events.par_iter_mut().for_each(
                        |x|{
                            x.event.iter_mut().for_each(
                                |p| {
                                    let mp = lab_momentum(p, $dict);
                                    p.p = mp;
                                }
                            );
                        }
                    );
                }
                events
            };
            let analyzer = HEPEventAnalyzer::new(&events);
            let distr_res = if $calc_target.contains(&CalcTarget::Distribution) {
                analyzer.calculate_distribution_criteria(crate::anlz::IS_FINAL_FILTER::<<$DataFile as GenericDataContainer>::Block>, d_criteria, $dict) 
            } else {Default::default()};
            let stat_res = if $calc_target.contains(&CalcTarget::Statistics) {
                analyzer.calculate_criteria(crate::anlz::IS_FINAL_FILTER::<<$DataFile as GenericDataContainer>::Block>, criteria, $dict)
            } else {Default::default()};

            (stat_res, distr_res)
        }
    };
}

#[macro_export]
macro_rules! run_criteria_list {
    (
        $args:expr,
        $dict:expr,
        $calc_target:expr,
        $criteria_vec:expr,
        $( 
            ($Definer: ident::$DefinerVeriant: ident, $DEG_MIN:expr, $DEG_MAX:expr, $DEG_CNT:expr, $NAME:expr $(, arg=($( $ARG:expr, )*) )? )
        ),*
    ) => {
        
        match $args.ftype {
            AcceptedTypes::EPOS => {
                run_criteria_list_inner!(
                    { $args },
                    { $calc_target },
                    { $criteria_vec },
                    {
                        vec!(
                            $(
                                #[allow(unused_assignments)]
                                {
                                    standard_criteria!(
                                        $Definer::$DefinerVeriant,
                                        OSCEposDataFile<'_>,
                                        $DEG_MIN, $DEG_MAX, $DEG_CNT, $NAME $(, arg=$($ARG ,)* )?
                                    )
                                }
                            ),* ,
                        )
                    },
                    $dict,
                    OSCEposDataFile<'_>
                )
            },
            AcceptedTypes::UrQmdF19 => {
                run_criteria_list_inner!(
                    { $args },
                    { $calc_target },
                    { $criteria_vec },
                    {
                        vec!(
                            $(
                                #[allow(unused_assignments)]
                                {
                                    standard_criteria!(
                                        $Definer::$DefinerVeriant,
                                        OSC97UrQMDDataFile<'_>,
                                        $DEG_MIN, $DEG_MAX, $DEG_CNT, $NAME $(, arg=$($ARG, )* )?
                                    )
                                }
                            ),* ,
                        )
                    },
                    $dict,
                    OSC97UrQMDDataFile<'_>
                )
            },
            AcceptedTypes::PHQMD => {
                run_criteria_list_inner!(
                    { $args },
                    { $calc_target },
                    { $criteria_vec },
                    {
                        vec!(
                            $(
                                #[allow(unused_assignments)]
                                {
                                    standard_criteria!(
                                        $Definer::$DefinerVeriant,
                                        PHQMDDataFile<'_>,
                                        $DEG_MIN, $DEG_MAX, $DEG_CNT, $NAME $(, arg=$($ARG, )* )?
                                    )
                                }
                            ),* ,
                        )
                    },
                    $dict,
                    PHQMDDataFile<'_>
                )
            },
        }
        // */
    }
}

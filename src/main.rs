use std::{fs::File, io::{BufReader, Write}};

use fmt::{decoder::EposDict, generic::GenericDataContainer};

use rayon::prelude::*;

mod fmt;
// mod anlz;


fn main() {
    let dict = EposDict::upload(BufReader::new(File::open("./dicts/EPOS.particles.txt").unwrap()), fmt::decoder::DctCoding::PDG);
    let dictBaryo = EposDict::upload(BufReader::new(File::open("./dicts/EPOS_BARYONS.particles.txt").unwrap()), fmt::decoder::DctCoding::PDG);
    let dictLepto = EposDict::upload(BufReader::new(File::open("./dicts/EPOS_LEPTONS.particles.txt").unwrap()), fmt::decoder::DctCoding::PDG);

    // let f = File::open("C:\\Users\\yoitz\\Documents\\Reports\\s4Check\\sim\\eposSimBig\\eposSim\\epos_3\\results\\z-pp7000.data").unwrap();
    // let oscFile = fmt::oscar::OSCEposDataFile::upload(BufReader::new(f), &dict).unwrap();

    let f = File::open("C:\\Users\\yoitz\\PHQMD\\pp7tdata\\phsd.dat").unwrap();
    let phqmdFile = fmt::phqmd::PHQMDDataFile::upload(BufReader::new(f), &dict).unwrap();

    use std::time::SystemTime;

    let start = SystemTime::now();
    let res = phqmdFile.get_blocks().par_iter().map(
        |event| {
            event.event.iter().fold(
                (0., 0., 0), |(mut E, mut B, mut L), p| {
                    {
                        // LAST GEN
                        //ENERGY
                        E += p.E;
                        //BARYON CHARGE
                        if let Some(pd) = dict.get(&p.code) {
                            B += [  pd.ifl1.unwrap_or(0),
                                    pd.ifl2.unwrap_or(0),
                                    pd.ifl3.unwrap_or(0)].iter().map(
                                        |&x| { if(x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                                    ).sum::<f64>() / 3.0;
                        } else if let Some(apd) = dict.get(& -(p.code)) {
                            B -= [ apd.ifl1.unwrap_or(0),
                            apd.ifl2.unwrap_or(0),
                            apd.ifl3.unwrap_or(0)].iter().map(
                                |&x| { if(x > 0) { 1. } else if (x < 0) {-1.} else {0.} }
                            ).sum::<f64>() / 3.0;
                            
                        }
                        // LEPTON CHARGE
                        if dictLepto.get(&p.code).is_some() {
                            L += 1;
                        } else if dictLepto.get(& -(p.code)).is_some() {
                            L -= 1;
                        }
                    }
                    (E, B, L)
                })
        }
    ).collect::<Vec<_>>();
    let end = start.elapsed().unwrap();
    
    println!("CALCULATION DONE: {}millis", end.as_millis());

    let mut f = File::create("./phsd.csv.statistics").unwrap();
    f.write("E[GeV];\tB;\tL\n".as_bytes()).unwrap();
    res.iter().for_each(
        |(x, y, z)| {
            f.write(
                format!("{};\t{};\t{}\n", x, y, z).to_string().as_bytes()
            ).unwrap();
        }
    );

}

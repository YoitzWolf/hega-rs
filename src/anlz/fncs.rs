/*

    
def pseudorapidity(v1, v2):
    theta = np.arccos(v1.scal(v2) / (v1.len() * v2.len()))
    return -np.log(np.tan(theta/2))

*/

use std::{fmt, fs::File, io::BufReader};

use crate::fmt::{decoder::{DctCoding, EposDict}, oscar::{OSCEposHeader, OscarParticle}};

use super::Particle;

/// z-pseudorapidity
pub fn pseudorapidity((x, y, z): &(f64, f64, f64)) -> f64 {
    let theta = ( (z) / ((x*x+y*y+z*z).sqrt()) ).acos();
    - (theta / 2.0).tan().ln()
}


pub fn beta<P: Particle>(p: &P, dec: &P::Decoder ) -> f64 {
    p.momentum_energy(dec) / p.energy(dec)
}

pub fn gamma<P: Particle>(p: &P, dec: &P::Decoder ) -> f64 {
    p.energy(dec) / p.mass_energy(dec)
}


pub fn lab_momentum<P: Particle>(p: &P, dec: &P::Decoder ) -> (f64, f64, f64) {
    let &(x, y ,z) = p.momentum(dec);
    // let pen = p.momentum_energy(dec);
    // let e = p.energy(dec);
    // let m = p.mass_energy(dec);
    let gamma = gamma(p, dec);
    let beta = beta(p, dec);
    let z = gamma * (
        z + beta*p.energy(dec) // pen*gamma
    );
    (x, y, z)
}

pub fn rapidity<P: Particle>(p: &P, dec: &P::Decoder ) -> f64 {
    beta(p, dec).atanh()
}


#[test]
fn test_lab_mom() {

    let dict_lepto = EposDict::upload(
        BufReader::new(File::open("./dicts/EPOS_LEPTONS.particles.txt").unwrap()),
        DctCoding::EPOS,
        None,
    );
    let dict_EPOS = EposDict::upload(
        BufReader::new(File::open("./dicts/EPOS.particles.txt").unwrap()),
        DctCoding::EPOS,
        Some(dict_lepto.codes().cloned().collect())
    );
    drop(dict_lepto);

    let pu0 = 199.239f64;

    let p0 = (0.0f64, 0.0f64, -pu0);
    let E0 = (p0.0.powi(2) + p0.1.powi(2) + p0.2.powi(2) + 172.592f64.powi(2)).sqrt();
    let p = OscarParticle {
        id: 0,
        code: 1120,
        state: 0,
        p: p0,
        p0: pu0,
        mass: 172.592,
        coords: (0. , 0. , 0.),
        time: 0.,
    };
    let rap0 = rapidity(&p, &dict_EPOS);

    // let t0 = p.energy(&dict_EPOS).powi(2) + p.

    let np = lab_momentum(&p, &dict_EPOS);
    println!("{:?}", np);
    let T1 = (np.0.powi(2) + np.1.powi(2) + np.2.powi(2)).sqrt();
    let p2 = OscarParticle {
        id: 0,
        code: 1120,
        state: 0,
        p: np,
        p0: T1,
        mass: 172.592,
        coords: (0. , 0. , 0.),
        time: 0.,
    };

    let E1 = p2.energy(&dict_EPOS);
    let rap1 = rapidity(&p2, &dict_EPOS);

    println!("E {}, {}", E0, E1);
    println!("Rapidity {}, {}", rap0, rap1);
    println!("Beta {}, {}",  beta(&p, &dict_EPOS), beta(&p2, &dict_EPOS));
    println!("Gamma {}, {}", gamma(&p, &dict_EPOS), gamma(&p2, &dict_EPOS));
    

}
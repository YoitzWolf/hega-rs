/*

    
def pseudorapidity(v1, v2):
    theta = np.arccos(v1.scal(v2) / (v1.len() * v2.len()))
    return -np.log(np.tan(theta/2))

*/

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
    let &mp = p.momentum(dec);
    let pen = p.momentum_energy(dec);
    let e = p.energy(dec);
    let m = p.mass_energy(dec);
    let gamma = e / m;
    // let beta = pen / e;
    let z = gamma * ( mp.2 + pen*gamma );
    (mp.0, mp.1, z)
}
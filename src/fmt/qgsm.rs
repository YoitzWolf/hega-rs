use std::error::Error;
use std::io::prelude::*;
use with_position::{WithPosition, Position};

use super::decoder::EposDict;
/// OSCAR1999 format reader and interpreter

use super::generic::*;

#[derive(Debug)]
pub struct QGSMBlock {
    /// Block Header
    pub header: QGSMBlockHeader,
    /// Particles
    pub event: Vec<QGSMParticle>
}

#[derive(Debug)]
pub struct QGSMBlockHeader {
    pub event_id: usize,
    /// Count of particles in OSCAR event description
    pub nout: usize,
    /// impact parameter
    pub b: f64,
    pub bx: f64,
    pub by: f64,
    
}

#[derive(Debug)]
pub struct QGSMHeader {
    /// Collision Energy
    pub snn: f64,
    /// OSCAR event signature. 
    /// (1, 1) + (1, 1) or Au(..) + Au(..)
    pub event_signature: String,
}

#[derive(Debug)]
pub struct QGSMParticle {
    pub charge: i64,
    pub lepton_number: i64,
    pub strangeness: i64,
    pub baryon_number: i64,
    pub code: i32,
    pub p: (f64, f64, f64),
    pub p_lab_z: f64,
    pub mass: f64,
}

impl<'a> TryFrom<(QGSMBlockHeader, &'a Vec<String>)> for QGSMBlock {
    type Error = Box<dyn Error>;
    fn try_from(value: (QGSMBlockHeader, &'a Vec<String>)) -> Result<Self, Self::Error> {
        let particles = value.1.iter().try_fold(
            vec![],
            |mut v, line| -> Result<Vec<_>, Box<dyn Error>> {
                let mut iter = line.split_ascii_whitespace();
                let mut args = [""; 11];
                [0usize, 1, 2, 3, 4, 5 ,6 ,7 ,8 ,9 ,10].iter()
                    .try_fold(&mut args, |a, x| {
                        let v = iter.next();
                        a[*x] = v?;
                        Some(a)
                });
                v.push(
                    QGSMParticle::try_from( args )?
                );
                Ok(v)
            }
        )?;
        Ok(
            Self {
                header: value.0,
                event: particles,
            }
        )
    }
}

impl<'a> TryFrom<[&'a str; 11]> for QGSMParticle {
    type Error = Box<dyn Error>;

    fn try_from(value: [&'a str; 11]) -> Result<Self, Self::Error> {
        let s = Self {
            charge: value[0].parse()?,
            lepton_number: value[1].parse()?,
            strangeness: value[2].parse()?,
            baryon_number: value[3].parse()?,
            code: value[4].parse()?,
            p: (value[5].parse()?, value[6].parse()?, value[7].parse()?),
            p_lab_z: value[8].parse()?,
            // velue[9] is unknown in QGSM data, //TODO
            mass: value[10].parse()?
        };
        Ok(s)
    }
}

impl<'a> DataBlock<'a, QGSMBlockHeader> for QGSMBlock {
    fn get_header(&self) -> &QGSMBlockHeader {
        &self.header
    }
}


#[derive(Debug)]
pub struct QGSMDataFile<'a> {
    header: QGSMHeader,
    events: Vec<QGSMBlock>,
    pub decoder: &'a EposDict
}


impl<'a, 'b> GenericDataContainer<'a, 'b> for QGSMDataFile<'b> {
    type Header = QGSMHeader;

    type BlockHeader = QGSMBlockHeader;

    type Block = QGSMBlock;

    type Decoder = EposDict;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_blocks(&self) -> &Vec<Self::Block> {
        &self.events
    }

    fn borrow_blocks(self)  -> Vec<Self::Block> {
        self.events
    }

    fn upload<T: Sized + std::io::Read>(data: std::io::BufReader<T>, decoder: &'b Self::Decoder) -> Result<Self, std::io::Error> {
        match data.lines().with_position().enumerate().try_fold(
            (
                None,
                None,
                Vec::<String>::new(),
                vec![]
            ),
            |
                (mut header, mut bufheader, mut buf, mut events)
                , (idx, (position, _line))
            | -> Result<_, Box<dyn Error> > {
                match _line {
                    Ok(line) => {
                        if idx <= 3 && line.contains("Results of QGSM") {
                            // COMMENT
                            let cmt = line.trim();
                            // let s: Vec<_> = line.split_ascii_whitespace().map(|x| x.trim()).filter(|x| x.len()>0).collect();
                            header = Some (
                                    Self::Header {
                                        snn: 0.0,
                                        event_signature: cmt.to_owned(),
                                    }
                            );
                            Ok((header, bufheader, buf, events))
                            // END COMMENT
                        } else if idx <= 3 && line.contains("sqrt(s)=") {
                            let s: f64 = line.split_once("sqrt(s)=").unwrap().1.split_once(")").unwrap().0.trim().parse()?;
                            header.as_mut().unwrap().snn = s;
                            Ok((header, bufheader, buf, events))
                        } else if idx > 3 {
                            // DATA OR EMPTY
                            let tr = line.trim();
                            let tokens: Vec<_> = tr.split_ascii_whitespace().filter(|x| x.len() > 0).collect();
                            match tokens.len() {
                                5 => {
                                    
                                    // new event
                                    if let Some(hd) = bufheader {
                                        let block = Self::Block::try_from(
                                            (hd, &buf)
                                        )?;
                                        events.push(block);
                                        buf.clear();
                                    }
                                    bufheader = Some(Self::BlockHeader {                                        
                                        event_id: tokens[0].parse()?,
                                        nout: tokens[1].parse()?,
                                        b: tokens[2].parse()?,
                                        bx: tokens[3].parse()?,
                                        by: tokens[4].parse()?,
                                    });
                                    
                                },
                                (6..) => {
                                    // line event
                                    buf.push({
                                        tokens.join(" ")
                                    });
                                    if let Position::Last = position {
                                        if let Some(hd) = bufheader {
                                            let block = Self::Block::try_from(
                                                (hd, &buf)
                                            )?;
                                            events.push(block);
                                            buf.clear();
                                        }
                                        bufheader = None;
                                    }
                                },
                                _ => {
                                    println!("To little symbols in line or empty, skipped..");
                                }
                            }
                            Ok((header, bufheader, buf, events))
                        } else {Ok((header, bufheader, buf, events))}
                    },
                    Err(e) => {
                        Err(Box::new(e))
                    },
                }
            }
        ) {
            Ok((a, b ,c, events)) => {
                Ok(
                    Self {
                        header: a.unwrap(),
                        events: events,
                        decoder: decoder
                    }
                )
            },
            Err(e) => {
                Err(std::io::Error::new::<String>(std::io::ErrorKind::InvalidData, e.as_ref().to_string().into()))
            },
        }
    }

    fn push_back(&mut self, mut t: Self) {
        self.events.append(&mut t.events);
    }
}

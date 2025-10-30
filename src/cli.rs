use clap::{Parser, *};


#[derive(
    clap::ValueEnum, Clone, Debug, Default
)]
pub enum AcceptedTypes {
    #[default]
    EPOS,
    UrQmdF19,
    PHQMD,
    //ROOT,
}

#[derive(
    clap::ValueEnum, Clone, Debug, Default, PartialEq, Eq, Hash
)]
pub enum CalcTarget {
    #[default]
    Statistics,
    Distribution,
}

#[derive(
    clap::ValueEnum, Clone, Debug, Default, PartialEq, Eq, Hash
)]
pub enum CalcMode {
    /// final result checkout
    #[default]
    Default,
    /// calculate checkout to each timestep.
    /// 
    /// will cause error, if tryed to used with 
    /// non-history format (ROOT).
    /// 
    /// timesteps calculated as times, where no changes occured, i.e. 
    /// no decays, no collision, borning or annigilating particles.
    InTime
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
pub struct Args {
    /// Type of file
    pub ftype: AcceptedTypes,

    /// Take if need to check in Lab system [change Pz momentum]
    #[clap(long)]
    pub lab: bool,

    /// List of calculation targets
    #[clap(short, long, num_args = 1.., value_delimiter = ',', default_value="statistics")]
    pub target: Vec<CalcTarget>,

    #[clap(short, long, default_value="default")]
    pub mode: CalcMode,

    /// List of files, delimeter ','. Use "quotes" if path contains whitespaces
    #[clap(short, long, num_args = 1.., value_delimiter = ',')]
    pub filenames: Vec<String>,

    #[clap(short, long="output", default_value="results.csv.stat")]
    pub o: String

}
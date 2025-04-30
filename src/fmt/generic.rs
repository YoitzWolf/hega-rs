use std::{fs::File, io::BufReader};



pub trait DataBlock<'a, Header>: TryFrom<(Header, &'a Vec<String>)> {
    fn get_header(&self) -> &Header;
}

pub trait GenericDataContainer<'a, 'b>: Sized {
    type Header;
    type BlockHeader;
    type Block: DataBlock<'a, Self::BlockHeader>;
    type Decoder;
    fn get_header(&self) -> &Self::Header;

    fn get_blocks(&self) -> &Vec<Self::Block>;

    fn borrow_blocks(self)  -> Vec<Self::Block>;

    fn upload<T: Sized + std::io::Read>(data: BufReader<T>, decoder: &'b Self::Decoder) -> Result<Self, std::io::Error>;

    fn push_back(&mut self, t: Self);

}
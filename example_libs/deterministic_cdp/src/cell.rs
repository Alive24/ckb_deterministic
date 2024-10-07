use ckb_deterministic::recipes::MoleculeCellData;
use crate::data::PoolData;
pub struct Pool;

impl MoleculeCellData for Pool {
    type MoleculeCellDataType = PoolData;
}



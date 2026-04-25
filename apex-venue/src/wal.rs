use crate::{OrderedOp, State};
use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter, Read};
use crc32fast::Hasher;
use sha2::{Sha256, Digest};

pub struct WalManager {
        file: BufWriter<File>,
            latest_root: [u8; 32],
                path: String,
}

impl WalManager {
        pub fn open(path: &str) -> std::io::Result<Self> {
                    let file = OpenOptions::new().create(true).append(true).open(path)?;
                            Ok(Self { file: BufWriter::new(file), latest_root: [0; 32], path: path.to_string() })
        }
            pub fn commit_batch(&mut self, ops: &[OrderedOp], _state: &State) -> std::io::Result<[u8; 32]> {
                        for op in ops {
                                        let payload = bincode::serialize(op).unwrap();
                                                    let mut hasher = Hasher::new();
                                                                hasher.update(&payload);
                                                                            let crc = hasher.finalize();
                                                                                        self.file.write_all(&(payload.len() as u32).to_le_bytes())?;
                                                                                                    self.file.write_all(&payload)?;
                                                                                                                self.file.write_all(&crc.to_le_bytes())?;
                        }
                                self.file.write_all(&0xDEADBEEF_u32.to_le_bytes())?;
                                        self.file.flush()?;
                                                self.file.get_ref().sync_all()?;
                                                        let mut root_hasher = Sha256::new();
                                                                for op in ops { root_hasher.update(&op.causal_id); }
                                                                        let new_root: [u8; 32] = root_hasher.finalize().into();
                                                                                self.latest_root = new_root;
                                                                                        Ok(new_root)
            }
                pub fn recover(&mut self) -> std::io::Result<Vec<OrderedOp>> {
                            let mut file = File::open(&self.path)?;
                                    let mut ops = Vec::new();
                                            let mut len_buf = [0u8; 4];
                                                    loop {
                                                                    if file.read_exact(&mut len_buf).is_err() { break; }
                                                                                let len = u32::from_le_bytes(len_buf) as usize;
                                                                                            let mut payload = vec![0u8; len];
                                                                                                        if file.read_exact(&mut payload).is_err() { break; }
                                                                                                                    let mut crc_buf = [0u8; 4];
                                                                                                                                if file.read_exact(&mut crc_buf).is_err() { break; }
                                                                                                                                                    let stored_crc = u32::from_le_bytes(crc_buf);
                                                                                                                                                        let mut hasher = Hasher::new();
                                                                                                                                                                    hasher.update(&payload);
                                                                                                                                                                                if hasher.finalize() != stored_crc { break; }
                                                                                                                                                                                            if let Ok(op) = bincode::deserialize::<OrderedOp>(&payload) { ops.push(op); }
                                                                                                                                                                                                        let mut barrier = [0u8; 4];
                                                                                                                                                                                                                    if file.read_exact(&mut barrier).is_err() { break; }
                                                                                                                                                                                                                                if u32::from_le_bytes(barrier) != 0xDEADBEEF { break; }
                                                    }
                                                            Ok(ops)
                }
                    pub fn set_latest_root(&mut self, root: [u8; 32]) { self.latest_root = root; }
                        pub fn latest_root(&self) -> &[u8; 32] { &self.latest_root }
                            pub fn sync_all(&mut self) -> std::io::Result<()> { self.file.get_ref().sync_all() }
}

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use blake2::{Blake2s256, Digest};
use rapidhash::RapidHashMap;

use crate::utils::{ReadMappedFile, get_plugin_directory};

const RECORD_SIZE: u64 = 48;
const TOMBSTONE: u64 = u64::MAX;
const ZSTD_LEVEL: i32 = 12;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub(crate) struct Code {
    pub(in crate::plugin) blake: [u8; 32],
    pub(in crate::plugin) pos: u64,
    pub(in crate::plugin) len: u64,
}

impl Code {
    fn to_bytes(self) -> [u8; RECORD_SIZE as usize] {
        let mut buf = [0u8; RECORD_SIZE as usize];

        buf[0..32].copy_from_slice(&self.blake);
        buf[32..40].copy_from_slice(&self.pos.to_le_bytes());
        buf[40..48].copy_from_slice(&self.len.to_le_bytes());

        buf
    }

    fn from_bytes(buf: &[u8]) -> Self {
        let mut blake = [0u8; 32];
        blake.copy_from_slice(&buf[0..32]);
        let pos = u64::from_le_bytes(buf[32..40].try_into().unwrap());
        let len = u64::from_le_bytes(buf[40..48].try_into().unwrap());
        Code { blake, pos, len }
    }

    fn is_tombstone(&self) -> bool {
        self.pos == TOMBSTONE
    }
}

struct Lookup {
    shards: Vec<RapidHashMap<[u8; 32], Vec<u32>>>,
}

impl Lookup {
    fn new() -> Self {
        Lookup {
            shards: (0..256).map(|_| RapidHashMap::default()).collect(),
        }
    }

    fn shard(&self, blake: &[u8; 32]) -> &RapidHashMap<[u8; 32], Vec<u32>> {
        &self.shards[blake[0] as usize]
    }

    fn shard_mut(&mut self, blake: &[u8; 32]) -> &mut RapidHashMap<[u8; 32], Vec<u32>> {
        &mut self.shards[blake[0] as usize]
    }

    fn candidates(&self, blake: &[u8; 32]) -> &[u32] {
        self.shard(blake)
            .get(blake)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn insert(&mut self, blake: [u8; 32], slot: u32) {
        self.shard_mut(&blake).entry(blake).or_default().push(slot);
    }

    fn remove(&mut self, blake: &[u8; 32], slot: u32) {
        if let Some(v) = self.shard_mut(blake).get_mut(blake) {
            v.retain(|&s| s != slot);
        }
    }
}

enum DatFile {
    #[cfg(windows)]
    Mapped(ReadMappedFile),
    Writable(File),
}

pub(crate) struct Codes {
    idx: File,
    dat_path: PathBuf,

    dat: Option<DatFile>,
    lookup: Lookup,
    free: Vec<u32>,

    slots: u32,
}

impl Codes {
    pub(in crate::plugin) fn open(name: impl AsRef<str>) -> io::Result<Self> {
        let path = get_plugin_directory().unwrap().join(name.as_ref());

        let dat_path = path.with_extension("dat");

        let mut idx = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.with_extension("idx"))?;

        let dat_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&dat_path)?;

        let dat_len = dat_file.metadata()?.len();
        let dat = Some(Self::open_dat(&dat_path, dat_file, dat_len)?);

        let idx_len = idx.seek(SeekFrom::End(0))?;
        let slots = (idx_len / RECORD_SIZE) as u32;

        let mut lookup = Lookup::new();
        let mut free = Vec::new();

        idx.seek(SeekFrom::Start(0))?;

        let mut buf = vec![0u8; RECORD_SIZE as usize];

        for slot in 0..slots {
            idx.read_exact(&mut buf)?;
            let rec = Code::from_bytes(&buf);
            if rec.is_tombstone() {
                free.push(slot);
            } else {
                lookup.insert(rec.blake, slot);
            }
        }

        Ok(Codes {
            idx,
            dat_path,
            dat,
            lookup,
            free,
            slots,
        })
    }

    fn open_dat(path: &Path, file: File, len: u64) -> io::Result<DatFile> {
        if len == 0 {
            Ok(DatFile::Writable(file))
        } else {
            drop(file);
            Ok(DatFile::Mapped(ReadMappedFile::open(path)?))
        }
    }

    fn dat_mut(&mut self) -> &mut DatFile {
        self.dat
            .as_mut()
            .expect("dat backend missing between mode switches")
    }

    fn writable_dat(&mut self) -> &mut File {
        match self.dat_mut() {
            DatFile::Writable(file) => file,
            #[cfg(windows)]
            DatFile::Mapped(_) => unreachable!("caller must ensure_writable() first"),
        }
    }

    fn ensure_writable(&mut self) -> io::Result<()> {
        if matches!(self.dat, Some(DatFile::Writable(_))) {
            return Ok(());
        }

        self.dat = None;
        self.dat = Some(DatFile::Writable(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&self.dat_path)?,
        ));
        Ok(())
    }

    #[cfg(windows)]
    fn ensure_mapped(&mut self) -> io::Result<()> {
        let len = match self.dat.as_ref() {
            Some(DatFile::Writable(file)) => file.metadata()?.len(),
            _ => return Ok(()),
        };

        if len == 0 {
            return Ok(());
        }

        self.dat = None;
        self.dat = Some(DatFile::Mapped(ReadMappedFile::open(&self.dat_path)?));

        Ok(())
    }

    pub(in crate::plugin) fn write_session(&mut self) -> io::Result<WriteSession<'_>> {
        self.ensure_writable()?;
        Ok(WriteSession {
            codes: self,
            done: false,
        })
    }

    fn read_record(&mut self, slot: u32) -> io::Result<Code> {
        if slot >= self.slots {
            return Err(io::Error::new(io::ErrorKind::NotFound, "slot out of range"));
        }
        self.idx.seek(SeekFrom::Start(slot as u64 * RECORD_SIZE))?;
        let mut buf = [0u8; RECORD_SIZE as usize];
        self.idx.read_exact(&mut buf)?;
        Ok(Code::from_bytes(&buf))
    }

    fn write_record(&mut self, slot: u32, rec: Code) -> io::Result<()> {
        self.idx.seek(SeekFrom::Start(slot as u64 * RECORD_SIZE))?;
        self.idx.write_all(&rec.to_bytes())
    }

    fn read_compressed(&mut self, rec: Code) -> io::Result<Vec<u8>> {
        let start = rec.pos as usize;
        let end = start + rec.len as usize;
        match self.dat_mut() {
            #[cfg(windows)]
            DatFile::Mapped(map) => Ok(map.as_bytes()[start..end].to_vec()),
            DatFile::Writable(file) => {
                file.seek(SeekFrom::Start(rec.pos))?;
                let mut buf = vec![0u8; rec.len as usize];
                file.read_exact(&mut buf)?;
                Ok(buf)
            }
        }
    }

    pub(in crate::plugin) fn get(&mut self, slot: u32) -> io::Result<Vec<u8>> {
        let rec = self.read_record(slot)?;
        if rec.is_tombstone() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "slot removed"));
        }
        let compressed = self.read_compressed(rec)?;
        let mut decoder = zstd::stream::read::Decoder::new(compressed.as_slice())?;
        let mut out = Vec::new();
        decoder.read_to_end(&mut out)?;
        Ok(out)
    }

    pub(in crate::plugin) fn push(&mut self, bytes: &[u8]) -> io::Result<u32> {
        let digest = Blake2s256::digest(bytes);
        let mut blake = [0u8; 32];
        blake.copy_from_slice(&digest);

        for &candidate in self.lookup.candidates(&blake).to_vec().iter() {
            if self.get(candidate)? == bytes {
                return Ok(candidate);
            }
        }

        self.ensure_writable()?;
        let compressed = zstd::bulk::compress(bytes, ZSTD_LEVEL)?;
        let dat = self.writable_dat();
        let pos = dat.seek(SeekFrom::End(0))?;
        dat.write_all(&compressed)?;

        let rec = Code {
            blake,
            pos,
            len: compressed.len() as u64,
        };
        let slot = match self.free.pop() {
            Some(slot) => {
                self.write_record(slot, rec)?;
                slot
            }
            None => {
                let slot = self.slots;
                self.idx.seek(SeekFrom::End(0))?;
                self.idx.write_all(&rec.to_bytes())?;
                self.slots += 1;
                slot
            }
        };
        self.lookup.insert(blake, slot);
        Ok(slot)
    }

    pub(in crate::plugin) fn remove_many(&mut self, slots_to_remove: &[u32]) -> io::Result<()> {
        if slots_to_remove.is_empty() {
            return Ok(());
        }
        self.ensure_writable()?;

        let removed: std::collections::HashSet<u32> = slots_to_remove.iter().copied().collect();

        for &slot in &removed {
            let rec = self.read_record(slot)?;
            if rec.is_tombstone() {
                continue;
            }
            self.lookup.remove(&rec.blake, slot);
            self.write_record(
                slot,
                Code {
                    blake: [0; 32],
                    pos: TOMBSTONE,
                    len: 0,
                },
            )?;
            self.free.push(slot);
        }

        let mut live: Vec<(u32, Code)> = Vec::with_capacity(self.slots as usize);

        for slot in 0..self.slots {
            let rec = self.read_record(slot)?;
            if !rec.is_tombstone() {
                live.push((slot, rec));
            }
        }
        live.sort_by_key(|(_, rec)| rec.pos);

        const CHUNK: usize = 64 * 1024;
        let mut buf = vec![0u8; CHUNK];
        let mut write_cursor: u64 = 0;

        for (slot, rec) in live {
            if rec.pos != write_cursor {
                let mut remaining = rec.len;
                let mut src = rec.pos;
                let mut dst = write_cursor;
                while remaining > 0 {
                    let n = remaining.min(CHUNK as u64) as usize;

                    let dat = self.writable_dat();
                    dat.seek(SeekFrom::Start(src))?;
                    dat.read_exact(&mut buf[..n])?;
                    dat.seek(SeekFrom::Start(dst))?;
                    dat.write_all(&buf[..n])?;
                    src += n as u64;
                    dst += n as u64;
                    remaining -= n as u64;
                }
                self.write_record(
                    slot,
                    Code {
                        pos: write_cursor,
                        ..rec
                    },
                )?;
            }
            write_cursor += rec.len;
        }

        self.writable_dat().set_len(write_cursor)?;
        Ok(())
    }

    pub(in crate::plugin) fn slots(&self) -> u32 {
        self.slots
    }
}

pub(in crate::plugin) struct WriteSession<'c> {
    codes: &'c mut Codes,
    done: bool,
}

impl<'c> WriteSession<'c> {
    pub(in crate::plugin) fn finish(mut self) -> io::Result<()> {
        self.done = true;
        self.codes.ensure_mapped()
    }
}

impl<'c> std::ops::Deref for WriteSession<'c> {
    type Target = Codes;
    fn deref(&self) -> &Codes {
        &*self.codes
    }
}

impl<'c> std::ops::DerefMut for WriteSession<'c> {
    fn deref_mut(&mut self) -> &mut Codes {
        &mut *self.codes
    }
}

impl<'c> Drop for WriteSession<'c> {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.codes.ensure_mapped();
        }
    }
}

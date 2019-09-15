use super::{Backend, HashSet, Path};
use crate::normalize_path;
use std::path::PathBuf;
use failure::bail;
use persy::Persy;

pub struct PersyBackend {
    persy: Persy,
    norm_path: PathBuf,
}

impl PersyBackend {
    /// USAGE: persy:PERSY_PATH:NORM_PATH[:additional_modifier]
    pub fn new(args: Vec<&str>) -> Result<PersyBackend, failure::Error> {
        if args.len() < 2 || args.len() > 3 {
            bail!("persy backend: invalid invocation (expects only 2 or 3 args)");
        }
        let persy_path = args[0];
        let norm_path = Path::new(args[1]).to_path_buf();
        let mut do_init = false;
        if args.len() == 3 {
            match args[2] {
                "init" => {
                    Persy::create(persy_path)?;
                    do_init = true;
                },
                x => bail!("persy backend: unknown modifier '{}'", x),
            }
        }
        let persy = Persy::open(persy_path, persy::Config::default())?;
        if do_init {
            let mut tx = persy.begin()?;
            persy.create_index::<String, String>(&mut tx, "zstags", persy::ValueMode::CLUSTER)?;
            let prepared = persy.prepare_commit(tx)?;
            persy.commit(prepared)?;
        }
        Ok(PersyBackend {
            persy,
            norm_path,
        })
    }

    fn mangle_path(&self, path: &Path) -> std::io::Result<PathBuf> {
        normalize_path(path, &self.norm_path)
    }
}

impl Backend for PersyBackend {
    fn tags(&self, path: &Path) -> Result<HashSet<String>, failure::Error> {
        let path_as_str = self.mangle_path(path)?.to_string_lossy().into_owned();
        Ok(self.persy.get::<String, String>("zstags", &path_as_str)?.map(|x| match x {
            persy::Value::SINGLE(x) => vec![x],
            persy::Value::CLUSTER(x) => x,
        }).unwrap_or(vec![]).into_iter().collect())
    }

    fn set_tags(&mut self, path: &Path, tags: HashSet<String>) -> Result<(), failure::Error> {
        let path_as_str = self.mangle_path(path)?.to_string_lossy().into_owned();
        let mut tx = self.persy.begin()?;
        self.persy.remove::<String, String>(&mut tx, "zstags", path_as_str.clone(), None)?;
        for i in tags {
            self.persy.put::<String, String>(&mut tx, "zstags", path_as_str.clone(), i)?;
        }
        let prepared = self.persy.prepare_commit(tx)?;
        self.persy.commit(prepared)?;
        Ok(())
    }

    fn add_tag(&mut self, path: &Path, tag: String) -> Result<(), failure::Error> {
        let path_as_str = self.mangle_path(path)?.to_string_lossy().into_owned();
        let mut tx = self.persy.begin()?;
        self.persy.remove::<String, String>(&mut tx, "zstags", path_as_str.clone(), Some(tag.clone()))?;
        self.persy.put::<String, String>(&mut tx, "zstags", path_as_str, tag)?;
        let prepared = self.persy.prepare_commit(tx)?;
        self.persy.commit(prepared)?;
        Ok(())
    }

    fn delete_tag(&mut self, path: &Path, tag: &str) -> Result<(), failure::Error> {
        let path_as_str = self.mangle_path(path)?.to_string_lossy().into_owned();
        let mut tx = self.persy.begin()?;
        self.persy.remove::<String, String>(&mut tx, "zstags", path_as_str, Some(tag.to_owned()))?;
        let prepared = self.persy.prepare_commit(tx)?;
        self.persy.commit(prepared)?;
        Ok(())
    }
}
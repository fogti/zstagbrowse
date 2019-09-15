use failure::bail;
pub use std::{collections::HashSet, path::Path};

pub trait Backend {
    fn tags(&self, path: &Path) -> Result<HashSet<String>, failure::Error>;
    fn set_tags(&mut self, path: &Path, tags: HashSet<String>) -> Result<(), failure::Error>;

    fn add_tag(&mut self, path: &Path, tag: String) -> Result<(), failure::Error> {
        let mut tags = self.tags(path)?;
        tags.insert(tag);
        self.set_tags(path, tags)
    }

    fn delete_tag(&mut self, path: &Path, tag: &str) -> Result<(), failure::Error> {
        let mut tags = self.tags(path)?;
        tags.remove(tag);
        self.set_tags(path, tags)
    }
}

#[cfg(feature = "persy")]
mod persy;

#[cfg(feature = "xattr")]
mod xattr;

/// bspec :: "<schema>[:<subspec>]"
#[allow(unused_variables)]
pub fn create_backend(bspec: &str) -> Result<Box<dyn Backend>, failure::Error> {
    let mut bssit = bspec.split(|x| x == ':');
    let bschema = bssit.next().unwrap();
    let brest: Vec<_> = bssit.collect();
    let ret: Box<dyn Backend> = match bschema {
        #[cfg(feature = "persy")]
        "persy" => Box::new(self::persy::PersyBackend::new(brest)?),
        #[cfg(feature = "xattr")]
        "xattr" => Box::new(self::xattr::XattrBackend),
        _ => bail!("got invalid backend specification (unknown/unsupported schema)"),
    };
    Ok(ret)
}

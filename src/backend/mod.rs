pub use std::{collections::HashSet, path::Path};

pub trait Backend {
    fn tags(&self, path: &Path) -> Result<HashSet<String>, failure::Error>;
    fn set_tags(&mut self, path: &Path, tags: HashSet<String>) -> Result<(), failure::Error>;

    fn add_tag(&mut self, path: &Path, tag: String) -> Result<bool, failure::Error> {
        let mut tags = self.tags(path)?;
        let ret = tags.insert(tag);
        self.set_tags(path, tags)?;
        Ok(ret)
    }

    fn delete_tag(&mut self, path: &Path, tag: &str) -> Result<bool, failure::Error> {
        let mut tags = self.tags(path)?;
        let ret = tags.remove(tag);
        self.set_tags(path, tags)?;
        Ok(ret)
    }
}

#[cfg(feature = "xattr")]
mod xattr;

/// bspec :: "<schema>[:<subspec>]"
/// ex. : "xattr" | "persy:./lookup.persy"
pub fn create_backend(bspec: &str) -> Option<Box<dyn Backend>> {
    let bschema = bspec.split(|x| x == ':').next().unwrap();
    let brest = if bspec.len() == bschema.len() {
        None
    } else {
        Some(&bspec[bschema.len() + 1..])
    };
    Some(match bspec {
        #[cfg(feature = "xattr")]
        "xattr" => Box::new(self::xattr::XattrBackend),
        _ => return None,
    })
}

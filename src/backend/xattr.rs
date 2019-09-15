use super::{Backend, HashSet, Path};

pub struct XattrBackend;

impl Backend for XattrBackend {
    fn tags(&self, path: &Path) -> Result<HashSet<String>, failure::Error> {
        match xattr::get(path, "user.zstags")? {
            Some(x) => x
                .split(|x| *x == 0 || *x == b'|')
                .map(|x| Ok(String::from(std::str::from_utf8(x)?)))
                .collect::<Result<HashSet<_>, failure::Error>>()
                .map_err(|e| e.context("zstags::XattrBackend::tags").into()),
            None => Ok(HashSet::new()),
        }
    }

    fn set_tags(&mut self, path: &Path, tags: HashSet<String>) -> Result<(), failure::Error> {
        if tags.is_empty() {
            let ret = xattr::remove(path, "user.zstags");
            if !self.tags(path)?.is_empty() {
                ret
            } else {
                Ok(())
            }
        } else {
            // "|" : some file managers and programs can't deal with xattrs containing null bytes
            xattr::set(
                path,
                "user.zstags",
                tags.into_iter().collect::<Vec<_>>().join("|").as_bytes(),
            )
        }
        .map_err(|e| {
            let e: failure::Error = e.into();
            e.context("zstags::XattrBackend::set_tags").into()
        })
    }
}

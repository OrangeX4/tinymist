//! Conversions between Typst and LSP types and representations

use tinymist_std::path::PathClean;
use tinymist_world::vfs::PathResolution;

use crate::prelude::*;

/// An LSP Position encoded by [`PositionEncoding`].
pub use tinymist_analysis::location::LspPosition;
/// An LSP range encoded by [`PositionEncoding`].
pub use tinymist_analysis::location::LspRange;

pub use tinymist_analysis::location::*;

const UNTITLED_ROOT: &str = "/untitled";
static EMPTY_URL: LazyLock<Url> = LazyLock::new(|| Url::parse("file://").unwrap());

/// Convert a path to a URL.
pub fn untitled_url(path: &Path) -> anyhow::Result<Url> {
    Ok(Url::parse(&format!("untitled:{}", path.display()))?)
}

/// Determines if a path is untitled.
pub fn is_untitled_path(p: &Path) -> bool {
    p.starts_with(UNTITLED_ROOT)
}

/// Convert a path to a URL.
pub fn path_to_url(path: &Path) -> anyhow::Result<Url> {
    if let Ok(untitled) = path.strip_prefix(UNTITLED_ROOT) {
        // rust-url will panic on converting an empty path.
        if untitled == Path::new("nEoViM-BuG") {
            return Ok(EMPTY_URL.clone());
        }

        return untitled_url(untitled);
    }

    url_from_file_path(path)
}

/// Convert a path resolution to a URL.
pub fn path_res_to_url(path: PathResolution) -> anyhow::Result<Url> {
    match path {
        PathResolution::Rootless(path) => untitled_url(path.as_ref().as_rooted_path_compat()),
        PathResolution::Resolved(path) => path_to_url(&path),
    }
}

/// Convert a URL to a path.
pub fn url_to_path(uri: &Url) -> PathBuf {
    if uri.scheme() == "file" {
        // typst converts an empty path to `Path::new("/")`, which is undesirable.
        if !uri.has_host() && uri.path() == "/" {
            return PathBuf::from("/untitled/nEoViM-BuG");
        }

        return url_to_file_path(uri);
    }

    if uri.scheme() == "untitled" {
        let mut bytes = UNTITLED_ROOT.as_bytes().to_vec();

        // This is rust-url's path_segments, but vscode's untitle doesn't like it.
        let path = uri.path();
        let segs = path.strip_prefix('/').unwrap_or(path).split('/');
        for segment in segs {
            bytes.push(b'/');
            bytes.extend(percent_encoding::percent_decode(segment.as_bytes()));
        }

        return Path::new(String::from_utf8_lossy(&bytes).as_ref()).clean();
    }

    url_to_file_path(uri)
}

#[cfg(not(target_arch = "wasm32"))]
fn url_from_file_path(path: &Path) -> anyhow::Result<Url> {
    Url::from_file_path(path).or_else(|never| {
        let _: () = never;

        anyhow::bail!("could not convert path to URI: path: {path:?}",)
    })
}

#[cfg(target_arch = "wasm32")]
fn url_from_file_path(path: &Path) -> anyhow::Result<Url> {
    virtual_path_to_url(path)
}

#[cfg(any(target_arch = "wasm32", test))]
fn virtual_path_to_url(path: &Path) -> anyhow::Result<Url> {
    // wasm32-unknown-unknown has rooted virtual paths but no native absolute paths.
    if !path.has_root() {
        anyhow::bail!("virtual path must be absolute");
    }
    let mut url = Url::parse("file:///")?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid file URL"))?;
        segments.clear();
        for component in path.components() {
            match component {
                std::path::Component::RootDir => {}
                std::path::Component::Normal(part) => {
                    segments.push(
                        part.to_str()
                            .ok_or_else(|| anyhow::anyhow!("virtual path is not UTF-8"))?,
                    );
                }
                _ => anyhow::bail!("virtual path must be normalized"),
            }
        }
    }
    Ok(url)
}

#[cfg(any(target_arch = "wasm32", test))]
fn virtual_url_to_path(uri: &Url) -> PathBuf {
    PathBuf::from(
        percent_encoding::percent_decode_str(uri.path())
            .decode_utf8_lossy()
            .as_ref(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn url_to_file_path(uri: &Url) -> PathBuf {
    uri.to_file_path()
        .unwrap_or_else(|_| panic!("could not convert URI to path: URI: {uri:?}",))
}

#[cfg(target_arch = "wasm32")]
fn url_to_file_path(uri: &Url) -> PathBuf {
    virtual_url_to_path(uri)
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn web_paths_preserve_unicode_and_reserved_characters() {
        for name in ["中文.typ", "space # ? %25.typ", "literal%20.typ"] {
            let path = Path::new("/workspace").join(name);
            let uri = virtual_path_to_url(&path).unwrap();
            assert_eq!(uri.query(), None);
            assert_eq!(uri.fragment(), None);
            assert_eq!(virtual_url_to_path(&uri), path);
        }
    }

    #[test]
    fn test_untitled() {
        let path = Path::new("/untitled/test");
        let uri = path_to_url(path).unwrap();
        assert_eq!(uri.scheme(), "untitled");
        assert_eq!(uri.path(), "test");

        let path = url_to_path(&uri);
        assert_eq!(path, Path::new("/untitled/test").clean());
        assert!(is_untitled_path(&path));
    }

    #[test]
    fn unnamed_buffer() {
        // https://github.com/neovim/nvim-lspconfig/pull/2226
        let uri = EMPTY_URL.clone();
        let path = url_to_path(&uri);
        assert_eq!(path, Path::new("/untitled/nEoViM-BuG"));

        let uri2 = path_to_url(&path).unwrap();
        assert_eq!(EMPTY_URL.clone(), uri2);
    }
}

use std::path::{PathBuf, Path, Component};
use std::collections::HashMap;
use std::fs;
use std::fmt;

type Routes = HashMap<String, PathBuf>;

#[derive(Clone)]
pub struct Router {
    routes: Routes
}

impl Router {
    /// Resolves a URI to a path
    pub fn route_to(&self, uri: &str) -> Option<&PathBuf> {
        self.routes.get(uri)
    }

    /// Registers a route for the router
//    pub fn register_route(&mut self, uri: String, path: PathBuf) {
//        self.routes.entry(uri)
//            .and_modify(|e| {*e = path})
//            .or_insert(path);
//    }

    /// Creates a set of routes for the given directory
    pub fn default_from_directory(root: &Path) -> Self {
        let mut routes = Routes::new();
        _create_directory_routes(root, "/", &mut routes);
        Self {
            routes
        }
    }
}

impl fmt::Debug for Router {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.routes)
    }
}

fn _create_directory_routes(dir: &Path, prefix: &str, routes: &mut Routes) {
    // Add the index for the directory, if it exists
    let indices = ["index.html", "index.htm"];
    for s in indices.iter() {
        let index = dir.join(s);
        if index.exists() {
            routes.insert(String::from(prefix), index);
            break;
        }
    }

    // Insert URI-to-path resolutions for this directory
    for entry in fs::read_dir(dir).unwrap() {
        let abs_path = if let Ok(de) = entry {
            de.path()
        } else {
            continue;
        };

        let component = abs_path.components().last().unwrap();

        if let Component::Normal(osstr) = component {
            let path = Path::new(osstr);

            if abs_path.is_dir() {
                let new_dir = dir.join(&path);
                let new_prefix = format!("{}{}/",
                                         prefix,
                                         path.to_str().unwrap()
                                             .trim_left_matches("./"));
                _create_directory_routes(&dir.join(&path), &new_prefix, routes);
            } else if abs_path.is_file() {
                let uri = format!("{}{}", prefix, path.to_str().unwrap().trim_left_matches("./"));
                routes.insert(uri, dir.join(&path));
            } else {
                continue;
            }
        }
    }
}
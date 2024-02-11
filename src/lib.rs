
pub trait PluginTrait: Send + Sync {
    /// 注册插件
    fn register(&self) -> Plugin;
    /// 加载插件
    fn load(&self) {}
    /// 重载插件
    fn reload(&self) {}
    ///卸载插件
    fn unload(&self) {}
}
#[repr(C)]
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub author: String,
    pub explain: String,
}

impl Default for Plugin {
    fn default() -> Self {
        let version: &str = env!("CARGO_PKG_VERSION");
        let name: &str = env!("CARGO_PKG_NAME");
        let author: &str = env!("CARGO_PKG_AUTHORS");
        Self {
            name: name.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            explain: "not explain".to_owned(),
        }
    }
}

impl Plugin {
    /// set plugin name
    fn set_name(&mut self, name: String) -> &Self {
        self.name = name;
        self
    }

    /// set plugin version
    fn set_version(&mut self, version: String) -> &Self {
        self.version = version;
        self
    }

    /// set plugin author
    fn set_author(&mut self, author: String) -> &Self {
        self.author = author;
        self
    }

    /// set plugin explain
    fn set_explain(&mut self, explain: String) -> &Self {
        self.explain = explain;
        self
    }
}

pub enum PlguninResult<T> {
    Ok(T),
    Err(T),
}

pub(crate) struct UcenterResult<T>(pub T);

use libloader::libloading::{Library, Symbol};
use std::{collections::HashMap, ffi::OsStr, fs, sync::Arc};

pub struct PluginManager {
    path: String,
    plugin_hashmap: HashMap<String, Arc<Box<dyn PluginTrait>>>,
    loaded_libraries: Vec<Library>,
}

impl PluginManager {
    /**
     let path = "./plugins";
     let mut app_extend_manager = PluginManager::new(path.to_owned());
     app_extend_manager.load_all();
     app_extend_manager.unload_all();
    */
    pub fn new(path: String) -> PluginManager {
        fs::create_dir(&path).err();
        PluginManager {
            path,
            plugin_hashmap: HashMap::new(),
            loaded_libraries: Vec::new(),
        }
    }

    //插件目录下所有插件
    pub fn load_all(&mut self) -> PlguninResult<()> {
        let r = fs::read_dir(self.path.clone())
            .map_err(|err| println!("error to filedir->{}", err))
            .unwrap();
        for i in r {
            let entity = i
                .map_err(|err| println!("error to filename->{}", err))
                .unwrap();
            let path = entity.path();
            let match_ext = {
                if cfg!(target_os = "windows") {
                    path.extension()
                        .map(|v| v.to_str().unwrap())
                        .unwrap_or("")
                        .eq("dll")
                } else {
                    path.extension()
                        .map(|v| v.to_str().unwrap())
                        .unwrap_or("")
                        .eq("so")
                }
            };
            if path.is_file() && match_ext {
                unsafe { self.load_extend(path) }.unwrap();
            }
        }
        PlguninResult::Ok(())
    }

    ///加载插件
    unsafe fn load_extend<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<(), String> {
        type PluginTraitCreator = unsafe fn() -> *mut dyn PluginTrait;

        let lib = Library::new(filename.as_ref()).or(Err({})).unwrap();

        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();
        let constructor: Symbol<PluginTraitCreator> = lib.get(b"_post_plugin").unwrap();
        let boxed_raw = constructor();

        let extend = Box::from_raw(boxed_raw);
        extend.load();
        let plugin = extend.register();
        self.plugin_hashmap
            .insert(plugin.name.to_string(), Arc::new(extend));

        Ok(())
    }

    ///卸载全部插件
    pub fn unload_all(&mut self) {
        for (_name, plgunin) in &self.plugin_hashmap {
            plgunin.unload();
        }
        self.plugin_hashmap.clear();
    }

    ///卸载全部插件
    pub fn reload_all(&mut self) {
        for (_name, plgunin) in &self.plugin_hashmap {
            plgunin.reload();
        }
        self.plugin_hashmap.clear();
    }

    ///获取插件指针
    pub fn select<T: Into<String>>(&self, target: T) -> PlguninResult<Arc<Box<dyn PluginTrait>>> {
        let key: String = target.into();
        PlguninResult::Ok(self.plugin_hashmap.get(&key).map(|v| v.clone()).unwrap())
    }
}

pub trait PluginTrait: Send + Sync {
    /// 注册插件
    fn register(&self) -> Plugin;
    /// 加载插件
    fn load(&self) {}
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

// impl Plugin {
//     /// set plugin name
//     fn set_name(&mut self, name: String) -> &Self {
//         self.name = name;
//         self
//     }

//     /// set plugin version
//     fn set_version(&mut self, version: String) -> &Self {
//         self.version = version;
//         self
//     }

//     /// set plugin author
//     fn set_author(&mut self, author: String) -> &Self {
//         self.author = author;
//         self
//     }

//     /// set plugin explain
//     fn set_explain(&mut self, explain: String) -> &Self {
//         self.explain = explain;
//         self
//     }
// }

pub enum PlguninResult<T> {
    Ok(T),
    Err,
}

use libloader::libloading::{Library, Symbol};
use std::{collections::HashMap, fs, sync::Arc};

#[derive(Debug)]
pub struct PluginManager {
    path: String,
    plugins: HashMap<String, Arc<Box<dyn PluginTrait>>>,
    loaded_libraries: Vec<Library>,
}

impl Default for PluginManager {
    fn default() -> Self {
        let plugin_manager = Self {
            path: "./plugins".to_owned(),
            plugins: HashMap::new(),
            loaded_libraries: Vec::new(),
        };
        fs::create_dir(&plugin_manager.path).err();
        plugin_manager
    }
}

impl PluginManager {
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
                let file_name = path.file_name().unwrap().to_str().unwrap();
                unsafe { self.load_extend(file_name) }.unwrap();
            }
        }
        PlguninResult::Ok(())
    }

    /**
     *
     * warn !!!
     *
     * filename in plugins-> files error
     * "ibplugin.so"
     *
     * unsafe { plugin.load_extend("libplugin.so") };
     */
    unsafe fn load_extend(&mut self, filename: &str) -> Result<(), String> {
        type PluginTraitCreator = unsafe fn() -> *mut dyn PluginTrait;
        let path = format!("{}/{}", self.path.as_str(), filename);
        let lib = Library::new(path).or(Err({})).unwrap();

        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();
        let constructor: Symbol<PluginTraitCreator> = lib.get(b"_post_plugin").unwrap();
        let boxed_raw = constructor();

        let extend = Box::from_raw(boxed_raw);
        extend.load();
        let plugin = extend.register();
        self.plugins
            .insert(plugin.name.to_string(), Arc::new(extend));

        Ok(())
    }

    ///卸载全部插件
    pub fn unload_all(&mut self) {
        for (_name, plgunin) in &self.plugins {
            plgunin.unload();
        }
        self.plugins.clear();
    }

    ///重载全部插件
    pub fn reload_all(&mut self) {
        self.plugins.clear();
        self.load_all();
    }

    ///获取插件
    pub fn select<T: Into<String>>(&self, target: T) -> PlguninResult<Arc<Box<dyn PluginTrait>>> {
        let key: String = target.into();
        let plugin = self.plugins.get(&key).map(|v| v.clone());
        match plugin {
            Some(plugin) => PlguninResult::Ok(plugin),
            None => PlguninResult::Err,
        }
    }

}

#[test]
fn test() {
    let mut plugin_manager = PluginManager::default();
    plugin_manager.load_all();
    println!("当前剩余插件 {}", plugin_manager.plugins.len());
    plugin_manager.unload_all();
    println!("当前剩余插件 {}", plugin_manager.plugins.len());
    
    unsafe { plugin_manager.load_extend("libplugin.so") };

    let plugin_na = plugin_manager.select("plugin_manager_lib");
    match plugin_na {
        PlguninResult::Ok(plugin) => {
            plugin.unload();
            println!("插件存在");
        }
        PlguninResult::Err => {
            println!("插件不存在")
        }
    }
    println!("当前剩余插件 {}", plugin_manager.plugins.len());
}

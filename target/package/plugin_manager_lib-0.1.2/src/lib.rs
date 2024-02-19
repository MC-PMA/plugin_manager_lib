use std::any::Any;

pub trait PluginTrait: Any + Send + Sync {
    /// 注册插件
    fn register(&self) -> Plugin;
    /// 加载插件
    fn load(&self) {}
    ///卸载插件
    fn unload(&self) {}
}
#[repr(C)]
#[derive(Debug, Clone, Serialize)]
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

pub enum PlguninResult<T> {
    Ok(T),
    Err,
}

use libloader::libloading::{Library, Symbol};
use serde::Serialize;
use std::{collections::HashMap, fs, sync::Arc};

pub struct PluginManager {
    path: String,
    pub plugins: HashMap<String, Arc<Box<dyn PluginTrait>>>,
    pub loaded_libraries: Vec<Library>,
    pub plugin_structs: HashMap<String, Plugin>,
}

impl Default for PluginManager {
    fn default() -> Self {
        let plugin_manager = Self {
            path: "./plugins".to_owned(),
            plugins: HashMap::new(),
            loaded_libraries: Vec::new(),
            plugin_structs: HashMap::new(),
        };
        fs::create_dir(&plugin_manager.path).err();
        plugin_manager
    }
}

impl PluginManager {
    //插件目录下所有插件
    pub fn load_all(&mut self) -> PlguninResult<()> {
        let r = fs::read_dir(PluginManager::default().path)
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
        // 加载动态库
        let lib = Library::new(path).or(Err({})).unwrap();

        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();

        // 取得函数符号
        let constructor: Symbol<PluginTraitCreator> = lib.get(b"_post_plugin").unwrap();

        // 调用该函数，取得 UcenterApp Trait 实例的原始指针
        let boxed_raw = constructor();

        // 通过原始指针构造 Box，至此逻辑重归安全区
        let extend = Box::from_raw(boxed_raw);
        let plugin = extend.register();
        extend.load();
        self.plugins.insert(plugin.clone().name, extend.into());
        println!("加载插件: {}", plugin.name);
        self.plugin_structs.insert(plugin.clone().name, plugin);

        Ok(())
    }

    // 选取指定 name 的拓展
    pub fn select<T: Into<String>>(
        &self,
        target: T,
    ) -> Option<(&std::string::String, &Arc<Box<dyn PluginTrait>>)> {
        let key: String = target.into();
        self.plugins.get_key_value(&key)
    }

    ///卸载全部插件
    pub fn unload_all(&mut self) {
        self.plugins.clear();
        self.plugin_structs.clear();
        self.loaded_libraries.clear();
    }

    ///重载全部插件
    pub fn reload_all(&mut self) {
        self.plugins.clear();
        self.plugin_structs.clear();
        self.loaded_libraries.clear();
        self.load_all();
    }
}
#[test]
fn test() {
    let mut plugin_manager = PluginManager::default();
    
    plugin_manager.load_all();
    println!(
        "加载全部插件-> 当前剩余插件 {}",
        plugin_manager.plugins.len()
    );

    plugin_manager.unload_all();
    println!(
        "卸载全部插件-> 当前剩余插件 {}",
        plugin_manager.plugins.len()
    );

    unsafe { plugin_manager.load_extend("libplugin.so") };

    match plugin_manager.select("plugin_manager_lib") {
        Some((name, plugin)) => {
            println!("插件存在");
            plugin.unload()
        }
        None => {
            println!("插件不存在")
        }
    }

    println!(
        "加载指定插件文件-> 当前剩余插件 {}",
        plugin_manager.plugins.len()
    );

    plugin_manager.reload_all();
    println!(
        "重载全部插件-> 当前剩余插件 {}",
        plugin_manager.plugins.len()
    );

    plugin_manager.unload_all();
    println!(
        "卸载全部插件-> 当前剩余插件 {}",
        plugin_manager.plugins.len()
    );
}

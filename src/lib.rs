use pmaapi::{Plugin, PluginTrait};

#[no_mangle]
pub extern "C" fn _post_plugin() -> *mut dyn PluginTrait {
    // 创建对象
    let object = QASystemExtend::default(); // 通过 Box 在堆上存储该对象实例
    let boxed: Box<dyn PluginTrait> = Box::new(object);
    // 返回原始指针（这是个 unsafe 调用，不过在 extern "C" 这种 ABI 定义处整个代码段都处于 unsafe 下，所以不用额外写 unsafe）
    Box::into_raw(boxed)
}

#[derive(Default, Debug)]
pub struct QASystemExtend;
impl PluginTrait for QASystemExtend {
    fn register(&self) -> Plugin {
        Plugin::default()
    }
    fn load(&self) {
        println!("插件加载")
    }
    fn reload(&self) {
        println!("重载插件")
    }
    fn unload(&self) {
        println!("卸载插件")
    }
}

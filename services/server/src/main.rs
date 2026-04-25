use anyhow::Result;
use webmagic_core::module::ModuleDescriptor;

fn main() -> Result<()> {
    let module =
        ModuleDescriptor::new("server", "控制面服务，后续承接发布、任务、日志和鉴权 API。");

    println!("webmagic-ng {}", module.summary());
    Ok(())
}

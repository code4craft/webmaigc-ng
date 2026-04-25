use anyhow::Result;
use webmagic_core::module::ModuleDescriptor;

fn main() -> Result<()> {
    let module = ModuleDescriptor::new(
        "worker",
        "执行面服务，后续承接任务拉取、运行隔离和状态上报。",
    );

    println!("webmagic-ng {}", module.summary());
    Ok(())
}

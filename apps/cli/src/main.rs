use anyhow::Result;
use webmagic_core::module::ModuleDescriptor;

fn main() -> Result<()> {
    let module = ModuleDescriptor::new(
        "cli",
        "Quick Start 命令行入口，后续承接本地分析、生成和执行流程。",
    );

    println!("webmagic-ng {}", module.summary());
    Ok(())
}

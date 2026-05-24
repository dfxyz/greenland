use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

#[derive(clap::Parser)]
struct Arguments {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Default, clap::Subcommand)]
enum Command {
    #[default]
    #[command(about = "启动图形界面")]
    GUI,
    #[command(about = "使用指定的模板导出配色方案")]
    Export {
        #[arg(help = "模板文件路径", name = "TEMPLATE")]
        template_path: PathBuf,
        #[arg(help = "输出文件路径（使用'-'输出到标准输出）", name = "OUTPUT")]
        output_path: PathBuf,
    },
    #[command(
        name = "jb",
        about = "比较模板导出结果与指定的JetBrains配色方案之间的差异"
    )]
    CompareJetBrains {
        #[arg(help = "模板文件路径", name = "TEMPLATE")]
        template_path: PathBuf,
        #[arg(help = "目标配色方案文件路径", name = "TARGET")]
        target_path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let _guard = naive_logger::init();
    let args = Arguments::parse();
    if let Err(e) = match args.command.unwrap_or_default() {
        Command::GUI => greenland::app::run(),
        Command::Export {
            template_path: template,
            output_path: output,
        } => run_export_command(template, output),
        Command::CompareJetBrains {
            template_path: template,
            target_path: scheme,
        } => run_compare_jetbrains_command(template, scheme),
    } {
        eprintln!("{e:?}");
        return Err(e);
    }
    Ok(())
}

fn run_export_command(template_path: PathBuf, output_path: PathBuf) -> anyhow::Result<()> {
    let scheme = greenland::utils::ColorScheme::load_from(greenland::DEFAULT_SCHEME_FILENAME)
        .context("无法加载默认配色方案")?;
    let result =
        greenland::utils::process_template(template_path, &scheme).context("处理模板时发生错误")?;
    if output_path.to_string_lossy() == "-" {
        println!("{result}");
    } else {
        std::fs::write(output_path, result).context("输出结果时发生错误")?;
    }
    Ok(())
}

fn run_compare_jetbrains_command(
    tempate_path: PathBuf,
    target_path: PathBuf,
) -> anyhow::Result<()> {
    let scheme = greenland::utils::ColorScheme::load_from(greenland::DEFAULT_SCHEME_FILENAME)
        .context("无法加载默认配色方案")?;
    let result =
        greenland::utils::process_template(tempate_path, &scheme).context("处理模板时发生错误")?;
    let template_scheme = greenland::utils::jetbrains::ColorScheme::from_str(&result)
        .context("解析模板导出的配色方案时发生错误")?;
    let target_scheme = greenland::utils::jetbrains::ColorScheme::load(target_path)
        .context("无法加载目标配色方案")?;
    let difference = template_scheme.compare(&target_scheme);
    if !difference.removed_colors.is_empty() {
        println!("移除了以下颜色：");
        for name in difference.removed_colors {
            println!("- `{name}`");
        }
        println!(
            "--------------------------------------------------------------------------------"
        );
    }
    if !difference.removed_attributes.is_empty() {
        println!("移除了以下属性：");
        for name in difference.removed_attributes {
            println!("- `{name}`");
        }
        println!(
            "--------------------------------------------------------------------------------"
        );
    }
    if !difference.modified.colors.is_empty() {
        println!("修改了以下颜色：");
        for (name, value) in difference.modified.colors {
            match value {
                Some(value) => {
                    let result = scheme.get_by_rgba_u32(value);
                    if result.is_empty() {
                        println!("    <option name=\"{name}\" value=\"{value:X}\" />");
                    } else {
                        for (prefix, entry) in result {
                            for purpose in &entry.purposes {
                                println!(
                                    "    <option name=\"{name}\" value=\"${prefix}{purpose}$\" />"
                                );
                            }
                        }
                    }
                }
                None => {
                    println!("    <option name=\"{name}\" value=\"\" />");
                }
            }
        }
        println!(
            "--------------------------------------------------------------------------------"
        );
    }
    if !difference.modified.attributes.is_empty() {
        println!("修改了以下属性：");
        for (name, value) in difference.modified.attributes {
            match value {
                greenland::utils::jetbrains::Attribute::Inherit(base) => {
                    println!("<option name=\"{name}\" baseAttributes=\"{base}\" />");
                }
                greenland::utils::jetbrains::Attribute::Value(attr) => {
                    if attr.is_empty() {
                        println!("<option name=\"{name}\">");
                        println!("  <value />");
                        println!("</option>");
                    } else {
                        println!("<option name=\"{name}\">");
                        println!("  <value>");

                        macro_rules! check_and_print {
                            (u32: $field:ident, $key:literal) => {
                                if let Some(value) = attr.$field {
                                    let results = scheme.get_by_rgba_u32(value);
                                    if results.is_empty() {
                                        println!(
                                            concat!(
                                                "    <option name=\"",
                                                $key,
                                                "\" value=\"{:X}\" />"
                                            ),
                                            value
                                        );
                                    } else {
                                        for (prefix, entry) in results {
                                            for purpose in &entry.purposes {
                                                println!(
                                                    concat!(
                                                        "    <option name=\"",
                                                        $key,
                                                        "\" value=\"${}{}$\" />"
                                                    ),
                                                    prefix, purpose
                                                );
                                            }
                                        }
                                    }
                                }
                            };
                            (i8: $field:ident, $key:literal) => {
                                if let Some(value) = attr.$field {
                                    println!(
                                        concat!("    <option name=\"", $key, "\" value=\"{}\" />"),
                                        value
                                    );
                                }
                            };
                        }

                        check_and_print!(u32: fg, "FOREGROUND");
                        check_and_print!(u32: bg, "BACKGROUND");
                        check_and_print!(u32: stripe, "ERROR_STRIPE_COLOR");
                        check_and_print!(u32: ec, "EFFECT_COLOR");
                        check_and_print!(i8: et, "EFFECT_TYPE");
                        check_and_print!(i8: ft, "FONT_TYPE");

                        println!("  </value>");
                        println!("</option>\n");
                    }
                }
            }
        }
    }
    Ok(())
}

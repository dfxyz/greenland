use std::path::Path;

use anyhow::Context as _;
use serde::Deserialize;

use crate::utils::{ColorScheme, OklchColor};

/// 模板元数据
#[derive(Deserialize)]
struct TemplateMetadata {
    format: OutputFormat,
}

/// 颜色的输出格式
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum OutputFormat {
    /// HEX6或HEX8字符串
    Hex,
    /// HEX8字符串
    Hex8,
    /// 不带`#`前缀的HEX6或HEX8字符串
    HexWithoutHash,
    /// RGB的u32整数表示
    RgbNumber,
    /// BGR的u32整数表示
    BgrNumber,
}

/// 颜色的修正参数
#[derive(Default, Deserialize)]
#[serde(default)]
struct ColorCorrectionArg {
    /// 亮度修正（乘以该值）
    l: Option<f32>,
    /// 色度修正（乘以该值）
    c: Option<f32>,
    /// 色相修正（加该值）
    h: Option<i32>,
    /// 透明度修正（乘以该值）
    a: Option<f32>,
    /// 输出格式（覆盖默认值）
    f: Option<OutputFormat>,
}

/// 处理指定路径的模板文件，返回转换后的字符串
pub fn process_template<P: AsRef<Path>>(path: P, scheme: &ColorScheme) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(path).context("无法读取模板文件")?;
    let mut lines = content.lines();
    let mut metadata_lines = Vec::new();
    let mut metadata_ok = false;
    for line in &mut lines {
        let line = line.trim();
        if line.starts_with("---") {
            metadata_ok = true;
            break;
        }
        metadata_lines.push(line);
    }
    if !metadata_ok {
        anyhow::bail!("模板文件格式错误：缺少元数据分隔符'---'");
    }
    let metadata: TemplateMetadata =
        toml::from_str(&metadata_lines.join("\n")).context("解析模板元数据失败")?;
    process_body(scheme, &metadata, lines)
}

fn process_body<'a>(
    scheme: &ColorScheme,
    metadata: &TemplateMetadata,
    lines: impl IntoIterator<Item = &'a str>,
) -> anyhow::Result<String> {
    let mut result = String::new();
    for mut line in lines {
        let mut processed_line = String::new();
        while let Some(dollar_pos) = line.find('$') {
            processed_line.push_str(&line[..dollar_pos]);
            line = &line[dollar_pos + 1..];
            if line.starts_with("$$") {
                processed_line.push('$');
                line = &line[2..];
                continue;
            }
            let Some(dollar_pos) = line.find('$') else {
                anyhow::bail!(
                    "模板文件格式错误：发现未闭合的模板边界符'$'\n已处理内容：{}\n剩余内容：{}",
                    processed_line.trim(),
                    line.trim()
                );
            };
            let purpose = &line[..dollar_pos];
            line = &line[dollar_pos + 1..];
            let parts: Vec<&str> = purpose.split('|').collect();
            let purpose = &parts[0];
            let color = scheme
                .get_by_purpose(purpose)
                .map(|entry| entry.color)
                .with_context(|| format!("模板中引用了未定义的颜色用途'{}'", purpose))?;
            let mut correction_args = String::new();
            for part in &parts[1..] {
                if part.contains('=') {
                    correction_args.push_str(part);
                    correction_args.push('\n');
                } else {
                    correction_args.push_str(&format!("f=\"{part}\"\n"));
                }
            }
            let correction_args: ColorCorrectionArg = toml::from_str(&correction_args)
                .with_context(|| format!("解析颜色用途'{}'的修正参数失败", purpose))?;
            write_color(&mut processed_line, color, correction_args, metadata.format)?;
        }
        if !line.is_empty() {
            processed_line.push_str(line);
        }
        result.push_str(&processed_line);
        result.push('\n');
    }
    Ok(result)
}

fn write_color(
    string: &mut String,
    mut color: OklchColor,
    correction: ColorCorrectionArg,
    default_format: OutputFormat,
) -> anyhow::Result<()> {
    use std::fmt::Write;
    if let Some(l) = correction.l {
        color.set_l(color.l() * l);
    }
    if let Some(c) = correction.c {
        color.set_c(color.c() * c);
    }
    if let Some(h) = correction.h {
        color.set_h((color.h() + h as f32).rem_euclid(360.0));
    }
    if let Some(a) = correction.a {
        color.set_a(color.a() * a);
    }
    match correction.f.unwrap_or(default_format) {
        OutputFormat::Hex => write!(string, "{}", color.to_hex_string()),
        OutputFormat::Hex8 => write!(string, "{}", color.to_hex8_string()),
        OutputFormat::HexWithoutHash => write!(string, "{}", &color.to_hex_string()[1..]),
        OutputFormat::RgbNumber => write!(string, "{}", color.to_rgb_u32()),
        OutputFormat::BgrNumber => write!(string, "{}", color.to_bgr_u32()),
    }
    .context("写入格式化字符串时发生错误")
}

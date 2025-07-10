use super::progress::format_number;
use crate::core::{analysis::ProjectAnalysis, error::Result};
use colored::Colorize;

pub fn print_table_format(
    project_analysis: &ProjectAnalysis,
    detailed: bool,
    quiet: bool,
) {
    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    if !quiet {
        println!("{}", "=".repeat(80));
    }

    println!(" {:<56} {}", "Project", summary.project_name);
    println!(
        " {:<56} {}",
        "Total Files",
        format_number(summary.total_files)
    );
    println!(
        " {:<56} {}",
        "Total Lines",
        format_number(summary.total_lines)
    );
    println!(
        " {:<56} {}",
        "Code Lines",
        format_number(summary.total_code_lines)
    );
    println!(
        " {:<56} {}",
        "Comment Lines",
        format_number(summary.total_comment_lines)
    );
    println!(
        " {:<56} {}",
        "Blank Lines",
        format_number(summary.total_blank_lines)
    );
    println!(
        " {:<56} {}",
        "Languages",
        format_number(summary.language_count)
    );
    if let Some(ref primary) = summary.primary_language {
        println!(" {:<56} {}", "Primary Language", primary);
    }
    println!(
        " {:<56} {:.1}%",
        "Code Ratio",
        summary.overall_complexity_ratio * 100.0
    );
    println!(
        " {:<56} {:.1}%",
        "Documentation",
        summary.overall_documentation_ratio * 100.0
    );

    if !language_stats.is_empty() && !quiet {
        println!("{}", "=".repeat(80));

        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>8}",
            "Language".bold(),
            "Files",
            "Lines",
            "Code",
            "Comments",
            "Blanks",
            "Share%"
        );
        println!("{}", "=".repeat(80));

        for stats in &language_stats {
            let share_percentage = if summary.total_lines > 0 {
                (stats.total_lines as f64 / summary.total_lines as f64) * 100.0
            } else {
                0.0
            };

            println!(
                " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
                stats.language_name,
                format_number(stats.file_count),
                format_number(stats.total_lines),
                format_number(stats.code_lines),
                format_number(stats.comment_lines),
                format_number(stats.blank_lines),
                share_percentage
            );
        }

        println!("{}", "=".repeat(80));
        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
            "Total".bold(),
            format_number(summary.total_files),
            format_number(summary.total_lines),
            format_number(summary.total_code_lines),
            format_number(summary.total_comment_lines),
            format_number(summary.total_blank_lines),
            100.0
        );
    }

    if detailed && !quiet {
        println!("{}", "=".repeat(80));

        for (lang_name, analysis) in &project_analysis.language_analyses {
            if !analysis.file_metrics.is_empty() {
                println!();
                println!("{} Files", lang_name.bold());

                for file in &analysis.file_metrics {
                    println!(
                        "   {:<50} {:>6} lines ({} code, {} comments)",
                        file.file_path,
                        format_number(file.total_lines),
                        format_number(file.code_lines),
                        format_number(file.comment_lines)
                    );
                }
            }
        }
    }
}

pub fn print_json_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(project_analysis)?;
    println!("{json}");
    Ok(())
}

pub fn print_csv_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let language_stats = project_analysis.get_language_statistics();
    let summary = project_analysis.get_summary();

    println!("Language,Files,Lines,Code,Comments,Blanks,SharePercent");
    for stats in language_stats {
        let share_percentage = if summary.total_lines > 0 {
            (stats.total_lines as f64 / summary.total_lines as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "\"{}\",{},{},{},{},{},{:.2}",
            stats.language_name,
            format_number(stats.file_count),
            format_number(stats.total_lines),
            format_number(stats.code_lines),
            format_number(stats.comment_lines),
            format_number(stats.blank_lines),
            share_percentage
        );
    }

    Ok(())
}

pub fn print_xml_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!("<project_analysis>");

    println!(
        "  <project_name>{}</project_name>",
        xml_escape(&summary.project_name)
    );

    println!("  <summary>");
    println!("    <total_files>{}</total_files>", summary.total_files);
    println!("    <total_lines>{}</total_lines>", summary.total_lines);
    println!(
        "    <total_code_lines>{}</total_code_lines>",
        summary.total_code_lines
    );
    println!(
        "    <total_comment_lines>{}</total_comment_lines>",
        summary.total_comment_lines
    );
    println!(
        "    <total_blank_lines>{}</total_blank_lines>",
        summary.total_blank_lines
    );
    println!(
        "    <language_count>{}</language_count>",
        summary.language_count
    );

    if let Some(ref primary_lang) = summary.primary_language {
        println!(
            "    <primary_language>{}</primary_language>",
            xml_escape(primary_lang)
        );
    }

    println!(
        "    <overall_complexity_ratio>{:.6}</overall_complexity_ratio>",
        summary.overall_complexity_ratio
    );
    println!(
        "    <overall_documentation_ratio>{:.6}</overall_documentation_ratio>",
        summary.overall_documentation_ratio
    );
    println!("  </summary>");

    println!("  <language_statistics>");
    for stats in language_stats {
        println!("    <language>");
        println!("      <name>{}</name>", xml_escape(&stats.language_name));
        println!("      <file_count>{}</file_count>", stats.file_count);
        println!("      <total_lines>{}</total_lines>", stats.total_lines);
        println!("      <code_lines>{}</code_lines>", stats.code_lines);
        println!(
            "      <comment_lines>{}</comment_lines>",
            stats.comment_lines
        );
        println!("      <blank_lines>{}</blank_lines>", stats.blank_lines);
        println!(
            "      <complexity_ratio>{:.6}</complexity_ratio>",
            stats.complexity_ratio
        );
        println!("    </language>");
    }
    println!("  </language_statistics>");

    println!("</project_analysis>");
    Ok(())
}

fn xml_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

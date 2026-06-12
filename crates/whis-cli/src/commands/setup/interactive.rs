//! Interactive prompt helpers using dialoguer.

use anyhow::Result;
use dialoguer::{Input, Password, Select, console::Style, console::style, theme::ColorfulTheme};

pub fn theme() -> ColorfulTheme {
    ColorfulTheme {
        success_prefix: style("[*]".to_string()).for_stderr(),
        success_suffix: style("".to_string()).for_stderr(),
        checked_item_prefix: style("[*]".to_string()).for_stderr(),
        unchecked_item_prefix: style("[ ]".to_string()).for_stderr(),
        active_item_prefix: style("[>]".to_string()).for_stderr(),
        inactive_item_prefix: style("[ ]".to_string()).for_stderr(),
        prompt_prefix: style("[?]".to_string()).for_stderr(),
        prompt_suffix: style("".to_string()).for_stderr(),
        error_prefix: style("[!]".to_string()).for_stderr(),
        values_style: Style::new(),
        prompt_style: Style::new(),
        active_item_style: Style::new(),
        inactive_item_style: Style::new(),
        ..ColorfulTheme::default()
    }
}

pub fn select<T: std::fmt::Display>(
    prompt: &str,
    items: &[T],
    default: Option<usize>,
) -> Result<usize> {
    let theme = theme();
    let mut select = Select::with_theme(&theme).with_prompt(prompt).items(items);
    if let Some(idx) = default {
        select = select.default(idx);
    }
    Ok(select.interact()?)
}

pub fn select_clean(
    prompt: &str,
    items: &[impl AsRef<str>],
    clean_items: &[impl AsRef<str>],
    default: Option<usize>,
) -> Result<usize> {
    let theme = theme();
    let mut select = Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(items.iter().map(|s| s.as_ref()).collect::<Vec<_>>())
        .report(false);
    if let Some(idx) = default {
        select = select.default(idx);
    }
    let idx = select.interact()?;
    eprintln!("{} {}  {}", style("[*]"), prompt, clean_items[idx].as_ref());
    Ok(idx)
}

pub fn input(prompt: &str, default: Option<&str>) -> Result<String> {
    let theme = theme();
    let mut input = Input::with_theme(&theme).with_prompt(prompt);
    if let Some(d) = default {
        input = input.default(d.to_string());
    }
    Ok(input.interact_text()?)
}

#[allow(dead_code)]
pub fn password(prompt: &str) -> Result<String> {
    let theme = theme();
    Ok(Password::with_theme(&theme)
        .with_prompt(prompt)
        .interact()?)
}

pub fn error(text: &str) {
    eprintln!("{} {}", style("[!]").bold(), text);
}

pub fn info(text: &str) {
    println!("{} {}", style("[i]").for_stderr(), text);
}

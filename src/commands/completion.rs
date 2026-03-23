use std::io;

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::{Cli, CompletionShell};

pub fn execute(shell: CompletionShell) -> Result<()> {
    let mut command = Cli::command();
    let binary_name = command.get_name().to_string();
    let mut stdout = io::stdout();

    generate(to_generator(shell), &mut command, binary_name, &mut stdout);

    Ok(())
}

fn to_generator(shell: CompletionShell) -> Shell {
    match shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Elvish => Shell::Elvish,
        CompletionShell::Fish => Shell::Fish,
        CompletionShell::PowerShell => Shell::PowerShell,
        CompletionShell::Zsh => Shell::Zsh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_zsh_completion_script() {
        let mut command = Cli::command();
        let mut output = Vec::new();

        generate(Shell::Zsh, &mut command, "aip", &mut output);

        let script =
            String::from_utf8(output).expect("generated completion script should be valid UTF-8");

        assert!(script.contains("_aip"));
        assert!(script.contains("claude"));
        assert!(script.contains("completion"));
    }
}

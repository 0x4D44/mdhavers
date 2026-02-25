# Fish completion script for mdhavers
# Install: copy to ~/.config/fish/completions/mdhavers.fish

# Disable file completion by default
complete -c mdhavers -f

# Global options
complete -c mdhavers -s h -l help -d 'Show help information'
complete -c mdhavers -s V -l version -d 'Show version information'

# Commands
complete -c mdhavers -n __fish_use_subcommand -a run -d 'Run a mdhavers program'
complete -c mdhavers -n __fish_use_subcommand -a build -d 'Compile to native executable'
complete -c mdhavers -n __fish_use_subcommand -a repl -d 'Start interactive REPL'
complete -c mdhavers -n __fish_use_subcommand -a fmt -d 'Format source code'
complete -c mdhavers -n __fish_use_subcommand -a check -d 'Check for errors'
complete -c mdhavers -n __fish_use_subcommand -a help -d 'Show help information'

# run command
complete -c mdhavers -n '__fish_seen_subcommand_from run' -s h -l help -d 'Show help'
complete -c mdhavers -n '__fish_seen_subcommand_from run' -F -a '*.braw' -d 'Source file'

# build command
complete -c mdhavers -n '__fish_seen_subcommand_from build' -s h -l help -d 'Show help'
complete -c mdhavers -n '__fish_seen_subcommand_from build' -s o -l output -r -d 'Output file'
complete -c mdhavers -n '__fish_seen_subcommand_from build' -l opt-level -x -a '0 1 2 3' -d 'Optimization level'
complete -c mdhavers -n '__fish_seen_subcommand_from build' -F -a '*.braw' -d 'Source file'

# repl command
complete -c mdhavers -n '__fish_seen_subcommand_from repl' -s h -l help -d 'Show help'

# fmt command
complete -c mdhavers -n '__fish_seen_subcommand_from fmt' -s h -l help -d 'Show help'
complete -c mdhavers -n '__fish_seen_subcommand_from fmt' -l check -d 'Check formatting only'
complete -c mdhavers -n '__fish_seen_subcommand_from fmt' -F -a '*.braw' -d 'Source file'

# check command
complete -c mdhavers -n '__fish_seen_subcommand_from check' -s h -l help -d 'Show help'
complete -c mdhavers -n '__fish_seen_subcommand_from check' -F -a '*.braw' -d 'Source file'

# help command
complete -c mdhavers -n '__fish_seen_subcommand_from help' -a 'run build repl fmt check' -d 'Command'

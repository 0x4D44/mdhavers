#compdef mdhavers

# Zsh completion script for mdhavers
# Install: copy to ~/.zsh/completions/_mdhavers

_mdhavers() {
    local -a commands
    local -a global_opts

    commands=(
        'run:Run a mdhavers program'
        'build:Compile a mdhavers program to native executable'
        'repl:Start the interactive REPL'
        'fmt:Format mdhavers source code'
        'check:Check a mdhavers program for errors'
        'help:Show help information'
    )

    global_opts=(
        '(-h --help)'{-h,--help}'[Show help information]'
        '(-V --version)'{-V,--version}'[Show version information]'
    )

    _arguments -C \
        $global_opts \
        '1: :->command' \
        '*: :->args'

    case $state in
        command)
            _describe -t commands 'mdhavers commands' commands
            ;;
        args)
            case $words[2] in
                run)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:source file:_files -g "*.braw"'
                    ;;
                build)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
                        '--opt-level[Optimization level (0-3)]:level:(0 1 2 3)' \
                        '1:source file:_files -g "*.braw"'
                    ;;
                repl)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]'
                    ;;
                fmt)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '--check[Check formatting without modifying]' \
                        '*:source file:_files -g "*.braw"'
                    ;;
                check)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Show help]' \
                        '1:source file:_files -g "*.braw"'
                    ;;
                help)
                    _describe -t commands 'mdhavers commands' commands
                    ;;
            esac
            ;;
    esac
}

_mdhavers "$@"

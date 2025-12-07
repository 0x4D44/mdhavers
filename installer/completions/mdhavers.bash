# Bash completion script for mdhavers
# Install: source this file or copy to /etc/bash_completion.d/

_mdhavers_completions() {
    local cur prev words cword
    _init_completion || return

    local commands="run build repl fmt check help"
    local global_opts="--help --version -h -V"

    case "${prev}" in
        run|build|fmt|check)
            # Complete .braw files
            COMPREPLY=( $(compgen -f -X '!*.braw' -- "${cur}") )
            COMPREPLY+=( $(compgen -d -- "${cur}") )
            return 0
            ;;
        -o|--output)
            # Complete any file for output
            COMPREPLY=( $(compgen -f -- "${cur}") )
            return 0
            ;;
        --opt-level)
            COMPREPLY=( $(compgen -W "0 1 2 3" -- "${cur}") )
            return 0
            ;;
        mdhavers)
            COMPREPLY=( $(compgen -W "${commands} ${global_opts}" -- "${cur}") )
            return 0
            ;;
    esac

    case "${words[1]}" in
        run)
            if [[ "${cur}" == -* ]]; then
                COMPREPLY=( $(compgen -W "--help -h" -- "${cur}") )
            else
                COMPREPLY=( $(compgen -f -X '!*.braw' -- "${cur}") )
                COMPREPLY+=( $(compgen -d -- "${cur}") )
            fi
            ;;
        build)
            if [[ "${cur}" == -* ]]; then
                COMPREPLY=( $(compgen -W "--help -h -o --output --opt-level" -- "${cur}") )
            else
                COMPREPLY=( $(compgen -f -X '!*.braw' -- "${cur}") )
                COMPREPLY+=( $(compgen -d -- "${cur}") )
            fi
            ;;
        repl)
            COMPREPLY=( $(compgen -W "--help -h" -- "${cur}") )
            ;;
        fmt)
            if [[ "${cur}" == -* ]]; then
                COMPREPLY=( $(compgen -W "--help -h --check" -- "${cur}") )
            else
                COMPREPLY=( $(compgen -f -X '!*.braw' -- "${cur}") )
                COMPREPLY+=( $(compgen -d -- "${cur}") )
            fi
            ;;
        check)
            if [[ "${cur}" == -* ]]; then
                COMPREPLY=( $(compgen -W "--help -h" -- "${cur}") )
            else
                COMPREPLY=( $(compgen -f -X '!*.braw' -- "${cur}") )
                COMPREPLY+=( $(compgen -d -- "${cur}") )
            fi
            ;;
        help)
            COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
            ;;
        *)
            if [[ "${cur}" == -* ]]; then
                COMPREPLY=( $(compgen -W "${global_opts}" -- "${cur}") )
            else
                COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
            fi
            ;;
    esac
}

complete -F _mdhavers_completions mdhavers

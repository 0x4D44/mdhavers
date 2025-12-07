#!/bin/sh
# mdhavers environment setup
# Source this file to add mdhavers to your PATH
#
# Add this to your shell's rc file:
#   . "$HOME/.mdhavers/env"

export MDHAVERS_HOME="$HOME/.mdhavers"
export PATH="$MDHAVERS_HOME/bin:$PATH"

# Optional: Enable shell completions based on current shell
if [ -n "$BASH_VERSION" ] && [ -f "$MDHAVERS_HOME/completions/mdhavers.bash" ]; then
    . "$MDHAVERS_HOME/completions/mdhavers.bash"
elif [ -n "$ZSH_VERSION" ] && [ -f "$MDHAVERS_HOME/completions/mdhavers.zsh" ]; then
    . "$MDHAVERS_HOME/completions/mdhavers.zsh"
fi

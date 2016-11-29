## Force zsh aliases

This is a program that will force you to use your zsh aliases. It works by installing a hook that executes before any commands. It uses a client-server architecture, each client being invidual shells and the server running in the background on port 5571.

## Installation

First, install cargo, rust, and the binaries in this package (through cargo).
```bash
brew tap cheba/rust-nightly
brew install rust-nightly
git clone https://github.com/marcusbuffett/zsh-force-alias
pushd force_alias
cargo install --path .
popd
rm -rf force_alias
```

Then, add the following snippet to the bottom of your zsh file.

```bash
#####################
## ZSH force alias ##
zle -N expand-aliases; bindkey '^E' expand-aliases
(force-alias-server > /dev/null &) > /dev/null 2>&1
if [[ -z "$NO_CHECK" ]]; then force-alias-client --init --pid $$; fi
function force_alias_hook() {
  if [[ -n "$NO_CHECK" ]]; then zle accept-line; return; fi
  force-alias-client --pid $$ -- $BUFFER
  if [[ $? -eq 1 ]]; then BUFFER=""; fi
  zle accept-line
}
autoload -U add-zsh-hook; zle -N force_alias_hook;
bindkey '^J' force_alias_hook; bindkey '^M' force_alias_hook;
## End ZSH force alias ##
#########################
```

Now open a new shell and test it out by using a command that you have aliased, it should prevent you from executing it.

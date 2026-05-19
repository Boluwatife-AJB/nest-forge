# Shell Completions

forge can generate shell completions for bash, zsh, fish, and PowerShell.

## Bash

```bash
# Add to ~/.bashrc
eval "$(forge completions bash)"

# Or write to a completions file
forge completions bash > ~/.local/share/bash-completion/completions/forge
```

## Zsh

```zsh
# Add to ~/.zshrc
eval "$(forge completions zsh)"

# Or write to a site-functions directory
forge completions zsh > "${fpath[1]}/_forge"
```

## Fish

```fish
forge completions fish > ~/.config/fish/completions/forge.fish
```

## PowerShell

```powershell
# Add to your PowerShell profile ($PROFILE)
Invoke-Expression (& forge completions powershell | Out-String)
```
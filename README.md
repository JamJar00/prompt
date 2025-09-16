# Prompt
Prompt is my personal terminal prompt. It's simple, super fast and looks nice.

![Screenshot](./Screenshot.png)

## Installation
Download the latest build of the prompt from GitHub actions and store it on your $PATH

For bash, add the following to your `~/.bashrc` file:
```bash
if [ "$TERM_PROGRAM" = "iTerm.app" ]
    export PS1='$(prompt --exit-code $? --iterm2)'
else
    export PS1='$(prompt --exit-code $?)'
fi
```

For fish, add the following to your `~/.config/fish/config.fish`
```fish
function fish_prompt
  if [ "$TERM_PROGRAM" = "iTerm.app" ]
    prompt --exit-code $status --iterm2
  else
    prompt --exit-code $status
  end
end
```

## Usage
You can see help for arguments at any time using `prompt -h`

### Chevrons
Hopefully most of the prompt is faily self explanatory, however the three chevrons can take some getting used to:
```
❯❯❯
││└ Unpushed changes (yellow)/Unpulled changes (blue)/No upstream (white)
│└─ Uncommitted changes (yellow)/Untracked files (blue)
└── Exit code
```
To see this information, run `prompt --explain` at any time

### Custom Content
You can add custom content into the prompt using the `--message` flag. For example, to add the current shell name in you could use:
```bash
prompt --exit-code $? --message "$SHELL"
```

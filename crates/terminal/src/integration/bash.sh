unset HISTFILE
HISTSIZE=1000
__opsis_prompt() {
  local code=$?
  if [ -n "$__opsis_started" ]; then
    printf '\033]133;D;%s\007' "$code"
  fi
  __opsis_started=1
  printf '\033]133;A\007'
}
PROMPT_COMMAND=__opsis_prompt
PS1='\[\e[1;32m\]opsis\[\e[0m\]:\[\e[1;34m\]\w\[\e[0m\]$ \[\e]133;B\a\]'
PS0='\e]133;C\a'

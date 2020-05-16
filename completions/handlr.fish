function __handlr_autocomplete
  function subcommands
    set -l handlr_commands 'get help launch list open set unset'
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "get" -d "Show handler for mime"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "launch" -d "Launch given handler with path/args"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "list" -d "Show handlers (default applications)"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "open" -d "Open path/URL with default handler (like xdg-open)"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "set" -d "Set handler for extension (e.g. pdf) or mime type"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "unset" -d "Unset handler"
  end

  function _set
    complete -f -c handlr -n '__fish_seen_subcommand_from set' -s "em" 
    complete -f -c handlr -n '__fish_seen_subcommand_from set; __fish_prev_arg_in set' -a '-e -m'

    complete -f -c handlr -n '__fish_seen_subcommand_from set; __fish_prev_arg_in "-e"' -a "(handlr autocomplete -e)"
    complete -f -c handlr -n '__fish_seen_subcommand_from set; __fish_prev_arg_in "-m"' -a '(handlr autocomplete -m)'

    complete -f -c handlr -n '__fish_seen_subcommand_from set; set -l last (commandline -pco)[-2]; [ "$last" = "-e" ]' -a '(handlr autocomplete -d)' -d "desc1 desc2"
    complete -f -c handlr -n '__fish_seen_subcommand_from set; set -l last (commandline -pco)[-2]; [ "$last" = "-m" ]' -a '(handlr autocomplete -d)'
  end

  subcommands
  _set
  complete -f -c handlr -n '__fish_seen_subcommand_from get' -l 'json' -a '(handlr autocomplete -m)'
  complete -f -c handlr -n '__fish_seen_subcommand_from unset' -a '(handlr autocomplete -m)'
  complete -f -c handlr -n '__fish_seen_subcommand_from launch; __fish_prev_arg_in launch' -a '(handlr autocomplete -m)'

end

__handlr_autocomplete
